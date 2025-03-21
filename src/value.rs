use std::fmt;

use crate::vm::ExeState;

#[derive(Default, Clone)]
pub enum Value {
    Function(fn(&mut ExeState) -> i32),
    String(String),
    Number(f64),
    Boolean(bool),
    #[default]
    Nil
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Function(_) => write!(f, "Function"),
            Value::String(s) => write!(f, "String(\"{}\")", s),
            Value::Number(n) => write!(f, "Number({})", n),
            Value::Boolean(b) => write!(f, "Boolean({})", b),
            Value::Nil => write!(f, "Nil")
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Function(_) => write!(f, "Function"),
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil")
        }
    }
}