mod repl;

use std::io::{self, Write};
use repl::{parse_command, execute_command};

fn main() {
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
