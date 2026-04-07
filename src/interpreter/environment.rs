use std::collections::HashMap;

use crate::interpreter::{RuntimeError, SeleneValue};

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    pub values: HashMap<String, SeleneValue>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosed(parent: Environment) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(parent)),
        }
    }

    pub fn get(&self, name: &str, line: u64) -> Result<SeleneValue, RuntimeError> {
        match self.values.get(name) {
            Some(v) => Ok(v.clone()),
            None => match &self.enclosing {
                Some(parent) => parent.get(name, line),
                None => Err(RuntimeError::new(
                    line,
                    format!("Variável '{}' não definida.", name),
                )),
            },
        }
    }

    pub fn define(&mut self, name: String, value: SeleneValue) {
        self.values.insert(name, value);
    }

    pub fn assign(
        &mut self,
        name: String,
        line: u64,
        value: SeleneValue,
    ) -> Result<SeleneValue, RuntimeError> {
        match self.values.get(&name) {
            Some(_v) => {
                let val = self.values.insert(name.clone(), value.clone());
                match val {
                    Some(_v) => Ok(value),
                    None => Err(RuntimeError::new(
                        line,
                        format!("Variável '{}' não definida.", name),
                    )),
                }
            }
            None => match &mut self.enclosing {
                Some(parent) => parent.assign(name, line, value),
                None => Err(RuntimeError::new(
                    line,
                    format!("Variável '{}' não definida.", name),
                )),
            },
        }
    }
}
