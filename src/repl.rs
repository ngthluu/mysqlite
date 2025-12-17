use std::process::exit;

use crate::{btree::BTreeTable, row::Row, types::Value};

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

pub fn execute_command(command: Command, table: &mut BTreeTable) {
    match command {
        Command::Meta(cmd) => match cmd.as_str() {
            "exit" => {
                println!("Bye!");
                exit(0);
            }
            _ => {
                println!("Meta-command not recognized: {}", cmd);
            }
        },
        Command::Statement(sql) => {
            let parts: Vec<&str> = sql.split_whitespace().collect();
            if parts.is_empty() {
                return;
            }

            match parts[0].to_lowercase().as_str() {
                "insert" => {
                    if parts.len() - 1 != table.schema.columns.len() {
                        println!(
                            "Error: Expected {} values, but got {}",
                            table.schema.columns.len(),
                            parts.len() - 1
                        );
                        return;
                    }

                    let mut values = Vec::new();
                    for (i, col) in table.schema.columns.iter().enumerate() {
                        let val_str = parts[i + 1];
                        let val = match col.data_type {
                            crate::types::DataType::Integer => {
                                Value::Integer(val_str.parse().unwrap_or(0))
                            }
                            crate::types::DataType::Varchar => Value::Varchar(val_str.to_string()),
                            crate::types::DataType::Boolean => {
                                Value::Boolean(val_str == "true" || val_str == "1")
                            }
                        };
                        values.push(val);
                    }

                    let row = Row::new(values);
                    table.insert(row);
                    println!("Executed insert.");
                }
                "select" => {
                    let rows = table.select_all();
                    if rows.is_empty() {
                        println!("Table is empty")
                    } else {
                        for row in rows {
                            println!("{:?}", row.values);
                        }
                    }
                }
                _ => {
                    println!("Unrecognized statement: '{}'", sql);
                }
            }
        }
        Command::Empty => {}
    }
}
