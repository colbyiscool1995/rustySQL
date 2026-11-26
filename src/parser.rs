use crate::tokenize::{Token, TokenKind::{self}};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'a>{
    Select(SelectStatement<'a>),
    Insert(InsertStatement<'a>),
    Delete(DeleteStatement<'a>,),
    CreateTable(CreateTableStatement<'a>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct SelectStatement<'a> {
    pub columns: Vec<SelectItem<'a>>,
    pub table: &'a str,
    pub where_clause: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectItem<'a> {
    Wildcard,
    Column(&'a str),
    Expr(Expr<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct InsertStatement<'a> {
    pub table: &'a str,
    pub columns: Vec<&'a str>,
    pub values: Vec<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DeleteStatement<'a> {
    pub table: &'a str,
    pub where_clause: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CreateTableStatement<'a>{
    pub table : &'a str,


}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    Identifer(&'a str),
    Integer(i64),
    Float(f64),
    StringLiteral(&'a str),
    Boolean(bool),
    Null,
    Unary {
        op: UnaryOp,
        expr: Box<Expr<'a>>
    },
    Binary {
        left: Box<Expr<'a>>,
        op: BinaryOp,
        right: Box<Expr<'a>>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryOp {
    Not,
    Minus,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOp {
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Or,
    Plus,
    Minus,
    Multiply,
    Divide,
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self{
        Self { tokens, pos: 0 }
    }

    pub fn parse_statements(&mut self) -> Result<Vec<Statement<'a>>, String> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
            if self.match_kind(&TokenKind::Semicolon){
                continue;
            }
        }

        Ok(statements)
    }
 
    fn parse_statement(&mut self) -> Result<Statement<'a>, String> {
        let token = self.peek().ok_or("Unexpected end of input")?;

        match token.kind {
            TokenKind::Select => Ok(Statement::Select(self.parse_select()?)),
            TokenKind::Insert => Ok(Statement::Insert(self.parse_insert()?)),
            TokenKind::Delete => Ok(Statement::Delete(self.parse_delete()?)),
            _ => Err(format!(
                "Expected SQL statement at {}:{}, found {:?}",
                token.line, token.column, token.kind
            )),
        }
    }

    // Statement Parsers

    fn parse_select(&mut self) -> Result<SelectStatement<'a>, String> {
        self.expect(&TokenKind::Select)?;

        // Parse column list: Select * or Select col1, col2
    
        let mut columns = Vec::new();
        if self.match_kind(&TokenKind::Star) {
            columns.push(SelectItem::Wildcard);
        } else {
            loop {
                let expr = self.parse_expr(0)?;
                match expr {
                    Expr::Identifer(name) => columns.push(SelectItem::Column(name)),
                    other => columns.push(SelectItem::Expr(other))
                }
                if !self.match_kind(&TokenKind::Comma){
                    break;
                }
            }
        }

        // parse FROM <table>
        self.expect(&TokenKind::From)?;
        let table = self.expect_identifer()?;


        // parse WHERE <expr>
        let where_clause = if self.match_kind(&TokenKind::Where) {
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        Ok(SelectStatement { 
            columns, 
            table, 
            where_clause 
        })
    }

    fn parse_insert(&mut self) -> Result<InsertStatement<'a>, String> {
        self.expect(&TokenKind::Insert)?;
        self.expect(&TokenKind::Into)?;
        let table = self.expect_identifer()?;

        let mut columns = Vec::new();

        //optional column list: (col1,col2)
        if self.match_kind(&TokenKind::LParen) {
            loop {
                columns.push(self.expect_identifer()?);
                if !self.match_kind(&TokenKind::Comma){
                    break;
                }
            }
            self.expect(&TokenKind::RParen)?;
        }

        self.expect(&TokenKind::Values)?;
        self.expect(&TokenKind::LParen)?;

        let mut values = Vec::new();
        loop {
            values.push(self.parse_expr(0)?);
            if !self.match_kind(&TokenKind::Comma){
                break;
            }
        }
        self.expect(&TokenKind::RParen)?;

        Ok(InsertStatement { 
            table, 
            columns, 
            values 
        })
    }

    fn parse_delete(&mut self) -> Result<DeleteStatement<'a>, String> {
        self.expect(&TokenKind::Delete)?;
        self.expect(&TokenKind::From)?;
        let table = self.expect_identifer()?;

        let where_clause = if self.match_kind(&TokenKind::Where) {
            Some(self.parse_expr(0)?)
        } else { 
            None
        };

        Ok(DeleteStatement { 
            table, 
            where_clause 
        })
    }

    // Pratt Expression Parser 

    fn parse_expr(&mut self, min_precedence: u8) -> Result<Expr<'a>, String> {
        let mut left = self.parse_prefix()?;
        
        while let Some(token) = self.peek() {
            let Some((op,precedence)) = self.infix_binding_power(&token.kind) else {
                break;
            };

            if precedence < min_precedence {
                break;
            }

            self.advance();

            let right = self.parse_expr(precedence + 1)?;
            left = Expr::Binary { 
                left: Box::new(left), 
                op, 
                right: Box::new(right), 
            };
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr<'a>, String> {
        let token = self.advance().ok_or("Expected expression, found end of input")?;

        match token.kind {
            TokenKind::Identifer(name)      => Ok(Expr::Identifer(name)),
            TokenKind::Integer(n)           => Ok(Expr::Integer(n)),
            TokenKind::Float(f)             => Ok(Expr::Float(f)),
            TokenKind::StringLiteral(s)     => Ok(Expr::StringLiteral(s)),
            TokenKind::True                 => Ok(Expr::Boolean(true)),
            TokenKind::False                => Ok(Expr::Boolean(false)),
            TokenKind::Null                 => Ok(Expr::Null),

            // Unary operators: Not x, -5
            TokenKind::Not => {
                let expr = self.parse_expr(6)?;
                Ok(Expr::Unary {
                    op:UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }

            TokenKind::Minus => {
                let expr = self.parse_expr(6)?;
                Ok(Expr::Unary { 
                    op: UnaryOp::Minus,
                    expr: Box::new(expr),
                })
            }

            TokenKind::LParen => {
                let expr = self.parse_expr(0)?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }

            _ => Err(format!(
                "Unexpected token {:?} in expression at {}:{}",
                token.kind, token.line, token.column
            )),
        }
    }

    fn infix_binding_power(&self, kind: &TokenKind<'a>) -> Option<(BinaryOp, u8)> {
        match kind {
            TokenKind::Or           => Some((BinaryOp::Or, 1)),
            TokenKind::And          => Some((BinaryOp::And, 2)),
            TokenKind::Equal        => Some((BinaryOp::Equal, 3)),
            TokenKind::NotEqual     => Some((BinaryOp::NotEqual, 3)),
            TokenKind::LessThan     => Some((BinaryOp::LessThan, 3)),
            TokenKind::LessEqual    => Some((BinaryOp::LessEqual, 3)),
            TokenKind::GreaterThan  => Some((BinaryOp::GreaterThan, 3)),
            TokenKind::GreaterEqual => Some((BinaryOp::GreaterEqual, 3)),
            TokenKind::Plus         => Some((BinaryOp::Plus, 4)),
            TokenKind::Minus        => Some((BinaryOp::Minus, 4)),
            TokenKind::Star         => Some((BinaryOp::Multiply, 4)),
            TokenKind::Slash        => Some((BinaryOp::Divide, 5)),
            _ => None,
        }
    }

    // Cursor Helpers

    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token<'a>> {
        if self.pos < self.tokens.len(){
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn match_kind(&mut self, expected: &TokenKind<'a>) -> bool{
        if let Some(tok) = self.peek() {
            if &tok.kind == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expect(&mut self, expected: &TokenKind<'a>) -> Result<Token<'a>, String> {
        match self.peek() {
            Some(tok) if &tok.kind == expected => Ok(self.advance().unwrap()),
            Some(tok) => Err(format!(
                "Expected {:?}, found {:?} at {} {}",
                expected, tok.kind, tok.line, tok.column
            )),
            None => Err(format!("Expected {:?}, found end of imput", expected))
        }
    }

    fn expect_identifer(&mut self) -> Result<&'a str, String> {
        match self.peek() {
            Some(tok) => match tok.kind {
                TokenKind::Identifer(name) => {
                    self.advance();
                    Ok(name)
                }
                _ => Err(format!(
                    "Expected identifier, found {:?} at {}:{}",
                    tok.kind, tok.line, tok.column
                )),
                
            }
            None => Err("Expected Identifer, found end of input".to_string()),
        }
    }
}