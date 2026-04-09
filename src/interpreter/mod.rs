mod environment;
mod values;

use std::{cell::RefCell, rc::Rc};

pub use values::{RuntimeError, SeleneValue};

use crate::{
    expr::{ExprVisitor, Expression},
    interpreter::environment::Environment,
    stmt::{Statement, StmtVisitor},
    token::{BinaryOp, LogicalOp, TokenLiteral, UnaryOp},
};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl ExprVisitor for Interpreter {
    type Output = Result<SeleneValue, RuntimeError>;

    fn visit_binary(
        &mut self,
        left: &crate::expr::Expression,
        operator: &crate::token::BinaryOp,
        line: &u64,
        right: &crate::expr::Expression,
    ) -> Self::Output {
        match operator {
            BinaryOp::Plus => {
                let value_left = self.evaluate(left)?;
                let value_right = self.evaluate(right)?;

                match (value_left, value_right) {
                    (SeleneValue::Number(a), SeleneValue::Number(b)) => {
                        Ok(SeleneValue::Number(a + b))
                    }
                    (SeleneValue::String(a), SeleneValue::String(b)) => {
                        Ok(SeleneValue::String(a + &b))
                    }
                    (SeleneValue::Number(_), _) => Err(RuntimeError::new(
                        *line,
                        "Operando direito do '+' deve ser um número.".to_string(),
                    )),
                    (SeleneValue::String(_), _) => Err(RuntimeError::new(
                        *line,
                        "Operando direito do '+' deve ser uma string.".to_string(),
                    )),
                    _ => Err(RuntimeError::new(
                        *line,
                        "Operandos do '+' devem ser dois números ou duas strings.".to_string(),
                    )),
                }
            }
            BinaryOp::Minus => {
                let (a, b) = self.extract_numbers(left, right, *line, "-")?;
                Ok(SeleneValue::Number(a - b))
            }

            BinaryOp::Slash => {
                let (a, b) = self.extract_numbers(left, right, *line, "/")?;
                if b == 0.0 {
                    return Err(RuntimeError::new(
                        *line,
                        "Não é possível realizar uma divisão por zero.".to_string(),
                    ));
                }
                Ok(SeleneValue::Number(a / b))
            }
            BinaryOp::Star => {
                let (a, b) = self.extract_numbers(left, right, *line, "*")?;
                Ok(SeleneValue::Number(a * b))
            }
            BinaryOp::Greater => {
                let (a, b) = self.extract_numbers(left, right, *line, ">")?;
                Ok(SeleneValue::Boolean(a > b))
            }
            BinaryOp::GreaterEqual => {
                let (a, b) = self.extract_numbers(left, right, *line, ">=")?;
                Ok(SeleneValue::Boolean(a >= b))
            }
            BinaryOp::Less => {
                let (a, b) = self.extract_numbers(left, right, *line, "<")?;
                Ok(SeleneValue::Boolean(a < b))
            }
            BinaryOp::LessEqual => {
                let (a, b) = self.extract_numbers(left, right, *line, "<=")?;
                Ok(SeleneValue::Boolean(a <= b))
            }
            BinaryOp::EqualEqual => {
                let value_left = self.evaluate(left)?;
                let value_right = self.evaluate(right)?;
                Ok(SeleneValue::Boolean(value_left == value_right))
            }
            BinaryOp::BangEqual => {
                let value_left = self.evaluate(left)?;
                let value_right = self.evaluate(right)?;
                Ok(SeleneValue::Boolean(value_left != value_right))
            }
        }
    }

    fn visit_literal(&mut self, literal: &crate::token::TokenLiteral) -> Self::Output {
        match literal {
            TokenLiteral::Number(n) => Ok(SeleneValue::Number(*n)),
            TokenLiteral::Boolean(b) => Ok(SeleneValue::Boolean(*b)),
            TokenLiteral::StringLiteral(s) => Ok(SeleneValue::String(s.clone())),
            TokenLiteral::Null => Ok(SeleneValue::Null),
        }
    }

    fn visit_unary(
        &mut self,
        unary_op: &crate::token::UnaryOp,
        line: &u64,
        expr: &crate::expr::Expression,
    ) -> Self::Output {
        match unary_op {
            UnaryOp::Minus => {
                let value = self.evaluate(expr)?;
                match value {
                    SeleneValue::Number(n) => Ok(SeleneValue::Number(-n)),
                    _ => Err(RuntimeError::new(
                        *line,
                        "Operando deve ser um número".to_string(),
                    )),
                }
            }

            UnaryOp::Bang => {
                let value = self.evaluate(expr)?;
                Ok(SeleneValue::Boolean(!Interpreter::is_truthy(&value)))
            }
        }
    }

    fn visit_grouping(&mut self, expr: &Expression) -> Self::Output {
        self.evaluate(expr)
    }

    fn visit_variable(&mut self, name: &String, line: u64) -> Self::Output {
        Environment::get(&self.environment, name, line)
    }

    fn visit_assign(&mut self, name: &String, line: u64, expr: &Expression) -> Self::Output {
        let value = self.evaluate(expr)?;
        Environment::assign(&self.environment, name.clone(), line, value)
    }

    fn visit_logical(
        &mut self,
        left: &Expression,
        operator: &crate::token::LogicalOp,
        _line: &u64,
        right: &Expression,
    ) -> Self::Output {
        match operator {
            LogicalOp::And => {
                let left_val = self.evaluate(left)?;
                if !Interpreter::is_truthy(&left_val) {
                    return Ok(left_val);
                }
                self.evaluate(right)
            }
            LogicalOp::Or => {
                let left_val = self.evaluate(left)?;
                if Interpreter::is_truthy(&left_val) {
                    return Ok(left_val); 
                }
                self.evaluate(right)
            }
        }
    }
}

