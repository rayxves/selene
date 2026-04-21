use std::collections::HashMap;

use crate::{
    expr::{ExprVisitor, Expression},
    interpreter::Interpreter,
    stmt::{ Statement, StmtVisitor},
    token::Token,
};

pub struct ResolveError {
    pub line: u64,
    pub message: String,
}

impl ResolveError {
    pub fn new(line: u64, message: String) -> ResolveError {
        ResolveError { line, message }
    }
}
pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    is_function: IsFunction,
    is_class: IsClass,
}

#[derive(Clone, Copy)]
pub enum IsFunction {
    Function,
    Initializer,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IsClass {
    Class,
    None,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            is_function: IsFunction::None,
            is_class: IsClass::None,
        }
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: String, line: u64) -> Result<(), ResolveError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                return Err(ResolveError::new(
                    line,
                    "Já existe uma variável com esse nome neste escopo".to_string(),
                ));
            }
            scope.insert(name, false);
        }
        Ok(())
    }

    pub fn define(&mut self, name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, true);
        }
    }

    pub fn resolve_local(&mut self, id: usize, name: &str) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(name) {
                self.interpreter.resolve(id, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    pub fn into_interpreter(self) -> Interpreter {
        self.interpreter
    }

    pub fn resolve(&mut self, statements: &Vec<Statement>) -> Result<(), ResolveError> {
        for stmt in statements {
            stmt.accept(self)?;
        }
        Ok(())
    }
}

impl ExprVisitor for Resolver {
    type Output = Result<(), ResolveError>;

    fn visit_binary(
        &mut self,
        left: &Expression,
        _operator: &crate::token::BinaryOp,
        _line: &u64,
        right: &Expression,
    ) -> Self::Output {
        left.accept(self)?;
        right.accept(self)?;
        Ok(())
    }

    fn visit_literal(&mut self, _literal: &crate::token::TokenLiteral) -> Self::Output {
        Ok(())
    }

    fn visit_unary(
        &mut self,
        _unary_op: &crate::token::UnaryOp,
        _line: &u64,
        expr: &Expression,
    ) -> Self::Output {
        expr.accept(self)?;
        Ok(())
    }

    fn visit_grouping(&mut self, expr: &Expression) -> Self::Output {
        expr.accept(self)?;
        Ok(())
    }

    fn visit_variable(&mut self, name: &String, line: u64, id: usize) -> Self::Output {
        if !self.scopes.is_empty() && self.scopes.last().unwrap().get(name) == Some(&false) {
            return Err(ResolveError::new(
                line,
                "Não é possivel usar uma variavel que não foi declarada".to_string(),
            ));
        }
        self.resolve_local(id, name);
        Ok(())
    }

    fn visit_assign(
        &mut self,
        name: &String,
        _line: u64,
        expr: &Expression,
        id: usize,
    ) -> Self::Output {
        expr.accept(self)?;
        self.resolve_local(id, name);
        Ok(())
    }

    fn visit_logical(
        &mut self,
        left: &Expression,
        _operator: &crate::token::LogicalOp,
        _line: &u64,
        right: &Expression,
    ) -> Self::Output {
        left.accept(self)?;
        right.accept(self)?;
        Ok(())
    }

    fn visit_call(
        &mut self,
        callee: &Expression,
        args: &Vec<Expression>,
        _paren: &Token,
    ) -> Self::Output {
        callee.accept(self)?;
        for arg in args {
            arg.accept(self)?;
        }
        Ok(())
    }

    fn visit_get(&mut self, expr: &Expression, _token: &Token) -> Self::Output {
        expr.accept(self)?;
        Ok(())
    }

    fn visit_set(&mut self, expr: &Expression, _token: &Token, value: &Expression) -> Self::Output {
        expr.accept(self)?;
        value.accept(self)?;
        Ok(())
    }

    fn visit_this(&mut self, token: &Token, id: usize) -> Self::Output {
        if self.is_class == IsClass::Class {
            self.resolve_local(id, "this");
            Ok(())
        } else {
            return Err(ResolveError::new(
                token.line,
                "Não é possivel usar this sem uma classe".to_string(),
            ));
        }
    }
}
impl StmtVisitor for Resolver {
    type Output = Result<(), ResolveError>;

    fn visit_print(&mut self, expr: &Expression) -> Self::Output {
        expr.accept(self)?;
        Ok(())
    }

    fn visit_expr_statement(&mut self, expr: &Expression) -> Self::Output {
        expr.accept(self)?;
        Ok(())
    }

    fn visit_var(&mut self, name: &String, expr: Option<&Expression>, line: u64) -> Self::Output {
        self.declare(name.clone(), line)?;
        if let Some(e) = expr {
            e.accept(self)?;
        };
        self.define(name.clone());
        Ok(())
    }

    fn visit_block(&mut self, statements: &Vec<crate::stmt::Statement>) -> Self::Output {
        self.begin_scope();
        for stmt in statements {
            stmt.accept(self)?;
        }
        self.end_scope();
        Ok(())
    }

    fn visit_if(
        &mut self,
        expr: &Expression,
        stmt: &crate::stmt::Statement,
        else_stmt: Option<&crate::stmt::Statement>,
    ) -> Self::Output {
        expr.accept(self)?;
        stmt.accept(self)?;
        if let Some(e) = else_stmt {
            e.accept(self)?;
        }
        Ok(())
    }

    fn visit_while(&mut self, expr: &Expression, stmt: &crate::stmt::Statement) -> Self::Output {
        expr.accept(self)?;
        stmt.accept(self)?;
        Ok(())
    }

    fn visit_function(
        &mut self,
        name: &String,
        params: &Vec<String>,
        stmts: &Vec<crate::stmt::Statement>,
        line: u64,
    ) -> Self::Output {
        self.declare(name.clone(), line)?;
        self.define(name.clone());
        self.begin_scope();
        let is_fun = self.is_function;
        self.is_function = IsFunction::Function;
        for param in params {
            self.declare(param.clone(), line)?;
            self.define(param.clone());
        }
        for stmt in stmts {
            stmt.accept(self)?;
        }
        self.is_function = is_fun;
        self.end_scope();
        Ok(())
    }

    fn visit_return(&mut self, line: u64, value: Option<&Expression>) -> Self::Output {
        if matches!(self.is_function, IsFunction::None) {
            return Err(ResolveError::new(
                line,
                "Return só pode ser usado em funções".to_string(),
            ));
        } else if matches!(self.is_function, IsFunction::Initializer) && value.is_some(){
            return Err(ResolveError::new(
                line,
                "Não é possível retornar um valor de init".to_string(),
            ));
        }
        if let Some(v) = value {
            v.accept(self)?;
        }
        Ok(())
    }

    fn visit_class(
        &mut self,
        name: &String,
        line: u64,
        statements: &Vec<Statement>,
    ) -> Self::Output {
        let current_is_class = self.is_class;
        self.is_class = IsClass::Class;
        self.declare(name.clone(), line)?;
        self.define(name.clone());
        self.begin_scope();
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert("this".to_string(), true);
        }

        for stmt in statements {
            match stmt {
                Statement::Function(n, params, body, _line) => {
                    let prev_fn = self.is_function;
                    if n == "init" {
                        self.is_function = IsFunction::Initializer;
                    } else {
                        self.is_function = IsFunction::Function;
                    }
                    self.begin_scope();
                    for param in params {
                        self.declare(param.clone(), line)?;
                        self.define(param.clone());
                    }
                    self.resolve(body)?;
                    self.end_scope();
                    self.is_function = prev_fn;
                }
                _ => {}
            }
        }

        self.end_scope();
        self.is_class = current_is_class;
        Ok(())
    }
}
