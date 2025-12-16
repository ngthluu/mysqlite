use std::process::exit;

#[derive(Debug)]
pub enum Command {
    Meta(String),
    Statement(String),
    Empty,
}

pub fn parse_command(input: &str) -> Command {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Command::Empty;
    }

    if trimmed.starts_with(".") {
        return Command::Meta(trimmed.to_string());
    }

    Command::Statement(trimmed.to_string())
}

pub fn execute_command(command: Command) {
    match command {
        Command::Meta(cmd) => {
            match cmd.as_str() {
                ".exit" => {
                    println!("Bye!");
                    exit(0);
                }
                _ => {
                    println!("Meta-command not recognized: {}", cmd);
                }
            }
        }
        Command::Statement(sql) => {
            println!("[EXECUTION] SQL Statement: '{}'", sql);
        }
        Command::Empty => {}
    }
}
