use crate::token::{BinaryOp, TokenLiteral, UnaryOp};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOp, Box<Expression>),
    Literal(TokenLiteral),
    Unary(UnaryOp, Box<Expression>),
    Grouping(Box<Expression>),
}

pub fn print_tree(expr: &Expression, indent: usize) {
    let pad = " ".repeat(indent);
    match expr {
        Expression::Literal(lit) => println!("{}{:?}", pad, lit),
        Expression::Unary(op, right) => {
            println!("{}{:?}", pad, op);
            print_tree(right, indent + 2);
        }
        Expression::Binary(left, op, right) => {
            println!("{}{:?}", pad, op);
            print_tree(left, indent + 2);
            print_tree(right, indent + 2);
        }
        Expression::Grouping(expr) => {
            println!("{}group", pad);
            print_tree(expr, indent + 2);
        }
    }
}
