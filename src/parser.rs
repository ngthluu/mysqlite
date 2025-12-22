use crate::{
    ast::{ColumnDef, Expression, Statement},
    lexer::{Lexer, Token},
};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?}",
                expected, self.current_token
            ))
        }
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match &self.current_token {
            Token::Identifier(s) => {
                let name = s.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(format!(
                "Expected Identifier, found {:?}",
                self.current_token
            )),
        }
    }

    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token {
            Token::Create => self.parse_create_table(),
            Token::Select => self.parse_select(),
            Token::Insert => self.parse_insert(),
            Token::Update => self.parse_update(),
            Token::Delete => self.parse_delete(),
            _ => Err(format!("Unexpected token: {:?}", self.current_token)),
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement, String> {
        self.advance(); // consume CREATE
        self.expect(Token::Table)?;
        let name = self.parse_identifier()?;
        self.expect(Token::LParen)?;

        let mut columns = Vec::new();
        loop {
            let col_name = self.parse_identifier()?;
            let col_type = self.parse_identifier()?;
            columns.push(ColumnDef {
                name: col_name,
                col_type,
            });

            if let Token::Comma = self.current_token {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(Token::RParen)?;
        self.expect(Token::Semicolon)?;

        Ok(Statement::CreateTable { name, columns })
    }

    fn parse_select(&mut self) -> Result<Statement, String> {
        self.advance(); // consume SELECT

        let mut columns = Vec::new();
        if let Token::Star = self.current_token {
            columns.push("*".to_string());
            self.advance();
        } else {
            loop {
                columns.push(self.parse_identifier()?);
                if let Token::Comma = self.current_token {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(Token::From)?;
        let table = self.parse_identifier()?;

        let mut filter = None;
        if let Token::Where = self.current_token {
            self.advance();
            filter = Some(self.parse_expression()?);
        }

        self.expect(Token::Semicolon)?;

        Ok(Statement::Select {
            columns,
            from: table,
            filter,
        })
    }

    fn parse_insert(&mut self) -> Result<Statement, String> {
        self.advance(); // consume INSERT
        self.expect(Token::Into)?;
        let table = self.parse_identifier()?;

        // Optional: (col1, col2)
        let mut columns = Vec::new();
        if let Token::LParen = self.current_token {
            self.advance();
            loop {
                columns.push(self.parse_identifier()?);
                if let Token::Comma = self.current_token {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        }

        self.expect(Token::Values)?;
        self.expect(Token::LParen)?;

        let mut values = Vec::new();
        loop {
            values.push(self.parse_expression()?);
            if let Token::Comma = self.current_token {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(Token::RParen)?;
        self.expect(Token::Semicolon)?;

        Ok(Statement::Insert {
            table,
            columns,
            values,
        })
    }

    fn parse_update(&mut self) -> Result<Statement, String> {
        self.advance(); // consume UPDATE
        let table = self.parse_identifier()?;
        self.expect(Token::Set)?;

        let mut updates = Vec::new();
        loop {
            let col = self.parse_identifier()?;
            self.expect(Token::Eq)?;
            let val = self.parse_expression()?;
            updates.push((col, val));

            if let Token::Comma = self.current_token {
                self.advance();
            } else {
                break;
            }
        }

        let mut filter = None;
        if let Token::Where = self.current_token {
            self.advance();
            filter = Some(self.parse_expression()?);
        }

        self.expect(Token::Semicolon)?;
        Ok(Statement::Update {
            table,
            updates,
            filter,
        })
    }

    fn parse_delete(&mut self) -> Result<Statement, String> {
        self.advance(); // consume DELETE
        self.expect(Token::From)?;
        let table = self.parse_identifier()?;

        let mut filter = None;
        if let Token::Where = self.current_token {
            self.advance();
            filter = Some(self.parse_expression()?);
        }

        self.expect(Token::Semicolon)?;
        Ok(Statement::Delete { table, filter })
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let left = match &self.current_token {
            Token::Identifier(s) => Expression::Identifier(s.clone()),
            Token::Integer(i) => Expression::Integer(*i),
            Token::StringLit(s) => Expression::Literal(s.clone()),
            _ => {
                return Err(format!(
                    "Expected expression, found {:?}",
                    self.current_token
                ));
            }
        };
        self.advance();

        // Check if it's a binary operation (like x = y)
        if let Token::Eq = self.current_token {
            self.advance();
            let right = match &self.current_token {
                Token::Identifier(s) => Expression::Identifier(s.clone()),
                Token::Integer(i) => Expression::Integer(*i),
                Token::StringLit(s) => Expression::Literal(s.clone()),
                _ => {
                    return Err(format!(
                        "Expected right side of expression, found {:?}",
                        self.current_token
                    ));
                }
            };
            self.advance();
            return Ok(Expression::BinaryOp {
                left: Box::new(left),
                op: "=".to_string(),
                right: Box::new(right),
            });
        }

        Ok(left)
    }
}
