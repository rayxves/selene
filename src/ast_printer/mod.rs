use crate::{
    expr::{ExprVisitor, Expression},
    token::{BinaryOp, LogicalOp, TokenLiteral, UnaryOp},
};

pub struct AstPrinter;

impl ExprVisitor for AstPrinter {
    type Output = String;

    fn visit_binary(
        &mut self,
        left: &Expression,
        operator: &crate::token::BinaryOp,
        _line: &u64,
        right: &Expression,
    ) -> Self::Output {
        let op = match operator {
            BinaryOp::BangEqual => "!=".to_string(),
            BinaryOp::EqualEqual => "==".to_string(),
            BinaryOp::Greater => ">".to_string(),
            BinaryOp::GreaterEqual => ">=".to_string(),
            BinaryOp::Less => "<".to_string(),
            BinaryOp::LessEqual => "<=".to_string(),
            BinaryOp::Minus => "-".to_string(),
            BinaryOp::Plus => "+".to_string(),
            BinaryOp::Slash => "/".to_string(),
            BinaryOp::Star => "*".to_string(),
        };
        let left_str = left.accept(self);
        let right_str = right.accept(self);
        format!("(left {}, op {}, right {})", left_str, op, right_str)
    }

    fn visit_literal(&mut self, literal: &crate::token::TokenLiteral) -> Self::Output {
        return match literal {
            TokenLiteral::Number(n) => n.to_string(),
            TokenLiteral::Boolean(b) => b.to_string(),
            TokenLiteral::StringLiteral(s) => s.to_string(),
            TokenLiteral::Null => "null".to_string(),
        };
    }

    fn visit_unary(
        &mut self,
        unary_op: &crate::token::UnaryOp,
        _line: &u64,
        expr: &Expression,
    ) -> Self::Output {
        let op = match unary_op {
            UnaryOp::Bang => "!".to_string(),
            UnaryOp::Minus => "-".to_string(),
        };

        format!("op {} expr {}", op, expr.accept(self))
    }

    fn visit_grouping(&mut self, expr: &Expression) -> Self::Output {
        format!("(group {})", expr.accept(self))
    }

    fn visit_variable(&mut self, value: &String, _line: u64, _id: usize) -> Self::Output {
        value.to_string()
    }

    fn visit_assign(
        &mut self,
        name: &String,
        _line: u64,
        _expr: &Expression,
        _id: usize,
    ) -> Self::Output {
        name.to_string()
    }

    fn visit_logical(
        &mut self,
        left: &Expression,
        operator: &crate::token::LogicalOp,
        _line: &u64,
        right: &Expression,
    ) -> Self::Output {
        let op = match operator {
            LogicalOp::And => "and".to_string(),
            LogicalOp::Or => "or".to_string(),
        };
        let left_str = left.accept(self);
        let right_str = right.accept(self);
        format!("(left {}, op {}, right {})", left_str, op, right_str)
    }

    fn visit_call(
        &mut self,
        callee: &Expression,
        args: &Vec<Expression>,
        _paren: &crate::token::Token,
    ) -> Self::Output {
        let callee = callee.accept(self);
        let args_str = args
            .iter()
            .map(|arg| arg.accept(self))
            .collect::<Vec<String>>()
            .join(", ");
        format!("callee: {}, args: {}", callee, args_str)
    }

    fn visit_get(&mut self, expr: &Expression, token: &crate::token::Token) -> Self::Output {
        let expr = expr.accept(self);
        format!("{}.{}", expr, token.lexeme)
    }

    fn visit_set(
        &mut self,
        expr: &Expression,
        token: &crate::token::Token,
        value: &Expression,
    ) -> Self::Output {
        let expr = expr.accept(self);
        let value = value.accept(self);
        format!("{}.{} = {}", expr, token.lexeme, value)
    }

    fn visit_this(&mut self, _token: &crate::token::Token, _id: usize) -> Self::Output {
        "this".to_string()
    }

    fn visit_super(
        &mut self,
        _key_super: &crate::token::Token,
        method: &crate::token::Token,
        _id: usize,
    ) -> Self::Output {
        format!("Superclass method name: {}", method.lexeme)
    }
}
