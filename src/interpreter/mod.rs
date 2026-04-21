mod environment;
mod values;
use crate::{
    expr::{ExprVisitor, Expression},
    interpreter::{
        environment::Environment,
        values::{ClockFn, SeleneClass, SeleneFunction, SeleneInstance},
    },
    stmt::{Statement, StmtVisitor},
    token::{BinaryOp, LogicalOp, TokenLiteral, UnaryOp},
};
use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, fmt::Debug};
pub use values::{RuntimeError, SeleneValue};

pub trait SeleneCallable: Debug {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<SeleneValue>,
    ) -> Result<SeleneValue, RuntimeError>;
    fn name(&self) -> String;
}

pub struct Interpreter {
    pub locals: HashMap<usize, usize>,
    pub globals: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
}

impl ExprVisitor for Interpreter {
    type Output = Result<SeleneValue, RuntimeError>;

    fn visit_binary(
        &mut self,
        left: &Expression,
        operator: &BinaryOp,
        line: &u64,
        right: &Expression,
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
                    (SeleneValue::Number(_), _) => Err(RuntimeError::Error {
                        line: *line,
                        message: "Operando direito do '+' deve ser um número.".to_string(),
                    }),
                    (SeleneValue::String(_), _) => Err(RuntimeError::Error {
                        line: *line,
                        message: "Operando direito do '+' deve ser uma string.".to_string(),
                    }),
                    _ => Err(RuntimeError::Error {
                        line: *line,
                        message: "Operandos do '+' devem ser dois números ou duas strings."
                            .to_string(),
                    }),
                }
            }
            BinaryOp::Minus => {
                let (a, b) = self.extract_numbers(left, right, *line, "-")?;
                Ok(SeleneValue::Number(a - b))
            }
            BinaryOp::Slash => {
                let (a, b) = self.extract_numbers(left, right, *line, "/")?;
                if b == 0.0 {
                    return Err(RuntimeError::Error {
                        line: *line,
                        message: "Não é possível realizar uma divisão por zero.".to_string(),
                    });
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

    fn visit_literal(&mut self, literal: &TokenLiteral) -> Self::Output {
        match literal {
            TokenLiteral::Number(n) => Ok(SeleneValue::Number(*n)),
            TokenLiteral::Boolean(b) => Ok(SeleneValue::Boolean(*b)),
            TokenLiteral::StringLiteral(s) => Ok(SeleneValue::String(s.clone())),
            TokenLiteral::Null => Ok(SeleneValue::Null),
        }
    }

    fn visit_unary(&mut self, unary_op: &UnaryOp, line: &u64, expr: &Expression) -> Self::Output {
        match unary_op {
            UnaryOp::Minus => {
                let value = self.evaluate(expr)?;
                match value {
                    SeleneValue::Number(n) => Ok(SeleneValue::Number(-n)),
                    _ => Err(RuntimeError::Error {
                        line: *line,
                        message: "Operando deve ser um número.".to_string(),
                    }),
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

    fn visit_variable(&mut self, name: &String, line: u64, id: usize) -> Self::Output {
        match self.locals.get(&id) {
            Some(d) => Environment::get_at(&self.environment, *d, name.clone(), line),
            None => Environment::get(&self.globals, name, line),
        }
    }

    fn visit_assign(
        &mut self,
        name: &String,
        line: u64,
        expr: &Expression,
        id: usize,
    ) -> Self::Output {
        let value = self.evaluate(expr)?;

        match self.locals.get(&id) {
            Some(d) => Environment::assign_at(&self.environment, *d, name.clone(), value, line),
            None => Environment::assign(&self.globals, name.clone(), line, value),
        }
    }

    fn visit_logical(
        &mut self,
        left: &Expression,
        operator: &LogicalOp,
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

    fn visit_call(
        &mut self,
        callee: &Expression,
        args: &Vec<Expression>,
        paren: &crate::token::Token,
    ) -> Self::Output {
        let callee_value = self.evaluate(callee)?;
        match callee_value {
            SeleneValue::Function(func) => {
                let mut evaluated_args: Vec<SeleneValue> = Vec::new();
                for arg in args {
                    evaluated_args.push(arg.accept(self)?);
                }
                if evaluated_args.len() != func.arity() {
                    return Err(RuntimeError::Error {
                        line: paren.line,
                        message: format!(
                            "Esperava {} argumento(s), mas recebeu {}.",
                            func.arity(),
                            evaluated_args.len()
                        ),
                    });
                }
                func.call(self, evaluated_args)
            }
            SeleneValue::Class(class) => {
                let selene_instance = SeleneInstance {
                    class: Rc::clone(&class),
                    fields: HashMap::new(),
                };

                let instance_rc = Rc::new(RefCell::new(selene_instance));
                let instance_value = SeleneValue::Instance(Rc::clone(&instance_rc));
                let init_func = class.functions.get("init").cloned();

                match init_func {
                    Some(fun) => {
                        let mut evaluated_args: Vec<SeleneValue> = Vec::new();
                        for arg in args {
                            evaluated_args.push(arg.accept(self)?);
                        }
                        if evaluated_args.len() != fun.arity() {
                            return Err(RuntimeError::Error {
                                line: paren.line,
                                message: format!(
                                    "Esperava {} argumento(s), mas recebeu {}.",
                                    fun.arity(),
                                    evaluated_args.len()
                                ),
                            });
                        }
                        let env: Rc<RefCell<Environment>> =
                            Environment::new_enclosed(Rc::clone(&fun.closure));

                        Environment::define(&env, "this".to_string(), instance_value.clone());
                        let bound_func = SeleneFunction {
                            name: fun.name.clone(),
                            params: fun.params.clone(),
                            body: fun.body.clone(),
                            closure: env,
                            is_initializer: true,
                        };
                        bound_func.call(self, evaluated_args)?;

                        Ok(instance_value)
                    }
                    None => Ok(SeleneValue::Instance(instance_rc)),
                }
            }
            _ => Err(RuntimeError::Error {
                line: paren.line,
                message: format!(
                    "Expressão '{}' não é uma função e não pode ser chamada.",
                    paren.lexeme
                ),
            }),
        }
    }

    fn visit_get(&mut self, expr: &Expression, token: &crate::token::Token) -> Self::Output {
        let expr = expr.accept(self)?;
        match expr {
            SeleneValue::Instance(instance) => {
                let borrowed = instance.borrow();
                match borrowed.fields.get(&token.lexeme) {
                    Some(i) => return Ok(i.clone()),
                    None => match borrowed.class.functions.get(&token.lexeme) {
                        Some(func) => {
                            let env = Environment::new_enclosed(Rc::clone(&func.closure));
                            Environment::define(
                                &env,
                                "this".to_string(),
                                SeleneValue::Instance(Rc::clone(&instance)),
                            );
                            let new_func = SeleneFunction {
                                name: func.name.clone(),
                                params: func.params.clone(),
                                body: func.body.clone(),
                                closure: env,
                                is_initializer: func.is_initializer,
                            };
                            return Ok(SeleneValue::Function(Rc::new(new_func)));
                        }
                        None => Err(RuntimeError::Error {
                            line: token.line,
                            message: format!(
                                "Expressão '{}' não é uma função e não pode ser chamada.",
                                token.lexeme
                            ),
                        }),
                    },
                }
            }
            _ => Err(RuntimeError::Error {
                line: token.line,
                message: format!("Expressão '{}' não é uma instancia válida: ", token.lexeme),
            }),
        }
    }

    fn visit_set(
        &mut self,
        expr: &Expression,
        token: &crate::token::Token,
        value: &Expression,
    ) -> Self::Output {
        let expr = expr.accept(self)?;
        match expr {
            SeleneValue::Instance(instance) => {
                let value = value.accept(self)?;
                instance
                    .borrow_mut()
                    .fields
                    .insert(token.lexeme.clone(), value.clone());
                Ok(value)
            }
            _ => Err(RuntimeError::Error {
                line: token.line,
                message: format!("Expressão '{}' não é uma instancia válida: ", token.lexeme),
            }),
        }
    }

    fn visit_this(&mut self, token: &crate::token::Token, id: usize) -> Self::Output {
        match self.locals.get(&id) {
            Some(d) => Environment::get_at(&self.environment, *d, token.lexeme.clone(), token.line),
            None => Environment::get(&self.globals, &token.lexeme, token.line),
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

    fn visit_var(&mut self, name: &String, expr: Option<&Expression>, _line: u64) -> Self::Output {
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

    fn visit_function(
        &mut self,
        name: &String,
        params: &Vec<String>,
        stmts: &Vec<Statement>,
        _line: u64,
    ) -> Self::Output {
        let func = SeleneFunction {
            name: name.clone(),
            params: params.clone(),
            body: stmts.clone(),
            closure: Rc::clone(&self.environment),
            is_initializer: false,
        };
        Environment::define(
            &self.environment,
            name.clone(),
            SeleneValue::Function(Rc::new(func)),
        );
        Ok(())
    }

    fn visit_return(&mut self, _line: u64, value: Option<&Expression>) -> Self::Output {
        match value {
            Some(val) => match val.accept(self) {
                Ok(v) => Err(RuntimeError::Return(v)),
                Err(err) => Err(err),
            },
            None => Err(RuntimeError::Return(SeleneValue::Null)),
        }
    }

    fn visit_class(
        &mut self,
        name: &String,
        _line: u64,
        statements: &Vec<Statement>,
    ) -> Self::Output {
        let mut functions = HashMap::new();
        Environment::define(&self.environment, name.clone(), SeleneValue::Null);
        for stmt in statements {
            match stmt {
                Statement::Function(n, params, statements, _line) => {
                    let is_initializer = n == "init";
                    let selene_function = SeleneFunction {
                        name: n.clone(),
                        params: params.clone(),
                        body: statements.clone(),
                        closure: Rc::clone(&self.environment),
                        is_initializer: is_initializer,
                    };
                    functions.insert(n.clone(), selene_function);
                }
                _ => {}
            }
        }
        let selene_class = SeleneClass {
            name: name.clone(),
            functions: functions,
        };

        Environment::define(
            &self.environment,
            name.clone(),
            SeleneValue::Class(Rc::new(selene_class)),
        );
        Ok(())
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Environment::new();
        Environment::define(
            &globals,
            "clock".to_string(),
            SeleneValue::Function(Rc::new(ClockFn)),
        );
        let environment = Rc::clone(&globals);
        Interpreter {
            locals: HashMap::new(),
            globals,
            environment,
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
            _ => Err(RuntimeError::Error {
                line,
                message: format!("Operandos do '{}' devem ser números.", operator),
            }),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) {
        for stmt in statements {
            match stmt.accept(self) {
                Ok(_) => {}
                Err(RuntimeError::Error { line, message }) => {
                    println!("Erro na linha {}: {}", line, message)
                }
                Err(RuntimeError::Return(_)) => {}
            }
        }
    }

    pub fn execute_block(
        &mut self,
        env: Rc<RefCell<Environment>>,
        statements: &Vec<Statement>,
    ) -> Result<(), RuntimeError> {
        let previous = Rc::clone(&self.environment);
        self.environment = env;

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

    pub fn resolve(&mut self, id: usize, depth: usize) {
        self.locals.insert(id, depth);
    }
}
