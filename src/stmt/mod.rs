use crate::expr::Expression;

pub trait StmtVisitor {
    type Output;
    fn visit_print(&mut self, expr: &Expression) -> Self::Output;
    fn visit_expr_statement(&mut self, expr: &Expression) -> Self::Output;
    fn visit_var(&mut self, name: &String, expr: Option<&Expression>, line: u64) -> Self::Output;
    fn visit_block(&mut self, statements: &Vec<Statement>) -> Self::Output;
    fn visit_if(
        &mut self,
        expr: &Expression,
        stmt: &Statement,
        else_stmt: Option<&Statement>,
    ) -> Self::Output;
    fn visit_while(&mut self, expr: &Expression, stmt: &Statement) -> Self::Output;
    fn visit_function(
        &mut self,
        name: &String,
        params: &Vec<String>,
        stmts: &Vec<Statement>,
        line: u64,
    ) -> Self::Output;
    fn visit_return(&mut self, line: u64, value: Option<&Expression>) -> Self::Output;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Print(Expression),
    ExprStatement(Expression),
    Var(String, Option<Expression>, u64),
    Block(Vec<Statement>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    Function(String, Vec<String>, Vec<Statement>, u64),
    Return(u64, Option<Expression>),
}

impl Statement {
    pub fn accept<V: StmtVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Statement::ExprStatement(expr) => visitor.visit_expr_statement(expr),
            Statement::Print(expr) => visitor.visit_print(expr),
            Statement::Var(name, expr, line) => visitor.visit_var(name, expr.as_ref(), *line),
            Statement::Block(statements) => visitor.visit_block(statements),
            Statement::If(expr, stmt, else_stmt) => {
                visitor.visit_if(expr, stmt, else_stmt.as_deref())
            }
            Statement::While(expr, stmt) => visitor.visit_while(expr, stmt),
            Statement::Function(name, params, stmts, line) => visitor.visit_function(name, params, stmts, *line),
            Statement::Return(line, value) => visitor.visit_return(*line, value.as_ref()),
        }
    }
}
