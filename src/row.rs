pub enum Value {
    Int32(i32),
    Varchar(String),
}

pub struct Row {
    pub values: Vec<Value>
}

impl Row {

}