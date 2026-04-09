use crate::expr::Expression;

pub trait StmtVisitor {
    type Output;
    fn visit_print(&mut self, expr: &Expression) -> Self::Output;
    fn visit_expr_statement(&mut self, expr: &Expression) -> Self::Output;
    fn visit_var(&mut self, name: &String, expr: Option<&Expression>) -> Self::Output;
    fn visit_block(&mut self, statements: &Vec<Statement>) -> Self::Output;
    fn visit_if(
        &mut self,
        expr: &Expression,
        stmt: &Statement,
        else_stmt: Option<&Statement>,
    ) -> Self::Output;
    fn visit_while(&mut self, expr: &Expression, stmt: &Statement) -> Self::Output;
}

pub enum Statement {
    Print(Expression),
    ExprStatement(Expression),
    Var(String, Option<Expression>),
    Block(Vec<Statement>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
}

impl Statement {
    pub fn accept<V: StmtVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Statement::ExprStatement(expr) => visitor.visit_expr_statement(expr),
            Statement::Print(expr) => visitor.visit_print(expr),
            Statement::Var(name, expr) => visitor.visit_var(name, expr.as_ref()),
            Statement::Block(statements) => visitor.visit_block(statements),
            Statement::If(expr, stmt, else_stmt) => {
                visitor.visit_if(expr, stmt, else_stmt.as_deref())
            }
            Statement::While(expr, stmt) => visitor.visit_while(expr, stmt),
        }
    }
}
