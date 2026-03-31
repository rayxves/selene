use crate::{
    expr::Expression,
    token::{BinaryOp, Token, TokenLiteral, TokenType, UnaryOp},
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

impl ParseError {
    pub fn new(token: Token, message: String) -> ParseError {
        ParseError { token, message }
    }
}
pub struct Parser {
    tokens: Vec<Token>,
    current_pos: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current_pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.tokens[self.current_pos].token_type == TokenType::EOF
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.current_pos]
    }

    pub fn advance(&mut self) -> &Token {
        let pos = self.current_pos;
        self.current_pos += 1;
        &self.tokens[pos]
    }

    pub fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        match (&self.peek().token_type, token_type) {
            (TokenType::Identifier(_), TokenType::Identifier(_)) => true,
            (TokenType::Number(_), TokenType::Number(_)) => true,
            (a, b) => a == b,
        }
    }

    pub fn primary(&mut self) -> Result<Expression, ParseError> {
        match &self.peek().token_type {
            TokenType::Number(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Literal(TokenLiteral::Number(n)))
            }
            TokenType::StringLiteral(n) => {
                let n = n.to_string();
                self.advance();
                Ok(Expression::Literal(TokenLiteral::StringLiteral(n)))
            }
            TokenType::Boolean(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Literal(TokenLiteral::Boolean(n)))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expression::Literal(TokenLiteral::Null))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                match self.peek().token_type {
                    TokenType::RightParen => {
                        self.advance();
                        Ok(Expression::Grouping(Box::new(expr)))
                    }
                    _ => {
                        let error = ParseError::new(
                            self.peek().clone(),
                            "Esperava ')' após expressão.".to_string(),
                        );
                        self.errors.push(error.clone());
                        Err(error)
                    }
                }
            }
            _ => {
                let error = ParseError::new(
                    self.peek().clone(),
                    format!(
                        "Token inesperado '{}' na linha {}.",
                        self.peek().lexeme,
                        self.peek().line
                    ),
                );
                self.errors.push(error.clone());
                Err(error)
            }
        }
    }

    pub fn unary(&mut self) -> Result<Expression, ParseError> {
        match &self.peek().token_type {
            TokenType::Bang => {
                self.advance();
                let expr = self.unary()?;
                Ok(Expression::Unary(UnaryOp::Bang, Box::new(expr)))
            }
            TokenType::Minus => {
                self.advance();
                let expr = self.unary()?;
                Ok(Expression::Unary(UnaryOp::Minus, Box::new(expr)))
            }
            _ => self.primary(),
        }
    }

    pub fn factor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.unary()?;
        while self.check(&TokenType::Slash) || self.check(&TokenType::Star) {
            let operator = match self.peek().token_type {
                TokenType::Slash => BinaryOp::Slash,
                TokenType::Star => BinaryOp::Star,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.unary()?;
            left = Expression::Binary(Box::new(left), operator, Box::new(right));
        }
        Ok(left)
    }

    pub fn term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.factor()?;
        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let operator = match self.peek().token_type {
                TokenType::Plus => BinaryOp::Plus,
                TokenType::Minus => BinaryOp::Minus,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.factor()?;
            left = Expression::Binary(Box::new(left), operator, Box::new(right));
        }
        Ok(left)
    }

    pub fn comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.term()?;
        while self.check(&TokenType::Greater)
            || self.check(&TokenType::Less)
            || self.check(&TokenType::GreaterEqual)
            || self.check(&TokenType::LessEqual)
        {
            let operator = match self.peek().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.term()?;
            left = Expression::Binary(Box::new(left), operator, Box::new(right));
        }
        Ok(left)
    }

    pub fn equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.comparison()?;
        while self.check(&TokenType::EqualEqual) || self.check(&TokenType::BangEqual) {
            let operator = match self.peek().token_type {
                TokenType::EqualEqual => BinaryOp::EqualEqual,
                TokenType::BangEqual => BinaryOp::BangEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.comparison()?;
            left = Expression::Binary(Box::new(left), operator, Box::new(right));
        }
        Ok(left)
    }

    pub fn expression(&mut self) -> Result<Expression, ParseError> {
        self.equality()
    }

    fn synchronize(&mut self) {
    self.advance();
    while !self.is_at_end() {
        match self.peek().token_type {
            TokenType::If | TokenType::While | TokenType::For |
            TokenType::Return | TokenType::Var | TokenType::Print => return,
            _ => { self.advance(); }
        }
    }
}
}
