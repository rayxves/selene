use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::interpreter::{ RuntimeError, SeleneValue};

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, SeleneValue>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn new_enclosed(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: Some(parent),
        }))
    }

    pub fn get(
        env: &Rc<RefCell<Environment>>,
        name: &str,
        line: u64,
    ) -> Result<SeleneValue, RuntimeError> {
        let borrowed = env.borrow();
        match borrowed.values.get(name) {
            Some(v) => Ok(v.clone()),
            None => match &borrowed.enclosing {
                Some(parent) => Environment::get(parent, name, line),
                None => Err(RuntimeError::Error {
                    line,
                    message: format!("Variável '{}' não definida.", name),
                }),
            },
        }
    }

    pub fn define(env: &Rc<RefCell<Environment>>, name: String, value: SeleneValue) {
        env.borrow_mut().values.insert(name, value);
    }

    pub fn assign(
        env: &Rc<RefCell<Environment>>,
        name: String,
        line: u64,
        value: SeleneValue,
    ) -> Result<SeleneValue, RuntimeError> {
        let mut borrowed = env.borrow_mut();
        if borrowed.values.contains_key(&name) {
            borrowed.values.insert(name, value.clone());
            Ok(value)
        } else {
            match borrowed.enclosing.clone() {
                Some(parent) => {
                    drop(borrowed);
                    Environment::assign(&parent, name, line, value)
                }
                None => Err(RuntimeError::Error {
                    line,
                    message: format!("Variável '{}' não definida.", name),
                }),
            }
        }
    }

    pub fn ancestor(
        env: &Rc<RefCell<Environment>>,
        depth: usize,
    ) -> Rc<RefCell<Environment>> {
        let mut current = Rc::clone(env);
        let mut counter = depth;
        while counter > 0 {
            let next = current.borrow().enclosing.as_ref().unwrap().clone();
            current = next;
            counter -= 1;
        }

        current
    }

    pub fn get_at(
        env: &Rc<RefCell<Environment>>,
        depth: usize,
        name: String,
        line: u64,
    ) -> Result<SeleneValue, RuntimeError> {
        let env = Environment::ancestor(env, depth);
        Environment::get(&env, &name, line)
    }

    pub fn assign_at(
        env: &Rc<RefCell<Environment>>,
        depth: usize,
        name: String,
        value: SeleneValue,
        line: u64,
    ) -> Result<SeleneValue, RuntimeError> {
        let env = Environment::ancestor(env, depth);
        Environment::assign(&env, name, line, value)
    }
}
