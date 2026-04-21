use crate::token::{BinaryOp, LogicalOp, Token, TokenLiteral, UnaryOp};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

pub fn next_id() -> usize {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
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
    fn visit_variable(&mut self, name: &String, line: u64, id: usize) -> Self::Output;
    fn visit_assign(
        &mut self,
        name: &String,
        line: u64,
        expr: &Expression,
        id: usize,
    ) -> Self::Output;
    fn visit_logical(
        &mut self,
        left: &Expression,
        operator: &LogicalOp,
        line: &u64,
        right: &Expression,
    ) -> Self::Output;
    fn visit_call(
        &mut self,
        callee: &Expression,
        args: &Vec<Expression>,
        paren: &Token,
    ) -> Self::Output;
    fn visit_get(&mut self, expr: &Expression, token: &Token) -> Self::Output;
    fn visit_set(&mut self, expr: &Expression, token: &Token, value: &Expression) -> Self::Output;
    fn visit_this(&mut self, token: &Token, id: usize) -> Self::Output;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOp, u64, Box<Expression>),
    Unary(UnaryOp, u64, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(TokenLiteral),
    Variable(String, u64, usize),
    Assign(String, u64, Box<Expression>, usize),
    Logical(Box<Expression>, LogicalOp, u64, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>, Token),
    Get(Box<Expression>, Token),
    Set(Box<Expression>, Token, Box<Expression>),
    This(Token, usize),
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
            Expression::Variable(name, line, id) => visitor.visit_variable(name, *line, *id),
            Expression::Assign(name, line, expr, id) => {
                visitor.visit_assign(name, *line, expr, *id)
            }
            Expression::Logical(left, logical_op, line, right) => {
                visitor.visit_logical(left, logical_op, line, right)
            }
            Expression::Call(callee, args, paren) => visitor.visit_call(callee, args, paren),
            Expression::Get(expr, token) => visitor.visit_get(expr, token),
            Expression::Set(expr, token, value) => visitor.visit_set(expr, token, value),
            Expression::This(token, id) => visitor.visit_this(token, *id),
        }
    }
}
