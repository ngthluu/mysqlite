#[derive(Debug)]
pub enum DataType {
    Integer,
    Varchar,
    Boolean,
}

pub enum Value {
    Integer(i32),
    Varchar(String),
    Boolean(bool),
}

impl Value {
    pub fn serialize_size(&self) -> usize {
        match self {
            Value::Integer(_) => 4,
            // 2 bytes to save string size & metadata
            Value::Varchar(s) => 2 + s.len(),
            Value::Boolean(_) => 1,
        }
    }
}
