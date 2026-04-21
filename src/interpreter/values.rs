use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    interpreter::{SeleneCallable, environment::Environment},
    stmt::Statement,
};

#[derive(Debug, Clone)]
pub enum SeleneValue {
    Number(f64),
    Boolean(bool),
    String(String),
    Null,
    Function(Rc<dyn SeleneCallable>),
    Class(Rc<SeleneClass>),
    Instance(Rc<RefCell<SeleneInstance>>),
}
#[derive(Debug, Clone)]
pub struct SeleneClass {
    pub name: String,
    pub functions: HashMap<String, SeleneFunction>,
}

#[derive(Debug, Clone)]
pub struct SeleneInstance {
    pub class: Rc<SeleneClass>,
    pub fields: HashMap<String, SeleneValue>,
}

#[derive(Debug, Clone)]
pub struct SeleneFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub closure: Rc<RefCell<Environment>>,
    pub is_initializer: bool,
}

impl SeleneValue {
    pub fn to_display(&self) -> String {
        match self {
            SeleneValue::Number(n) => {
                if *n == n.floor() {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            SeleneValue::Boolean(b) => format!("{}", b),
            SeleneValue::String(s) => s.clone(),
            SeleneValue::Null => "null".to_string(),
            SeleneValue::Function(f) => format!("<fn {}>", f.name()),
            SeleneValue::Class(selene_class) => format!("{}", selene_class.name),
            SeleneValue::Instance(selene_instance) => {
                format!("{} instance", selene_instance.borrow().class.name)
            }
        }
    }
}

impl PartialEq for SeleneValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SeleneValue::Number(a), SeleneValue::Number(b)) => a == b,
            (SeleneValue::Boolean(a), SeleneValue::Boolean(b)) => a == b,
            (SeleneValue::String(a), SeleneValue::String(b)) => a == b,
            (SeleneValue::Null, SeleneValue::Null) => true,
            (SeleneValue::Function(_), SeleneValue::Function(_)) => false,
            (SeleneValue::Class(a), SeleneValue::Class(b)) => a.name == b.name,
            (SeleneValue::Instance(_), SeleneValue::Instance(_)) => false,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    Error { line: u64, message: String },
    Return(SeleneValue),
}

#[derive(Debug)]
pub struct ClockFn;

impl PartialEq for SeleneFunction {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl SeleneCallable for SeleneFunction {
    fn arity(&self) -> usize {
        return self.params.len();
    }

    fn call(
        &self,
        interpreter: &mut super::Interpreter,
        args: Vec<SeleneValue>,
    ) -> Result<SeleneValue, RuntimeError> {
        let child = Environment::new_enclosed(Rc::clone(&self.closure));
        for (param, arg) in self.params.iter().zip(args.iter()) {
            Environment::define(&child, param.clone(), arg.clone());
        }
        match interpreter.execute_block(child, &self.body) {
            Ok(_) => {
                if self.is_initializer {
                    Environment::get(&self.closure, "this", 0)
                } else {
                    Ok(SeleneValue::Null)
                }
            }
            Err(RuntimeError::Return(value)) => Ok(value),
            Err(e) => Err(e),
        }
    }

    fn name(&self) -> String {
        return self.name.clone();
    }
}

impl SeleneCallable for ClockFn {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut super::Interpreter,
        _args: Vec<SeleneValue>,
    ) -> Result<SeleneValue, RuntimeError> {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Ok(SeleneValue::Number(secs))
    }

    fn name(&self) -> String {
        "clock".to_string()
    }
}