impl StmtVisitor for Interpreter {
    type Output = Result<(), RuntimeError>;

    fn visit_print(&mut self, expr: &Expression) -> Self::Output {
        let value = self.evaluate(expr)?;
        println!("{}", value.to_display());
        Ok(())
    }

    fn visit_expr_statement(&mut self, expr: &Expression) -> Self::Output {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_var(&mut self, name: &String, expr: Option<&Expression>) -> Self::Output {
        let value = match expr {
            Some(e) => self.evaluate(e)?,
            None => SeleneValue::Null,
        };
        Environment::define(&self.environment, name.clone(), value);
        Ok(())
    }

    fn visit_block(&mut self, statements: &Vec<Statement>) -> Self::Output {
        let child = Environment::new_enclosed(Rc::clone(&self.environment));
        let previous = Rc::clone(&self.environment);
        self.environment = child;

        let mut result = Ok(());
        for stmt in statements {
            match stmt.accept(self) {
                Ok(_) => {}
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }

        self.environment = previous;
        result
    }

    fn visit_if(
        &mut self,
        expr: &Expression,
        stmt: &Statement,
        else_stmt: Option<&Statement>,
    ) -> Self::Output {
        if Interpreter::is_truthy(&self.evaluate(expr)?) {
            stmt.accept(self)
        } else {
            match else_stmt {
                Some(e) => e.accept(self),
                None => Ok(()),
            }
        }
    }

    fn visit_while(&mut self, expr: &Expression, stmt: &Statement) -> Self::Output {
        while Interpreter::is_truthy(&self.evaluate(expr)?) {
            stmt.accept(self)?
        }
        Ok(())
    }
}
impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn evaluate(&mut self, expr: &Expression) -> Result<SeleneValue, RuntimeError> {
        expr.accept(self)
    }

    pub fn is_truthy(value: &SeleneValue) -> bool {
        match value {
            SeleneValue::Boolean(b) => *b,
            SeleneValue::Null => false,
            _ => true,
        }
    }

    fn extract_numbers(
        &mut self,
        left: &Expression,
        right: &Expression,
        line: u64,
        operator: &str,
    ) -> Result<(f64, f64), RuntimeError> {
        let value_left = self.evaluate(left)?;
        let value_right = self.evaluate(right)?;

        match (value_left, value_right) {
            (SeleneValue::Number(a), SeleneValue::Number(b)) => Ok((a, b)),
            _ => Err(RuntimeError::new(
                line,
                format!("Operandos do '{}' devem ser números.", operator),
            )),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) {
        for stmt in statements {
            let statment = stmt.accept(self);
            match statment {
                Ok(_) => {}
                Err(e) => println!("Erro na linha {}: {}", e.line, e.message),
            }
        }
    }
}
