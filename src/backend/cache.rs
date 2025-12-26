use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex, RwLock},
};

use crate::backend::{
    lru_replacer::LRUReplacer,
    pager::{PAGE_SIZE, Page, Pager},
};

#[derive(Debug)]
pub struct Frame {
    // The actual data
    // None if frame is empty
    pub page: Option<Page>,

    // Has the data been modified in memory?
    pub is_dirty: bool,

    // How many threads are currently using this?
    pub pin_count: usize,
}

impl Frame {
    fn new() -> Self {
        Self {
            page: None,
            is_dirty: false,
            pin_count: 0,
        }
    }
}

pub struct Cache {
    pager: Mutex<Pager>,

    // A fixed pool of memory
    // use RwLock: multiple threads can READ a page at once
    frames: Vec<Arc<RwLock<Frame>>>,

    // Maps page_id -> frame_id
    pf_table: Mutex<HashMap<usize, usize>>,

    // List of frame_id that have never been used
    // completely empty
    free_list: Mutex<Vec<usize>>,

    // Algorithm to decide what frame should kick out
    // if the buffer pool is full
    replacer: Mutex<LRUReplacer>,
}

impl Cache {
    pub fn new(pager: Pager, pool_size: usize) -> Self {
        let mut frames = Vec::with_capacity(pool_size);
        let mut free_list = Vec::with_capacity(pool_size);

        for i in 0..pool_size {
            frames.push(Arc::new(RwLock::new(Frame::new())));
            free_list.push(i);
        }

        Self {
            pager: Mutex::new(pager),
            frames,
            pf_table: Mutex::new(HashMap::new()),
            free_list: Mutex::new(free_list),
            replacer: Mutex::new(LRUReplacer::new()),
        }
    }

    // Retreive a page, from memory (fast) or disk (slow)
    pub fn fetch_page(&self, page_id: usize) -> io::Result<Arc<RwLock<Frame>>> {
        let mut pf_table = self.pf_table.lock().unwrap();
        let mut replacer = self.replacer.lock().unwrap();

        // Hit Cache
        // Return the frame in cache
        if let Some(&frame_id) = pf_table.get(&page_id) {
            let frame_arc = self.frames[frame_id].clone();
            let mut frame = frame_arc.write().unwrap();

            frame.pin_count += 1;
            replacer.pin(frame_id);

            return Ok(frame_arc.clone());
        }

        // Miss Cache
        // Find a free frame, and read from pager (disk)
        let frame_id = self.find_free_frame(&mut replacer)?;
        let frame_arc = self.frames[frame_id].clone();
        let mut frame = frame_arc.write().unwrap();

        // Clean up the page if frame contains any page
        if let Some(ref old_page) = frame.page {
            // If the frame is dirty -> update to pager first
            if frame.is_dirty {
                let mut pager = self.pager.lock().unwrap();
                pager.write_page(old_page)?;
            }

            pf_table.remove(&old_page.id);
        }

        // Read from pager
        let mut pager = self.pager.lock().unwrap();
        let page_data = match pager.read_page(page_id) {
            Ok(p) => p,
            Err(_) => Page {
                id: page_id,
                data: [0; PAGE_SIZE],
            },
        };

        frame.page = Some(page_data);
        frame.pin_count = 1;
        frame.is_dirty = false;

        pf_table.insert(page_id, frame_id);
        replacer.pin(frame_id);

        Ok(frame_arc.clone())
    }

    // Force a specific page to be written to disk
    pub fn flush_page(&self, page_id: usize) -> io::Result<()> {
        let pf_table = self.pf_table.lock().unwrap();

        if let Some(&frame_id) = pf_table.get(&page_id) {
            let frame_arc = self.frames[frame_id].clone();
            let mut frame = frame_arc.write().unwrap();

            if let Some(ref page) = frame.page {
                let mut pager = self.pager.lock().unwrap();
                pager.write_page(page)?;
                frame.is_dirty = false;
            }
        }

        Ok(())
    }

    // Release the lock
    // System knows this page is free to be remove later
    pub fn unpin_page(&self, page_id: usize, is_dirty: bool) -> bool {
        let pf_table = self.pf_table.lock().unwrap();

        if let Some(&frame_id) = pf_table.get(&page_id) {
            let frame_arc = self.frames[frame_id].clone();
            let mut frame = frame_arc.write().unwrap();

            if frame.pin_count == 0 {
                return false; // Already unpinned
            }

            frame.pin_count -= 1;
            if is_dirty {
                frame.is_dirty = true;
            }

            // If pin_count hits 0, this frame is now a candidate for eviction
            if frame.pin_count == 0 {
                let mut replacer = self.replacer.lock().unwrap();
                replacer.unpin(frame_id);
            }
            return true;
        }
        false
    }

    // Find a free frame or remove a victim
    fn find_free_frame(&self, replacer: &mut LRUReplacer) -> io::Result<usize> {
        // Cheapest: try from free_list
        let mut free_list = self.free_list.lock().unwrap();
        if let Some(fid) = free_list.pop() {
            return Ok(fid);
        }

        // Try to find a victim in LRU
        if let Some(victim_id) = replacer.victim() {
            return Ok(victim_id);
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "Buffer pool full: All pages are pinned",
        ))
    }
}
