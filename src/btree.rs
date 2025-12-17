use crate::{
    pager::{PAGE_SIZE, Pager},
    row::Row,
    schema::Schema,
};

// --- Cấu trúc Header ---
const NODE_TYPE_SIZE: usize = 1;
const IS_ROOT_SIZE: usize = 1;
const PARENT_POINTER_SIZE: usize = 4;
const NUM_CELLS_SIZE: usize = 2;
const CELL_CONTENT_START_SIZE: usize = 2;

const COMMON_NODE_HEADER_SIZE: usize =
    NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE + NUM_CELLS_SIZE + CELL_CONTENT_START_SIZE;

const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + 4; // + Sibling Pointer

pub struct BTreeTable {
    pub pager: Pager,
    pub root_page_num: u32,
    pub schema: Schema,
}

impl BTreeTable {
    pub fn new(pager: Pager, schema: Schema) -> Self {
        BTreeTable {
            pager,
            root_page_num: 0,
            schema,
        }
    }

    // Khởi tạo một trang mới dưới dạng Leaf Node
    pub fn initialize_leaf_node(&mut self, page_num: u32) {
        let page = self.pager.get_page(page_num).unwrap();

        // Byte 0: Node Type (1 = Leaf)
        page[0] = 1;
        // Bytes 6-7: Num Cells (khởi tạo là 0)
        page[6..8].copy_from_slice(&0u16.to_le_bytes());
        // Bytes 8-9: Cell Content Start (khởi tạo ở cuối trang)
        page[8..10].copy_from_slice(&(PAGE_SIZE as u16).to_le_bytes());
    }

    pub fn insert(&mut self, row: Row) {
        let page_num = self.root_page_num;
        let serialized_row = row.serialize();
        let row_size = serialized_row.len();

        // Lấy khóa chính (ví dụ cột đầu tiên) để định danh cell
        let key = match row.values[self.schema.primary_key_index()] {
            crate::types::Value::Integer(k) => k,
            _ => panic!("Primary key must be an integer for now"),
        };

        let page = self.pager.get_page(page_num).unwrap();

        // 1. Đọc metadata từ Header
        let mut num_cells = u16::from_le_bytes(page[6..8].try_into().unwrap());
        let mut content_start = u16::from_le_bytes(page[8..10].try_into().unwrap());

        // 2. Tính toán vị trí mới cho dữ liệu (đẩy từ dưới lên)
        // Mỗi cell gồm: Key (4 bytes) + Row Data
        let cell_size = 4 + row_size;
        content_start -= cell_size as u16;

        // 3. Ghi dữ liệu vào vùng Content
        let mut offset = content_start as usize;
        page[offset..offset + 4].copy_from_slice(&key.to_le_bytes()); // Ghi Key
        page[offset + 4..offset + cell_size].copy_from_slice(&serialized_row); // Ghi Payload

        // 4. Ghi Pointer trỏ đến Cell đó vào mảng Slot
        // Slot array nằm ngay sau Header
        let slot_offset = LEAF_NODE_HEADER_SIZE + (num_cells as usize * 2);
        page[slot_offset..slot_offset + 2].copy_from_slice(&content_start.to_le_bytes());

        // 5. Cập nhật Header
        num_cells += 1;
        page[6..8].copy_from_slice(&num_cells.to_le_bytes());
        page[8..10].copy_from_slice(&content_start.to_le_bytes());

        println!("Inserted key {} at offset {}", key, content_start);
    }

    pub fn select_all(&mut self) -> Vec<Row> {
        let mut results = Vec::new();
        let page = self.pager.get_page(self.root_page_num).unwrap();

        let num_cells = u16::from_le_bytes(page[6..8].try_into().unwrap());

        for i in 0..num_cells {
            // Đọc offset từ slot array
            let slot_offset = LEAF_NODE_HEADER_SIZE + (i as usize * 2);
            let cell_offset =
                u16::from_le_bytes(page[slot_offset..slot_offset + 2].try_into().unwrap()) as usize;

            // Bỏ qua 4 byte đầu của cell (là key) để đọc payload
            let row_data = &page[cell_offset + 4..];
            let row = Row::deserialize(row_data, &self.schema);
            results.push(row);
        }
        results
    }
}
