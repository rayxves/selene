use crate::{
    expr::Expression,
    parser::{ParseError, Parser},
    stmt::Statement,
    token::TokenType,
};

impl Parser {
    pub fn print_statement(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let expr = self.expression()?;
        if self.check(&TokenType::Semicolon) {
            self.advance();
            Ok(Statement::Print(expr))
        } else {
            let error = ParseError::new(
                self.peek().clone(),
                "Esperava ';' após o valor do print.".to_string(),
            );
            self.errors.push(error.clone());
            Err(error)
        }
    }

    pub fn expr_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        if self.check(&TokenType::Semicolon) {
            self.advance();
            Ok(Statement::ExprStatement(expr))
        } else {
            let error = ParseError::new(
                self.peek().clone(),
                "Esperava ';' após expressão.".to_string(),
            );
            self.errors.push(error.clone());
            Err(error)
        }
    }

    pub fn var_statement(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let expr: Option<Expression>;
        let identifier: String;
        match &self.peek().token_type {
            TokenType::Identifier(n) => identifier = n.clone(),
            _ => {
                let error = ParseError::new(
                    self.peek().clone(),
                    "Esperava uma string válida para nome de variável.".to_string(),
                );
                self.errors.push(error.clone());
                return Err(error);
            }
        }

        self.advance();
        match self.peek().token_type {
            TokenType::Equal => {
                self.advance();
                expr = Some(self.expression()?);
            }
            _ => expr = None,
        }

        if self.check(&TokenType::Semicolon) {
            self.advance();
            Ok(Statement::Var(identifier, expr))
        } else {
            let error = ParseError::new(
                self.peek().clone(),
                "Esperava ';' após atribuição de variável.".to_string(),
            );
            self.errors.push(error.clone());
            Err(error)
        }
    }

    pub fn statement(&mut self) -> Option<Statement> {
        match self.peek().token_type {
            TokenType::Print => {
                let print_stmt = self.print_statement();
                match print_stmt {
                    Ok(n) => return Some(n),
                    Err(_e) => {
                        self.synchronize();
                        return None;
                    }
                }
            }

            TokenType::Var => {
                let var_stmt = self.var_statement();
                match var_stmt {
                    Ok(n) => return Some(n),
                    Err(_e) => {
                        self.synchronize();
                        return None;
                    }
                }
            }

            TokenType::LeftBrace => return self.block_statment(),

            _ => {
                let expr_stmt = self.expr_statement();
                match expr_stmt {
                    Ok(n) => return Some(n),
                    Err(_e) => {
                        self.synchronize();
                        return None;
                    }
                }
            }
        };
    }

    pub fn block_statment(&mut self) -> Option<Statement> {
        self.advance();
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let stmt = self.statement();
            match stmt {
                Some(e) => {
                    statements.push(e);
                }
                None => return None,
            }
        }
        self.advance();
        Some(Statement::Block(statements))
    }
}
