use crate::pager::Pager;

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Internal,
    Leaf
}

impl NodeType {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => NodeType::Internal,
            1 => NodeType::Leaf,
            _ => panic!("Invalid node type byte")
        }
    }

    pub fn as_byte(&self) -> u8 {
        match self {
            NodeType::Internal => 0,
            NodeType::Leaf => 1,
        }
    }
}

pub struct Table {
    pager: Pager,
    root_page_num: u32,
}
