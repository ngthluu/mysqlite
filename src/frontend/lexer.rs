#[derive(Debug)]
pub enum Token {
    // Keywords
    Create,
    Table,
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,

    // Symbols
    Star,
    Comma,
    Semicolon,
    LParen,
    RParen,
    Eq,

    // Data
    Identifier(String),
    Integer(i64),
    StringLit(String),

    // Control
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos >= self.input.len() {
            None
        } else {
            Some(self.input[self.pos])
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            None
        } else {
            let c = self.input[self.pos];
            self.pos += 1;
            Some(c)
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.peek() {
            None => Token::Eof,
            Some(c) => match c {
                '(' => {
                    self.advance();
                    Token::LParen
                }
                ')' => {
                    self.advance();
                    Token::RParen
                }
                ',' => {
                    self.advance();
                    Token::Comma
                }
                ';' => {
                    self.advance();
                    Token::Semicolon
                }
                '*' => {
                    self.advance();
                    Token::Star
                }
                '=' => {
                    self.advance();
                    Token::Eq
                }
                '\'' => self.read_string(),
                '0'..='9' => self.read_number(),
                'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
                _ => {
                    // Skip unknown chars or panic
                    self.advance();
                    self.next_token() // Recursively try next
                }
            },
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match ident.to_uppercase().as_str() {
            "CREATE" => Token::Create,
            "TABLE" => Token::Table,
            "SELECT" => Token::Select,
            "FROM" => Token::From,
            "WHERE" => Token::Where,
            "INSERT" => Token::Insert,
            "INTO" => Token::Into,
            "VALUES" => Token::Values,
            "UPDATE" => Token::Update,
            "SET" => Token::Set,
            "DELETE" => Token::Delete,
            _ => Token::Identifier(ident),
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        while let Some(c) = self.peek() {
            if c.is_digit(10) {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        Token::Integer(num_str.parse().unwrap_or(0))
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // consume opening '
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '\'' {
                self.advance(); // consume closing '
                break;
            }
            s.push(c);
            self.advance();
        }
        Token::StringLit(s)
    }
}
