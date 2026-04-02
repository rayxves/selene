use crate::expr::Expression;

pub trait StmtVisitor {
    type Output;
    fn visit_print(&mut self, expr: &Expression) -> Self::Output;
    fn visit_expr_statement(&mut self, expr: &Expression) -> Self::Output;
    fn visit_var(&mut self, name: &String, expr: Option<&Expression>) -> Self::Output;
}

pub enum Statement {
    Print(Expression),
    ExprStatement(Expression),
    Var(String, Option<Expression>),
}

impl Statement {
    pub fn accept<V: StmtVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Statement::ExprStatement(expr) => visitor.visit_expr_statement(expr),
            Statement::Print(expr) => visitor.visit_print(expr),
            Statement::Var(name, expr) => visitor.visit_var(name, expr.as_ref()),
        }
    }
}
