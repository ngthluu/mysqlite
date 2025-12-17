use crate::types::DataType;

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub is_primary_key: bool,
}

#[derive(Debug)]
pub struct Schema {
    pub name: String,
    pub columns: Vec<Column>,
}

impl Schema {
    pub fn new(name: String, columns: Vec<Column>) -> Self {
        Schema { name, columns }
    }

    pub fn primary_key_index(&self) -> usize {
        self.columns
            .iter()
            .position(|c| c.is_primary_key)
            .expect("Schema must have a primary key")
    }
}
