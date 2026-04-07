#[derive(Debug, Clone, PartialEq)]
pub enum SeleneValue {
    Number(f64),
    Boolean(bool),
    String(String),
    Null,
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub line: u64,
    pub message: String,
}

impl RuntimeError {
    pub fn new(line: u64, message: String) -> RuntimeError {
        RuntimeError { line, message }
    }
}