use crate::indexing::{
    table_heap::TableHeap,
    table_page::{OFFSET_NEXT_PAGE_ID, OFFSET_SLOT_COUNT},
};
use std::sync::Arc;

// An iterator that scans the entire table heap sequentially
pub struct TableIterator {
    table_heap: Arc<TableHeap>,
    current_page_id: usize,
    current_slot_id: u16,
}

impl TableIterator {
    pub fn new(table_heap: Arc<TableHeap>, start_page_id: usize) -> Self {
        Self {
            table_heap,
            current_page_id: start_page_id,
            current_slot_id: 0,
        }
    }

    // Moves to the next tuple and returns it.
    // Returns None if we reached the end of the table.
    pub fn next(&mut self) -> Option<Vec<u8>> {
        loop {
            // 1. Fetch the current page
            let frame_arc = self
                .table_heap
                .cache
                .fetch_page(self.current_page_id)
                .ok()?;
            let frame = frame_arc.read().unwrap();

            if let Some(ref page) = frame.page {
                let count_bytes = &page.data[OFFSET_SLOT_COUNT..OFFSET_SLOT_COUNT + 4];
                let slot_count = u32::from_ne_bytes(count_bytes.try_into().unwrap()) as u16;

                // 2. Check if we have more slots in this page
                if self.current_slot_id < slot_count {
                    drop(frame); // Release lock strictly before calling other methods

                    let tuple = self
                        .table_heap
                        .get_tuple(self.current_page_id, self.current_slot_id)
                        .ok();
                    self.current_slot_id += 1;

                    if tuple.is_some() {
                        return tuple;
                    }
                } else {
                    // 3. No more slots in this page. Move to next page.
                    let next_page_bytes = &page.data[OFFSET_NEXT_PAGE_ID..OFFSET_NEXT_PAGE_ID + 4];
                    let next_page_id =
                        u32::from_ne_bytes(next_page_bytes.try_into().unwrap()) as usize;

                    drop(frame); // Release lock
                    self.table_heap
                        .cache
                        .unpin_page(self.current_page_id, false);

                    if next_page_id == 0 {
                        return None; // End of Linked List
                    }

                    // Advance to next page, reset slot to 0
                    self.current_page_id = next_page_id;
                    self.current_slot_id = 0;
                }
            } else {
                // Should not happen if logic is correct
                return None;
            }
        }
    }
}
