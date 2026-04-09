use crate::{
    expr::Expression,
    parser::{ParseError, Parser},
    stmt::Statement,
    token::TokenType,
};

impl Parser {
    pub fn error(&mut self, message: String) -> ParseError {
        let error = ParseError::new(self.peek().clone(), message);
        self.errors.push(error.clone());
        error
    }

    pub fn print_statement(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let value = self.expression()?;
        if self.check(&TokenType::Semicolon) {
            self.advance();
            Ok(Statement::Print(value))
        } else {
            Err(self.error(format!(
                "Esperava ';' após o valor do print, mas encontrei '{}'.",
                self.peek().lexeme
            )))
        }
    }

    pub fn expr_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        if self.check(&TokenType::Semicolon) {
            self.advance();
            Ok(Statement::ExprStatement(expr))
        } else {
            Err(self.error(format!(
                "Esperava ';' após expressão, mas encontrei '{}'.",
                self.peek().lexeme
            )))
        }
    }

    pub fn var_statement(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let initializer: Option<Expression>;
        let name: String;
        match &self.peek().token_type {
            TokenType::Identifier(identifier) => name = identifier.clone(),
            _ => {
                return Err(self.error(format!(
                    "Esperava nome de variável após 'var', mas encontrei '{}'.",
                    self.peek().lexeme
                )));
            }
        }

        self.advance();
        match self.peek().token_type {
            TokenType::Equal => {
                self.advance();
                initializer = Some(self.expression()?);
            }
            _ => initializer = None,
        }

        if self.check(&TokenType::Semicolon) {
            self.advance();
            Ok(Statement::Var(name, initializer))
        } else {
            Err(self.error(format!(
                "Esperava ';' após declaração de variável, mas encontrei '{}'.",
                self.peek().lexeme
            )))
        }
    }

    pub fn statement(&mut self) -> Option<Statement> {
        match self.peek().token_type {
            TokenType::Print => match self.print_statement() {
                Ok(stmt) => Some(stmt),
                Err(_) => {
                    self.synchronize();
                    None
                }
            },
            TokenType::Var => match self.var_statement() {
                Ok(stmt) => Some(stmt),
                Err(_) => {
                    self.synchronize();
                    None
                }
            },
            TokenType::LeftBrace => self.block_statement(),
            TokenType::If => self.if_statement(),
            TokenType::While => self.while_statement(),
            TokenType::For => self.for_statement(),
            TokenType::Function => self.function_statement(),
            TokenType::Return => self.return_statement(),
            _ => match self.expr_statement() {
                Ok(stmt) => Some(stmt),
                Err(_) => {
                    self.synchronize();
                    None
                }
            },
        }
    }

    pub fn block_statement(&mut self) -> Option<Statement> {
        self.advance();
        let mut body = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match self.statement() {
                Some(stmt) => body.push(stmt),
                None => return None,
            }
        }
        if self.is_at_end() {
            self.error("Bloco não fechado — esperava '}'.".to_string());
            return None;
        }
        self.advance();
        Some(Statement::Block(body))
    }

    pub fn if_statement(&mut self) -> Option<Statement> {
        self.advance();
        if !self.check(&TokenType::LeftParen) {
            self.error(format!(
                "Esperava '(' após 'if', mas encontrei '{}'.",
                self.peek().lexeme
            ));
            self.synchronize();
            return None;
        }
        self.advance();

        let condition = match self.expression() {
            Ok(condition) => condition,
            Err(_) => {
                self.synchronize();
                return None;
            }
        };

        if !self.check(&TokenType::RightParen) {
            self.error(format!(
                "Esperava ')' após condição do 'if', mas encontrei '{}'.",
                self.peek().lexeme
            ));
            self.synchronize();
            return None;
        }
        self.advance();

        let then_branch = match self.statement() {
            Some(stmt) => stmt,
            None => return None,
        };

        if self.check(&TokenType::Else) {
            self.advance();
            match self.statement() {
                Some(else_branch) => Some(Statement::If(
                    condition,
                    Box::new(then_branch),
                    Some(Box::new(else_branch)),
                )),
                None => None,
            }
        } else {
            Some(Statement::If(condition, Box::new(then_branch), None))
        }
    }

    pub fn while_statement(&mut self) -> Option<Statement> {
        self.advance();
        if !self.check(&TokenType::LeftParen) {
            self.error(format!(
                "Esperava '(' após 'while', mas encontrei '{}'.",
                self.peek().lexeme
            ));
            self.synchronize();
            return None;
        }
        self.advance();

        let condition = match self.expression() {
            Ok(condition) => condition,
            Err(_) => {
                self.synchronize();
                return None;
            }
        };

        if !self.check(&TokenType::RightParen) {
            self.error(format!(
                "Esperava ')' após condição do 'while', mas encontrei '{}'.",
                self.peek().lexeme
            ));
            self.synchronize();
            return None;
        }
        self.advance();

        match self.statement() {
            Some(body) => Some(Statement::While(condition, Box::new(body))),
            None => None,
        }
    }

    pub fn for_statement(&mut self) -> Option<Statement> {
        self.advance();
        if !self.check(&TokenType::LeftParen) {
            self.error(format!(
                "Esperava '(' após 'for', mas encontrei '{}'.",
                self.peek().lexeme
            ));
            self.synchronize();
            return None;
        }
        self.advance();

        let initializer: Option<Statement> = match self.peek().token_type {
            TokenType::Var => match self.var_statement() {
                Ok(stmt) => Some(stmt),
                Err(_) => {
                    self.synchronize();
                    return None;
                }
            },
            TokenType::Semicolon => {
                self.advance();
                None
            }
            _ => match self.expr_statement() {
                Ok(stmt) => Some(stmt),
                Err(_) => {
                    self.synchronize();
                    return None;
                }
            },
        };

        let condition = match self.expression() {
            Ok(condition) => condition,
            Err(_) => {
                self.synchronize();
                return None;
            }
        };

        if !self.check(&TokenType::Semicolon) {
            self.error(format!(
                "Esperava ';' após condição do 'for', mas encontrei '{}'.",
                self.peek().lexeme
            ));
            self.synchronize();
            return None;
        }
        self.advance();

        let increment = match self.expression() {
            Ok(increment) => increment,
            Err(_) => {
                self.synchronize();
                return None;
            }
        };

        if !self.check(&TokenType::RightParen) {
            self.error(format!(
                "Esperava ')' após incremento do 'for', mas encontrei '{}'.",
                self.peek().lexeme
            ));
            self.synchronize();
            return None;
        }
        self.advance();

        match self.statement() {
            Some(body) => {
                let while_body = Statement::Block(vec![body, Statement::ExprStatement(increment)]);
                let mut outer = vec![Statement::While(condition, Box::new(while_body))];
                if let Some(init) = initializer {
                    outer.insert(0, init);
                }
                Some(Statement::Block(outer))
            }
            None => None,
        }
    }

    pub fn function_statement(&mut self) -> Option<Statement> {
        self.advance();
        let name: String;
        match &self.peek().token_type {
            TokenType::Identifier(identifier) => name = identifier.clone(),
            _ => {
                self.error(format!(
                    "Esperava um nome, mas encontrei '{}'.",
                    self.peek().lexeme
                ));
                return None;
            }
        }
        let mut params = Vec::new();
        self.advance();
        match self.peek().token_type {
            TokenType::LeftParen => {
                self.advance();
                while !self.check(&TokenType::RightParen) {
                    match &self.peek().token_type {
                        TokenType::Identifier(name) => {
                            params.push(name.clone());
                            self.advance();
                            if self.check(&TokenType::Comma) {
                                self.advance();
                            } else if !self.check(&TokenType::RightParen) {
                                self.error(format!(
                                    "Esperava ',' ou ')' após parâmetro, mas encontrei '{}'.",
                                    self.peek().lexeme
                                ));
                                return None;
                            }
                        }
                        _ => {
                            self.error(format!(
                                "Esperava um nome de parâmetro, mas encontrei '{}'.",
                                self.peek().lexeme
                            ));
                            return None;
                        }
                    }
                }
                self.advance();
                match self.block_statement() {
                    Some(Statement::Block(stmts)) => {
                        return Some(Statement::Function(name, params, stmts));
                    }
                    _ => return None,
                }
            }
            _ => {
                self.error(format!(
                    "Esperava '(' após nome da função, mas encontrei '{}'.",
                    self.peek().lexeme
                ));
                return None;
            }
        }
    }

    fn return_statement(&mut self) -> Option<Statement> {
        self.advance();
        if self.check(&TokenType::Semicolon) {
            self.advance();
            return Some(Statement::Return(self.peek().line, None));
        } else {
            match self.expression() {
                Ok(expr) => {
                    let expr = Some(Statement::Return(self.peek().line, Some(expr)));
                    if self.check(&TokenType::Semicolon) {
                        self.advance();
                        return expr;
                    }

                    self.error(format!(
                        "Esperava ';' após retorno, mas encontrei '{}'.",
                        self.peek().lexeme
                    ));
                    return None;
                }
                Err(_e) => {
                    self.error(format!(
                        "Esperava um valor válido de retorno, mas encontrei '{}'.",
                        self.peek().lexeme
                    ));
                    return None;
                }
            }
        }
    }
}
