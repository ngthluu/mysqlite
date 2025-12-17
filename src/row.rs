use crate::{
    schema::Schema,
    types::{DataType, Value},
};

pub struct Row {
    pub values: Vec<Value>,
}

impl Row {
    pub fn new(values: Vec<Value>) -> Self {
        Row { values }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        for value in self.values.iter() {
            match value {
                Value::Integer(val) => {
                    buffer.extend_from_slice(&val.to_le_bytes());
                }
                Value::Boolean(val) => buffer.push(if *val { 1 } else { 0 }),
                Value::Varchar(val) => {
                    let len = val.len() as u16;
                    buffer.extend_from_slice(&len.to_le_bytes());
                    buffer.extend_from_slice(val.as_bytes());
                }
            }
        }

        buffer
    }

    pub fn deserialize(data: &[u8], schema: &Schema) -> Self {
        let mut values = Vec::new();
        let mut offset = 0;

        for column in &schema.columns {
            match column.data_type {
                DataType::Integer => {
                    let mut bytes = [0u8; 4];
                    bytes.copy_from_slice(&data[offset..offset + 4]);
                    values.push(Value::Integer(i32::from_le_bytes(bytes)));
                    offset += 4;
                }
                DataType::Varchar => {
                    let mut len_bytes = [0u8; 2];
                    len_bytes.copy_from_slice(&data[offset..offset + 2]);
                    let len = u16::from_le_bytes(len_bytes) as usize;
                    offset += 2;

                    let s = String::from_utf8_lossy(&data[offset..offset + len]).to_string();
                    values.push(Value::Varchar(s));
                    offset += len;
                }
                DataType::Boolean => {
                    values.push(Value::Boolean(data[offset] == 1));
                    offset += 1;
                }
            }
        }

        Row { values }
    }
}
