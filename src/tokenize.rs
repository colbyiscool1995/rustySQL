

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind<'a> {

    //Literals & Identifiers 
    Identifer(&'a str),
    StringLiteral(&'a str),
    Integer(i64),
    Float(f64),
    
    // Keywords
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    Create,
    Table,
    And,
    Or,
    Not,
    Null,
    True,
    False,

    // Single & Multi-character Operators
    Equal,        // =
    NotEqual,     // != or <>
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=
    Plus,         // +
    Minus,        // -
    Star,         // * (wildcard or multiply)
    Slash,        // /

    // Delimiters
    Comma,        // ,
    Semicolon,    // ;
    Dot,          // . (e.g., users.id)
    LParen,       // (
    RParen,       // )
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self{
        Self {
            source,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, String> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek() {
            
            let start_line = self.line;
            let start_col  = self.column;

            match c {

                //Skipping whitespace
                c if c.is_whitespace() => {

                    self.advance();
                    
                }

                //Skip single line comments (-- ...)
                '-' if self.peek_next() == Some('-') => {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }

                //Skip multi-line comments (/* ... */)
                '/' if self.peek_next() == Some('*') => {
                    self.advance();
                    self.advance();
                    let mut closed = false;
                    while let Some(ch) = self.peek() {
                        if ch == '*' && self.peek_next() == Some('/') {
                            self.advance();
                            self.advance();
                            closed = true;
                            break;
                        }
                        self.advance();
                    }
                    if !closed {
                        return Err(format!(
                            "Unterminated block column at {}:{}",
                            start_line,start_col
                        ));
                    }
                }
                
                //Comparison Operators
                '=' => {
                    self.advance();
                    tokens.push(self.make_token(TokenKind::Equal, start_line, start_col));
                }

                '!' if self.peek_next() == Some('=') => {
                    self.advance();
                    self.advance();
                    tokens.push(self.make_token(TokenKind::NotEqual, start_line, start_col));
                }

                '<' => {
                    self.advance();
                    match self.peek() {
                        Some('=') => {
                            self.advance();
                            tokens.push(self.make_token(TokenKind::LessEqual, start_line, start_col));
                        }
                        Some('>') => {
                            self.advance();
                            tokens.push(self.make_token(TokenKind::NotEqual, start_line, start_col));
                        }
                        _ => {
                            tokens.push(self.make_token(TokenKind::LessThan, start_line, start_col));
                        }
                    }
                }

                '>' => {
                    self.advance();
                    if self.peek() == Some('='){
                        self.advance();
                        tokens.push(self.make_token(TokenKind::GreaterEqual, start_line, start_col));
                    } else {
                        tokens.push(self.make_token(TokenKind::GreaterThan, start_line, start_col));
                    }
                }

                '+' => { self.advance(); tokens.push(self.make_token(TokenKind::Plus, start_line, start_col)); }
                '-' => { self.advance(); tokens.push(self.make_token(TokenKind::Minus, start_line, start_col)); }
                '*' => { self.advance(); tokens.push(self.make_token(TokenKind::Star, start_line, start_col)); }
                '/' => { self.advance(); tokens.push(self.make_token(TokenKind::Slash, start_line, start_col)); }
                ',' => { self.advance(); tokens.push(self.make_token(TokenKind::Comma, start_line, start_col)); }
                ';' => { self.advance(); tokens.push(self.make_token(TokenKind::Semicolon, start_line, start_col)); }
                '.' => { self.advance(); tokens.push(self.make_token(TokenKind::Dot, start_line, start_col)); }
                '(' => { self.advance(); tokens.push(self.make_token(TokenKind::LParen, start_line, start_col)); }
                ')' => { self.advance(); tokens.push(self.make_token(TokenKind::RParen, start_line, start_col)); }

                // SQL String literals ('...')
                '\'' => {
                    tokens.push(self.read_quoted('\'', true, start_line, start_col)?);
                }

                // Double-quoted SQL identifiers ("table_name")
                '"' => {
                    tokens.push(self.read_quoted('"', false, start_line, start_col)?);
                }

                // Numbers (Integers & Floats)
                '0'..='9' => {
                    tokens.push(self.read_number(start_line, start_col)?);
                }

                // Unquoted Identifiers & Keywords
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_identifier(start_line, start_col));
                }

                _ => {
                    return Err(format!(
                        "Unexpected character '{}' at {}:{}",
                        c, start_line, start_col
                    ));
                }
            }
            
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut iter = self.source[self.pos..].chars();
        iter.next();
        iter.next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column += 1;
        } else {
            self.column += 1;
        }

        Some(c)
    }

    fn make_token(&self, kind: TokenKind<'a>, 
    line: usize, column: usize) -> Token<'a>{
        Token { kind, line, column}
    }

    fn read_identifier(&mut self, line: usize, column: usize) -> Token<'a>{
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let text = &self.source[start..self.pos];
        let kind = match text {
            "insert"    => TokenKind::Insert,
            "into"      => TokenKind::Into,
            "select"    => TokenKind::Select,
            "values"    => TokenKind::Values,
            "from"      => TokenKind::From,
            "create"    => TokenKind::Create,
            "table"     => TokenKind::Table,
            "update"    => TokenKind::Update,
            "set"       => TokenKind::Set,
            "and"       => TokenKind::And,
            "or"        => TokenKind::Or,
            "not"       => TokenKind::Not,
            "null"      => TokenKind::Null,
            "true"      => TokenKind::True,
            "false"     => TokenKind::False,

            _ => TokenKind::Identifer(text),
        };
        self.make_token(kind, line, column)
    }

    //checks if number is float or int
    fn read_number(&mut self, line: usize, column: usize) -> Result<Token<'a>, String>{
        let start = self.pos;
        let mut is_float = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            
            }
            else if c == '.' {
                if is_float {
                    Err(format!(
                        "Incorrect Float format at {}:{}",
                        line, column
                    ))?
                }
                is_float = true;
                self.advance();
            }
             else {
                break;
            }
        }

        if is_float {
            let num = self.source[start..self.pos]
            .parse::<f64>()
            .map_err(|_| format!("Invalid float at {}:{}", line, column))?;
            Ok(self.make_token(TokenKind::Float(num), line, column))
        } else {
            let num = self.source[start..self.pos]
            .parse::<i64>()
            .map_err(|_| format!("Invalid integer at {}:{}", line, column))?;
            Ok(self.make_token(TokenKind::Integer(num), line, column))
        }
        
        
    }

    // fn read_string(&mut self, line: usize, column: usize) -> Result<Token<'a>, String>{
    //     self.advance();
    //     let start = self.pos;
    //     while let Some(c) = self.peek() {
    //         if c == '"' {
    //             let text = &self.source[start..self.pos];
    //             self.advance();
    //             return Ok(self.make_token(TokenKind::StringLiteral(text), line, column, ));
    //         }
    //         self.advance();
    //     }
    //     Err(format!("Unterminated string literal starting at {}:{}", line, column))
    // }

    fn read_quoted(
        &mut self,
        quote: char,
        is_string_literal: bool,
        line: usize,
        column: usize,
    ) -> Result<Token<'a>, String> {
        self.advance(); // consume opening quote
        let start = self.pos;

        while let Some(c) = self.peek() {
            if c == quote {
                let text = &self.source[start..self.pos];
                self.advance(); // consume closing quote
                let kind = if is_string_literal {
                    TokenKind::StringLiteral(text)
                } else {
                    TokenKind::Identifer(text)
                };
                return Ok(self.make_token(kind, line, column));
            }
            self.advance();
        }

        Err(format!("Unterminated {} starting at {}:{}", quote, line, column))
    }
}



