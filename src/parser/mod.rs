mod statements;
use crate::{
    expr::{Expression, next_id},
    stmt::Statement,
    token::{BinaryOp, LogicalOp, Token, TokenLiteral, TokenType, UnaryOp},
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
            TokenType::Number(value) => {
                let value = *value;
                self.advance();
                Ok(Expression::Literal(TokenLiteral::Number(value)))
            }
            TokenType::StringLiteral(value) => {
                let value = value.to_string();
                self.advance();
                Ok(Expression::Literal(TokenLiteral::StringLiteral(value)))
            }
            TokenType::Boolean(value) => {
                let value = *value;
                self.advance();
                Ok(Expression::Literal(TokenLiteral::Boolean(value)))
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
                            format!(
                                "Esperava ')' após expressão, mas encontrei '{}'.",
                                self.peek().lexeme
                            ),
                        );
                        self.errors.push(error.clone());
                        Err(error)
                    }
                }
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                let line = self.peek().line;
                self.advance();
                let id = next_id();
                Ok(Expression::Variable(name, line, id))
            }
            TokenType::This => {
                let token = self.peek().clone();
                self.advance();
                let id = next_id();
                Ok(Expression::This(token, id))
            }
            TokenType::Super => {
                let key_super = self.peek().clone();
                self.advance();
                match self.peek().token_type {
                    TokenType::Dot => {
                        self.advance();
                        match &self.peek().token_type {
                            TokenType::Identifier(_) => {
                                let method = self.peek().clone();
                                self.advance();
                                let id = next_id();
                                Ok(Expression::Super(key_super, method, id))
                            }
                            _ => {
                                let error = ParseError::new(
                                    self.peek().clone(),
                                    format!(
                                        "Esperava o nome de um método, mas encontrei '{}'.",
                                        self.peek().lexeme
                                    ),
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
                                "Esperava '.' após uso de super, mas encontrei '{}'.",
                                self.peek().lexeme
                            ),
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

    pub fn call(&mut self) -> Result<Expression, ParseError> {
        let mut callee = self.primary()?;
        while self.check(&TokenType::LeftParen) || self.check(&TokenType::Dot) {
            if self.check(&TokenType::LeftParen) {
                let mut args = Vec::new();
                self.advance();
                while !self.check(&TokenType::RightParen) {
                    if args.len() > 255 {
                        self.error("Não é possível ter mais de 255 argumentos.".to_string());
                    }
                    args.push(self.expression()?);
                    if !self.check(&TokenType::Comma) {
                        break;
                    }
                    self.advance();
                }
                if !self.check(&TokenType::RightParen) {
                    return Err(self.error(format!(
                        "Esperava ')' após argumentos, mas encontrei '{}'.",
                        self.peek().lexeme
                    )));
                }
                let paren = self.peek().clone();
                self.advance();
                callee = Expression::Call(Box::new(callee), args, paren);
            } else if self.check(&TokenType::Dot) {
                self.advance();
                match &self.peek().token_type {
                    TokenType::Identifier(_t) => {
                        callee = Expression::Get(Box::new(callee), self.peek().clone());
                        self.advance();
                    }
                    _ => {
                        return Err(self.error(format!(
                            "Esperava nome de propriedade após '.', mas encontrei '{}'.",
                            self.peek().lexeme
                        )));
                    }
                }
            } else {
                break;
            }
        }
        Ok(callee)
    }
    pub fn unary(&mut self) -> Result<Expression, ParseError> {
        let line = self.peek().line;
        match &self.peek().token_type {
            TokenType::Bang => {
                self.advance();
                let operand = self.unary()?;
                Ok(Expression::Unary(UnaryOp::Bang, line, Box::new(operand)))
            }
            TokenType::Minus => {
                self.advance();
                let operand = self.unary()?;
                Ok(Expression::Unary(UnaryOp::Minus, line, Box::new(operand)))
            }
            _ => self.call(),
        }
    }

    pub fn factor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.unary()?;
        while self.check(&TokenType::Slash) || self.check(&TokenType::Star) {
            let line = self.peek().line;
            let operator = match self.peek().token_type {
                TokenType::Slash => BinaryOp::Slash,
                TokenType::Star => BinaryOp::Star,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.unary()?;
            left = Expression::Binary(Box::new(left), operator, line, Box::new(right));
        }
        Ok(left)
    }

    pub fn term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.factor()?;
        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let line = self.peek().line;
            let operator = match self.peek().token_type {
                TokenType::Plus => BinaryOp::Plus,
                TokenType::Minus => BinaryOp::Minus,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.factor()?;
            left = Expression::Binary(Box::new(left), operator, line, Box::new(right));
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
            let line = self.peek().line;
            let operator = match self.peek().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.term()?;
            left = Expression::Binary(Box::new(left), operator, line, Box::new(right));
        }
        Ok(left)
    }

    pub fn equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.comparison()?;
        while self.check(&TokenType::EqualEqual) || self.check(&TokenType::BangEqual) {
            let line = self.peek().line;
            let operator = match self.peek().token_type {
                TokenType::EqualEqual => BinaryOp::EqualEqual,
                TokenType::BangEqual => BinaryOp::BangEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.comparison()?;
            left = Expression::Binary(Box::new(left), operator, line, Box::new(right));
        }
        Ok(left)
    }

    pub fn and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.equality()?;
        while self.check(&TokenType::And) {
            let line = self.peek().line;
            self.advance();
            let right = self.equality()?;
            left = Expression::Logical(Box::new(left), LogicalOp::And, line, Box::new(right));
        }
        Ok(left)
    }

    pub fn or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.and()?;
        while self.check(&TokenType::Or) {
            let line = self.peek().line;
            self.advance();
            let right = self.and()?;
            left = Expression::Logical(Box::new(left), LogicalOp::Or, line, Box::new(right));
        }
        Ok(left)
    }

    pub fn assigment(&mut self) -> Result<Expression, ParseError> {
        let expr = self.or()?;
        if self.check(&TokenType::Equal) {
            self.advance();
            match expr {
                Expression::Variable(name, line, _id) => {
                    let assign_id = next_id();
                    let value = self.assigment()?;
                    return Ok(Expression::Assign(name, line, Box::new(value), assign_id));
                }
                Expression::Get(expr, token) => {
                    let value = self.assigment()?;
                    return Ok(Expression::Set(expr, token, Box::new(value)));
                }
                _ => {
                    let error = ParseError::new(
                        self.peek().clone(),
                        "Alvo de atribuição inválido.".to_string(),
                    );
                    self.errors.push(error.clone());
                    return Err(error);
                }
            }
        }
        Ok(expr)
    }

    pub fn expression(&mut self) -> Result<Expression, ParseError> {
        self.assigment()
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.statement() {
                statements.push(stmt);
            }
        }
        statements
    }

    fn synchronize(&mut self) {
        if self.is_at_end() {
            return;
        }
        self.advance();
        while !self.is_at_end() {
            match self.peek().token_type {
                TokenType::If
                | TokenType::While
                | TokenType::For
                | TokenType::Return
                | TokenType::Var
                | TokenType::Print => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
