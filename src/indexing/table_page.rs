use crate::backend::pager::{PAGE_SIZE, PageData};

// Memory header layout
// Bytes 0-3: ID of this page
// Bytes 4-7: ID of the prev page
// Bytes 8-11: ID of the next page
// Bytes 12-15: Points to where data starts
// Bytes 16-19: How many slots / tuples
const OFFSET_PAGE_ID: usize = 0;
const OFFSET_PREV_PAGE_ID: usize = 4;
const OFFSET_NEXT_PAGE_ID: usize = 8;
const OFFSET_FREE_SPACE: usize = 12;
const OFFSET_SLOT_COUNT: usize = 16;

pub const HEADER_SIZE: usize = 20;
const SLOT_SIZE: usize = 8; // Offset(4) + Length(4)

// Table page is for solving the fragmentation problem
// [Tuple A][Tuple B] -> delete [Tuple A] -> leave a hole
// If the next tuple > [Tuple A] -> Not fit
// To solve this problem: use Slotted Page
// A slotted page has 2 growing regions:
// 1. Header & Slot: Top-down
// 2. Tuple Data: Bottom-up
// 3. Free space: the empty gap in the middle. 2 regions meet -> Page is full
pub struct TablePage<'a> {
    data: &'a mut PageData,
}

impl<'a> TablePage<'a> {
    pub fn new(data: &'a mut PageData) -> Self {
        Self { data }
    }

    // Init a new page
    pub fn init(&mut self, page_id: u32, prev_id: u32) {
        self.write_u32(OFFSET_PAGE_ID, page_id);
        self.write_u32(OFFSET_PREV_PAGE_ID, prev_id);
        self.write_u32(OFFSET_NEXT_PAGE_ID, 0); // 0 acts as null
        self.write_u32(OFFSET_FREE_SPACE, PAGE_SIZE as u32); // Points to end of page
        self.write_u32(OFFSET_SLOT_COUNT, 0);
    }

    pub fn get_next_page_id(&self) -> Option<u32> {
        let id = self.read_u32(OFFSET_NEXT_PAGE_ID);
        if id == 0 { None } else { Some(id) }
    }

    pub fn set_next_page_id(&mut self, id: u32) {
        self.write_u32(OFFSET_NEXT_PAGE_ID, id);
    }

    pub fn get_slot_count(&self) -> u32 {
        self.read_u32(OFFSET_SLOT_COUNT)
    }

    pub fn insert_tuple(&mut self, tuple: &[u8]) -> Option<u16> {
        let size = tuple.len();
        let free_space_ptr = self.read_u32(OFFSET_FREE_SPACE) as usize;
        let slot_count = self.get_slot_count() as usize;

        // Calculate where the "Slots Area" ends
        let slots_end = HEADER_SIZE + (slot_count * SLOT_SIZE);

        // Calculate required space: Data Size + New Slot Entry (8 bytes)
        let space_needed = size + SLOT_SIZE;
        let space_available = free_space_ptr - slots_end;

        // Not enough space
        if space_needed > space_available {
            return None;
        }

        // 1. Move the Free Space Pointer UP (towards 0)
        let new_free_ptr = free_space_ptr - size;
        self.write_u32(OFFSET_FREE_SPACE, new_free_ptr as u32);

        // 2. Write the Tuple Data
        self.data[new_free_ptr..new_free_ptr + size].copy_from_slice(tuple);

        // 3. Write the Slot Entry (Offset, Length)
        // The slot is written at the current end of the slots area
        let slot_offset = slots_end;
        self.write_u32(slot_offset, new_free_ptr as u32); // Tuple Offset
        self.write_u32(slot_offset + 4, size as u32); // Tuple Length

        // 4. Increment Slot Count
        self.write_u32(OFFSET_SLOT_COUNT, (slot_count + 1) as u32);

        // Return the Slot ID (index)
        Some(slot_count as u16)
    }

    pub fn get_tuple(&self, slot_id: u16) -> Option<Vec<u8>> {
        let slot_count = self.get_slot_count() as u16;
        if slot_id >= slot_count {
            return None;
        }

        let slot_offset = HEADER_SIZE + (slot_id as usize * SLOT_SIZE);
        let tuple_offset = self.read_u32(slot_offset) as usize;
        let tuple_length = self.read_u32(slot_offset + 4) as usize;

        let mut result = vec![0u8; tuple_length];
        result.copy_from_slice(&self.data[tuple_offset..tuple_offset + tuple_length]);
        Some(result)
    }

    fn read_u32(&self, offset: usize) -> u32 {
        let bytes = &self.data[offset..offset + 4];
        u32::from_ne_bytes(bytes.try_into().unwrap())
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        self.data[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
    }
}
