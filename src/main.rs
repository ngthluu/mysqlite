mod backend;
mod cli;
mod frontend;
mod indexing;
mod row;
mod schema;
mod types;

use cli::repl::{execute_command, parse_command};
use std::io::{self, Write};

use crate::{
    backend::pager::Pager,
    indexing::btree::BTreeTable,
    schema::{Column, Schema},
    types::DataType,
};

fn main() {
    // Setup schema
    let schema = Schema::new(
        "my_table".to_string(),
        vec![
            Column {
                name: "id".to_string(),
                data_type: DataType::Integer,
                is_primary_key: true,
            },
            Column {
                name: "username".to_string(),
                data_type: DataType::Varchar,
                is_primary_key: false,
            },
            Column {
                name: "email".to_string(),
                data_type: DataType::Varchar,
                is_primary_key: false,
            },
        ],
    );

    // Setup pager
    let pager = match Pager::new("mydb.db") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error initializing Pager: {}", e);
            return;
        }
    };

    // Setup btree table
    let mut table = BTreeTable::new(pager, schema);

    println!("Welcome to mysqlite. Enter `exit` to quit.");
    loop {
        print!("db > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = parse_command(&input);
                execute_command(command, &mut table);
            }
            Err(e) => {
                println!("Error reading input: {}", e);
            }
        }
    }
}
