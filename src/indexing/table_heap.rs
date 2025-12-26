use std::sync::Arc;

use crate::{
    backend::cache::Cache,
    indexing::table_page::{HEADER_SIZE, TablePage},
};

// Table Heap
// DO NOT hold data itself. Just hold the ID of the first page
pub struct TableHeap {
    first_page_id: usize,
    cache: Arc<Cache>,
}

impl TableHeap {
    pub fn new(cache: Arc<Cache>, first_page_id: usize) -> Self {
        Self {
            first_page_id,
            cache,
        }
    }

    /// Insert a tuple into the table.
    /// Returns (PageID, SlotID) on success.
    pub fn insert(&self, tuple: &[u8]) -> Result<(usize, u16), String> {
        let mut current_page_id = self.first_page_id;

        loop {
            // 1. Fetch the page from Buffer Pool
            let frame_arc = self
                .cache
                .fetch_page(current_page_id)
                .map_err(|_| "Failed to fetch page")?;

            // Lock for writing
            let mut frame = frame_arc.write().unwrap();

            if let Some(ref mut page) = frame.page {
                let mut table_page = TablePage::new(&mut page.data);

                // 2. Try to insert into this page
                if let Some(slot_id) = table_page.insert_tuple(tuple) {
                    drop(frame); // Release lock
                    self.cache.unpin_page(current_page_id, true); // Mark dirty
                    return Ok((current_page_id, slot_id));
                }

                // 3. Page is full. Check for next page.
                match table_page.get_next_page_id() {
                    Some(next_id) => {
                        // Move to next page
                        drop(frame);
                        self.cache.unpin_page(current_page_id, false);
                        current_page_id = next_id as usize;
                    }
                    None => {
                        // 4. End of list. Create a NEW page.
                        drop(frame); // Unpin current (we'll re-fetch it shortly to link)

                        // Try to fetch the 'next' logical ID.
                        let new_page_id = current_page_id + 1;
                        let new_frame_arc = self
                            .cache
                            .fetch_page(new_page_id)
                            .map_err(|_| "Failed to allocate new page")?;

                        {
                            let mut new_frame = new_frame_arc.write().unwrap();
                            if let Some(ref mut new_page_data) = new_frame.page {
                                let mut new_table_page = TablePage::new(&mut new_page_data.data);
                                new_table_page.init(new_page_id as u32, current_page_id as u32);
                            }
                        }
                        self.cache.unpin_page(new_page_id, true); // Mark new page dirty

                        // 5. Link OLD page to NEW page
                        let old_frame_arc = self.cache.fetch_page(current_page_id).unwrap();
                        {
                            let mut old_frame = old_frame_arc.write().unwrap();
                            if let Some(ref mut old_page_data) = old_frame.page {
                                let mut old_table_page = TablePage::new(&mut old_page_data.data);
                                old_table_page.set_next_page_id(new_page_id as u32);
                            }
                        }
                        self.cache.unpin_page(current_page_id, true); // Mark old page dirty

                        // Loop will continue, current_page_id becomes new_page_id,
                        // and we will insert into the empty new page on next iteration.
                        current_page_id = new_page_id;
                    }
                }
            } else {
                return Err("Frame contained no page data".to_string());
            }
        }
    }

    pub fn get_tuple(&self, page_id: usize, slot_id: u16) -> Result<Vec<u8>, String> {
        let frame_arc = self
            .cache
            .fetch_page(page_id)
            .map_err(|_| "Failed to fetch page")?;

        let frame = frame_arc.read().unwrap(); // Read lock is enough

        if let Some(ref page) = frame.page {
            let slot_offset = HEADER_SIZE + (slot_id as usize * 8);

            let tuple_offset_bytes = &page.data[slot_offset..slot_offset + 4];
            let tuple_len_bytes = &page.data[slot_offset + 4..slot_offset + 8];

            let offset = u32::from_ne_bytes(tuple_offset_bytes.try_into().unwrap()) as usize;
            let len = u32::from_ne_bytes(tuple_len_bytes.try_into().unwrap()) as usize;

            let mut result = vec![0u8; len];
            result.copy_from_slice(&page.data[offset..offset + len]);

            drop(frame);
            self.cache.unpin_page(page_id, false);

            Ok(result)
        } else {
            Err("Frame empty".to_string())
        }
    }
}
