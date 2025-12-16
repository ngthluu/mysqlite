mod repl;
mod pager;
mod btree;
mod row;
mod types;
mod schema;

use std::io::{self, Write};
use repl::{parse_command, execute_command};

use crate::pager::Pager;

fn main() {
    // Setup pager
    let mut pager = match Pager::new("mydb.db") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error initializing Pager: {}", e);
            return;
        }
    };

    println!("Welcome to mysqlite. Enter .exit to quit.");
    loop {
        print!("db > ");
        io::stdout().flush().unwrap();

        let mut input= String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = parse_command(&input);
                execute_command(command);
            }
            Err(e) => {
                println!("Error reading input: {}", e);
            }
        }
    }
}
