use std::fmt;

use crate::vm::ExeState;

#[derive(Default, Clone)]
pub enum Value {
    Function(fn(&mut ExeState) -> i32),
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    #[default]
    Nil
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Function(_) => write!(f, "Function"),
            Value::String(s) => write!(f, "String(\"{}\")", s),
            Value::Integer(n) => write!(f, "Integer({})", n),
            Value::Float(n) => write!(f, "Float({})", n),
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
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil")
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Function(_), Value::Function(_)) => true,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Integer(n1), Value::Integer(n2)) => n1 == n2,
            (Value::Float(n1), Value::Float(n2)) => n1 == n2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Nil, Value::Nil) => true,
            _ => false
        }
    }
}