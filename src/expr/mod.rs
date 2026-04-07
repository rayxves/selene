use crate::token::{BinaryOp, TokenLiteral, UnaryOp};

pub trait ExprVisitor {
    type Output;
    fn visit_binary(
        &mut self,
        left: &Expression,
        operator: &BinaryOp,
        line: &u64,
        right: &Expression,
    ) -> Self::Output;
    fn visit_literal(&mut self, literal: &TokenLiteral) -> Self::Output;
    fn visit_unary(&mut self, unary_op: &UnaryOp, line: &u64, expr: &Expression) -> Self::Output;
    fn visit_grouping(&mut self, expr: &Expression) -> Self::Output;
    fn visit_variable(&mut self, name: &String, line: u64) -> Self::Output;
    fn visit_assign(&mut self, name: &String, line: u64, expr: &Expression) -> Self::Output;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOp, u64, Box<Expression>),
    Unary(UnaryOp, u64, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(TokenLiteral),
    Variable(String, u64),
    Assign(String, u64, Box<Expression>),
}

impl Expression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Expression::Binary(left, op, line, right) => {
                visitor.visit_binary(left, op, line, right)
            }
            Expression::Literal(literal) => visitor.visit_literal(literal),
            Expression::Grouping(expr) => visitor.visit_grouping(expr),
            Expression::Unary(unary_op, line, expr) => visitor.visit_unary(unary_op, line, expr),
            Expression::Variable(name, line) => visitor.visit_variable(name, *line),
            Expression::Assign(name, line, expr) => visitor.visit_assign(name, *line, expr),
        }
    }
}
