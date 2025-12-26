use std::collections::VecDeque;

#[derive(Debug)]
pub struct LRUReplacer {
    // Double-Ended Queue (Dequeue)
    // FRONT = Oldest (Least Recently Used) -> Candidate to remove
    // BACK = Newest (Most Recently Used)
    free_frames: VecDeque<usize>,
}

impl LRUReplacer {
    pub fn new() -> Self {
        Self {
            free_frames: VecDeque::new(),
        }
    }

    // Releasing a frame
    // Check if it's already in the list (linear scan O(n))
    // If not contains, add to the BACK of the list (Most Recently Used)
    pub fn unpin(&mut self, frame_id: usize) {
        if !self.free_frames.contains(&frame_id) {
            self.free_frames.push_back(frame_id);
        }
    }

    // Using a frame
    // Find the position of frame (linear scan O(n))
    // Remove it from the list
    pub fn pin(&mut self, frame_id: usize) {
        if let Some(pos) = self.free_frames.iter().position(|&x| x == frame_id) {
            self.free_frames.remove(pos);
        }
    }

    // Case: buffer pool is full, need an empty slot
    // Pop the FRONT (Least Recently Used)
    pub fn victim(&mut self) -> Option<usize> {
        self.free_frames.pop_front()
    }
}
