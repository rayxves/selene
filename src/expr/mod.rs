use crate::token::{BinaryOp, TokenLiteral, UnaryOp};

pub trait ExprVisitor {
    type Output;
    fn visit_binary(
        &mut self,
        left: &Expression,
        operator: &BinaryOp,
        right: &Expression,
    ) -> Self::Output;
    fn visit_literal(&mut self, literal: &TokenLiteral) -> Self::Output;
    fn visit_unary(&mut self, unary_op: &UnaryOp, expr: &Expression) -> Self::Output;
    fn visit_grouping(&mut self, expr: &Expression) -> Self::Output;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOp, Box<Expression>),
    Literal(TokenLiteral),
    Unary(UnaryOp, Box<Expression>),
    Grouping(Box<Expression>),
}

impl Expression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Expression::Binary(left, op, right) => visitor.visit_binary(left, op, right),
            Expression::Literal(literal) => visitor.visit_literal(literal),
            Expression::Grouping(expr) => visitor.visit_grouping(expr),
            Expression::Unary(unary_op, expr) => visitor.visit_unary(unary_op, expr),
        }
    }
}
