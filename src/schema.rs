use crate::types::DbDataType;

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub data_type: DbDataType,
    pub is_primary_key: bool,
    pub max_length: Option<usize>
}

#[derive(Debug)]
pub struct Schema {
    pub name: String,
    pub columns: Vec<Column>
}
