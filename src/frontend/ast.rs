#[derive(Debug)]
pub enum Statement {
    CreateTable {
        name: String,
        columns: Vec<ColumnDef>,
    },
    Select {
        columns: Vec<String>,
        from: String,
        filter: Option<Expression>,
    },
    Insert {
        table: String,
        columns: Vec<String>,
        values: Vec<Expression>,
    },
    Update {
        table: String,
        updates: Vec<(String, Expression)>,
        filter: Option<Expression>,
    },
    Delete {
        table: String,
        filter: Option<Expression>,
    },
}

#[derive(Debug)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
}

#[derive(Debug)]
pub enum Expression {
    Literal(String),
    Integer(i64),
    Identifier(String),
    BinaryOp {
        left: Box<Expression>,
        op: String,
        right: Box<Expression>,
    },
}
