use crate::ast::{BinaryOperator, Expr, Stmt};
use crate::error::LangError;
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

    fn peek(&self, offset: usize) -> Option<&TokenType> {
        self.tokens.get(self.position + offset)
    }

    fn has_tokens(&self) -> bool {
        self.position < self.tokens.len()
    }

    pub fn parse_expression(&mut self) -> Result<Expr, LangError> {
        let left = self.parse_primary()?;
        self.parse_binary_op(left, 0)
    }

    fn parse_primary(&mut self) -> Result<Expr, LangError> {
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
                self.advance();
                let expr = self.parse_expression()?;
                match self.current() {
                    Some(TokenType::RightParen) => {
                        self.advance();
                        Ok(expr)
                    }
                    _ => Err(LangError::parse("Expected closing ')'")),
                }
            }
            _ => Err(LangError::parse(
                "Expected expression (number, identifier, string, char, boolean, or '(')",
            )),
        }
    }

    fn parse_binary_op(&mut self, left: Expr, min_prec: u8) -> Result<Expr, LangError> {
        let mut left = left;

        while let Some(prec) = self.current().and_then(Self::get_precedence) {
            if prec < min_prec {
                break;
            }

            let op = self.token_to_binop()?;
            self.advance();

            let mut right = self.parse_primary()?;

            while let Some(next_prec) = self.current().and_then(Self::get_precedence) {
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

    fn token_to_binop(&self) -> Result<BinaryOperator, LangError> {
        match self.current() {
            Some(TokenType::Plus) => Ok(BinaryOperator::Add),
            Some(TokenType::Minus) => Ok(BinaryOperator::Subtract),
            Some(TokenType::Star) => Ok(BinaryOperator::Multiply),
            Some(TokenType::Slash) => Ok(BinaryOperator::Divide),
            Some(TokenType::EqualEqual) => Ok(BinaryOperator::EqualEqual),
            Some(TokenType::NotEqual) => Ok(BinaryOperator::NotEqual),
            Some(TokenType::Less) => Ok(BinaryOperator::Less),
            Some(TokenType::Greater) => Ok(BinaryOperator::Greater),
            Some(TokenType::LessEqual) => Ok(BinaryOperator::LessEqual),
            Some(TokenType::GreaterEqual) => Ok(BinaryOperator::GreaterEqual),
            _ => Err(LangError::parse("Expected a binary operator")),
        }
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


    pub fn parse_statement(&mut self) -> Result<Stmt, LangError> {
        match self.current() {
            Some(TokenType::Let) => self.parse_let(),
            Some(TokenType::Print) => self.parse_print(),
            Some(TokenType::If) => self.parse_if(),
            Some(TokenType::While) => self.parse_while(),
            Some(TokenType::For) => self.parse_for(),
            Some(TokenType::Break) => {
                self.advance();
                Ok(Stmt::Break)
            }
            Some(TokenType::Continue) => {
                self.advance();
                Ok(Stmt::Continue)
            }
            Some(TokenType::Identifier(_)) => self.parse_assignment_or_expr(),
            _ => {
                let expr = self.parse_expression()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, LangError> {
        self.advance();

        let name = match self.current() {
            Some(TokenType::Identifier(n)) => n.clone(),
            _ => return Err(LangError::parse("Expected variable name after 'declare'")),
        };
        self.advance();

        match self.current() {
            Some(TokenType::Equal) => self.advance(),
            _ => return Err(LangError::parse("Expected '=' in declaration")),
        }

        let expr = self.parse_expression()?;
        Ok(Stmt::Let(name, expr))
    }

    fn parse_print(&mut self) -> Result<Stmt, LangError> {
        self.advance();
        let expr = self.parse_expression()?;
        Ok(Stmt::Print(expr))
    }

    fn parse_assignment_or_expr(&mut self) -> Result<Stmt, LangError> {
        let name = match self.current() {
            Some(TokenType::Identifier(n)) => n.clone(),
            _ => unreachable!("called parse_assignment_or_expr without Identifier"),
        };

        if matches!(self.peek(1), Some(TokenType::Equal)) {
            self.advance();
            self.advance();
            let expr = self.parse_expression()?;
            return Ok(Stmt::Assign(name, expr));
        }

        let expr = self.parse_expression()?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, LangError> {
        match self.current() {
            Some(TokenType::LeftBrace) => self.advance(),
            _ => return Err(LangError::parse("Expected '{'")),
        }

        let mut stmts = Vec::new();
        while self.has_tokens() {
            if let Some(TokenType::RightBrace) = self.current() {
                self.advance();
                return Ok(stmts);
            }
            stmts.push(self.parse_statement()?);
        }

        Err(LangError::parse("Expected '}'"))
    }

    fn parse_if(&mut self) -> Result<Stmt, LangError> {
        self.advance();
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        let else_branch = if matches!(self.current(), Some(TokenType::Else)) {
            self.advance();
            if matches!(self.current(), Some(TokenType::If)) {
                Some(vec![self.parse_if()?])
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

    fn parse_while(&mut self) -> Result<Stmt, LangError> {
        self.advance();
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, LangError> {
        self.advance();

        let variable = match self.current() {
            Some(TokenType::Identifier(name)) => name.clone(),
            _ => return Err(LangError::parse("Expected variable name after 'for'")),
        };
        self.advance();

        match self.current() {
            Some(TokenType::In) => self.advance(),
            _ => return Err(LangError::parse("Expected 'in' after variable in for loop")),
        }

        let start = self.parse_expression()?;

        match self.current() {
            Some(TokenType::DotDot) => self.advance(),
            _ => return Err(LangError::parse("Expected '..' in for loop range")),
        }

        let end = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt::For {
            variable,
            start,
            end,
            body,
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, LangError> {
        let mut statements = Vec::new();
        while self.has_tokens() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }
}
