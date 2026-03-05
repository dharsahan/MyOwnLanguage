use crate::ast::{BinaryOperator, Expr, Stmt};
use crate::lexer::TokenType;

pub struct Parser {
    tokens: Vec<TokenType>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenType>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
    fn current(&self) -> Option<&TokenType> {
        self.tokens.get(self.position)
    }
    fn advance(&mut self) {
        self.position += 1;
    }
    pub fn parse_expression(&mut self) -> Result<Expr, String> {
        let left = self.parse_primary()?;
        self.parse_binary_op(left, 0)
    }
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Some(TokenType::Number(n)) => {
                let val = *n;
                self.advance();
                Ok(Expr::Number(val))
            }
            Some(TokenType::Identifier(name)) => {
                let var_name = name.clone();
                self.advance();
                Ok(Expr::Variable(var_name))
            }
            Some(TokenType::StringLiteral(s)) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::StringLiteral(val))
            }
            Some(TokenType::CharLiteral(c)) => {
                let val = *c;
                self.advance();
                Ok(Expr::CharLiteral(val))
            }
            Some(TokenType::True) => {
                self.advance();
                Ok(Expr::Boolean(true))
            }
            Some(TokenType::False) => {
                self.advance();
                Ok(Expr::Boolean(false))
            }
            Some(TokenType::LeftParen) => {
                self.advance(); // consume '('
                let expr = self.parse_expression()?;
                match self.current() {
                    Some(TokenType::RightParen) => {
                        self.advance(); // consume ')'
                        Ok(expr)
                    }
                    _ => Err("Expected closing ')'".to_string()),
                }
            }
            _ => Err("Expected a primary expression (number, identifier, string, char, boolean, or parenthesized expression)".to_string()),
        }
    }
    fn parse_binary_op(&mut self, left: Expr, min_prec: u8) -> Result<Expr, String> {
        let mut left = left;
        while let Some(op) = self.current() {
            let prec = match Self::get_precedence(op) {
                Some(p) => p,
                Option::None => break,
            };
            if prec < min_prec {
                break;
            }
            let op = match op {
                TokenType::Plus => BinaryOperator::Add,
                TokenType::Minus => BinaryOperator::Subtract,
                TokenType::Star => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                TokenType::EqualEqual => BinaryOperator::EqualEqual,
                TokenType::NotEqual => BinaryOperator::NotEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => return Err("Expected a binary operator".to_string()),
            };
            self.advance();
            let mut right = self.parse_primary()?;
            while let Some(next_op) = self.current() {
                let next_prec = match Self::get_precedence(next_op) {
                    Some(p) => p,
                    Option::None => break,
                };
                if next_prec > prec {
                    right = self.parse_binary_op(right, next_prec)?;
                } else {
                    break;
                }
            }
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    fn get_precedence(op: &TokenType) -> Option<u8> {
        match op {
            TokenType::EqualEqual | TokenType::NotEqual => Some(0),
            TokenType::Less | TokenType::Greater | TokenType::LessEqual | TokenType::GreaterEqual => Some(1),
            TokenType::Plus | TokenType::Minus => Some(2),
            TokenType::Star | TokenType::Slash => Some(3),
            _ => None,
        }
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current() {
            Some(TokenType::Let) => {
                self.advance(); 
                
                let name = match self.current() {
                    Some(TokenType::Identifier(n)) => n.clone(),
                    _ => return Err("Expected variable name".to_string()),
                };
                self.advance();

                match self.current() {
                    Some(TokenType::Equal) => self.advance(), 
                    _ => return Err("Expected '='".to_string()),
                }

                let expr = self.parse_expression()?;
                Ok(Stmt::Let(name, expr))
            }
            Some(TokenType::Print) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Stmt::Print(expr))
            }
            Some(TokenType::If) => self.parse_if(),
            _ => {
                let expr = self.parse_expression()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        match self.current() {
            Some(TokenType::LeftBrace) => self.advance(),
            _ => return Err("Expected '{'".to_string()),
        }
        let mut stmts = Vec::new();
        while self.position < self.tokens.len() {
            if let Some(TokenType::RightBrace) = self.current() {
                self.advance();
                return Ok(stmts);
            }
            stmts.push(self.parse_statement()?);
        }
        Err("Expected '}'".to_string())
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); 
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        let else_branch = if let Some(TokenType::Else) = self.current() {
            self.advance(); 
            if let Some(TokenType::If) = self.current() {
                
                let nested_if = self.parse_if()?;
                Some(vec![nested_if])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while self.position < self.tokens.len() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }
}
