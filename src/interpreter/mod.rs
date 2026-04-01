use crate::{
    expr::{ExprVisitor, Expression},
    token::{BinaryOp, TokenLiteral, UnaryOp},
};

#[derive(Debug, Clone, PartialEq)]
pub enum SeleneValue {
    Number(f64),
    Boolean(bool),
    String(String),
    Null,
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub line: u64,
    pub message: String,
}

impl RuntimeError {
    pub fn new(line: u64, message: String) -> RuntimeError {
        RuntimeError { line, message }
    }
}

pub struct Interpreter {}

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
                    return Err(RuntimeError::new(*line, "Não é possível realizar uma divisão por zero.".to_string()));
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
}

impl Interpreter {
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
}
