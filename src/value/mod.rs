mod table;

use std::{cell::RefCell, fmt, hash::{Hash, Hasher}, mem, rc::Rc};
use crate::vm::ExeState;
pub use table::Table;

const SHORT_STR_MAX: usize = 14;
const MID_STR_MAX: usize = 48 - 1;

#[derive(Default, Clone)]
pub enum Value {
    Function(fn(&mut ExeState) -> i32),
    Table(Rc<RefCell<Table>>),
    ShortString(u8, [u8; SHORT_STR_MAX]),
    MidString(Rc<(u8, [u8; MID_STR_MAX])>),
    LongString(Rc<Vec<u8>>),
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
            Value::Table(t) => {
                let t = t.borrow();
                let mut map_content = String::new();
                let mut array_content = String::new();
                for (i, v) in t.array.iter().enumerate() {
                    array_content += format!("\t[{}] = {}\n", i + 1, v).as_str();
                }
                for (k, v) in t.map.iter() {
                    map_content += format!("\t[{}] = {}\n", k, v).as_str();
                }
                write!(f, "Table(\narray[{}]:\n{}\nmap[{}]:\n{})", t.array.len(), array_content, t.map.len(), map_content)
            },
            Value::LongString(s) => {
                let s = String::from_utf8_lossy(s);
                write!(f, "LongString({})", s)
            },
            Value::ShortString(len, s) => {
                let s = String::from_utf8_lossy(&s[..*len as usize]);
                write!(f, "ShortString({})", s)
            },
            Value::MidString(s) => {
                let s = String::from_utf8_lossy(&s.1[..s.0 as usize]);
                write!(f, "MidString({})", s)
            }
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
            Value::Table(t) => {
                let t = t.borrow();
                let mut map_content = String::new();
                let mut array_content = String::new();
                for (i, v) in t.array.iter().enumerate() {
                    array_content += format!("[{}] = {}, ", i + 1, v).as_str();
                }
                for (k, v) in t.map.iter() {
                    map_content += format!("[{}] = {}, ", k, v).as_str();
                }
                write!(f, "{{{}{}}}", array_content, map_content)
            },
            Value::LongString(s) => {
                let s = String::from_utf8_lossy(s);
                write!(f, "\"{}\"", s)
            },
            Value::ShortString(len, s) => {
                let s = String::from_utf8_lossy(&s[..*len as usize]);
                write!(f, "\"{}\"", s)
            },
            Value::MidString(s) => {
                let s = String::from_utf8_lossy(&s.1[..s.0 as usize]);
                write!(f, "\"{}\"", s)
            }
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
            (Value::Table(t1), Value::Table(t2)) => Rc::ptr_eq(t1, t2),
            (Value::LongString(s1), Value::LongString(s2)) => s1 == s2,
            (Value::ShortString(len1, s1), Value::ShortString(len2, s2)) => {
                if len1 != len2 { return false; }
                let s1 = String::from_utf8_lossy(&s1[..*len1 as usize]);
                let s2 = String::from_utf8_lossy(&s2[..*len2 as usize]);
                s1 == s2
            },
            (Value::MidString(s1), Value::MidString(s2)) => {
                if s1.0 != s2.0 { return false; }
                let s1 = String::from_utf8_lossy(&s1.1[..s1.0 as usize]);
                let s2 = String::from_utf8_lossy(&s2.1[..s2.0 as usize]);
                s1 == s2
            }
            (Value::Integer(n1), Value::Integer(n2)) => n1 == n2,
            (Value::Float(n1), Value::Float(n2)) => n1 == n2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Nil, Value::Nil) => true,
            _ => false
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => (),
            Value::Boolean(b) => b.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => unsafe {
                mem::transmute_copy::<f64, i64>(f).hash(state)
            }
            Value::ShortString(len, buf) => buf[..*len as usize].hash(state),
            Value::MidString(s) => s.1[..s.0 as usize].hash(state),
            Value::LongString(s) => s.hash(state),
            Value::Table(t) => Rc::as_ptr(t).hash(state),
            Value::Function(f) => (*f as *const usize).hash(state),
        }
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        vec_to_short_mid_str(v).unwrap_or(Value::LongString(Rc::new(v.to_vec())))
    }
}
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        s.as_bytes().into() // &[u8]
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        vec_to_short_mid_str(&v).unwrap_or(Value::LongString(Rc::new(v)))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        s.into_bytes().into() // Vec<u8>
    }
}

fn vec_to_short_mid_str(v: &[u8]) -> Option<Value> {
    let len = v.len();
    if len <= SHORT_STR_MAX {
        let mut buf = [0; SHORT_STR_MAX];
        buf[..len].copy_from_slice(&v);
        Some(Value::ShortString(len as u8, buf))

    } else if len <= MID_STR_MAX {
        let mut buf = [0; MID_STR_MAX];
        buf[..len].copy_from_slice(&v);
        Some(Value::MidString(Rc::new((len as u8, buf))))

    } else {
        None
    }
}

impl<'a> From<&'a Value> for &'a [u8] {
    fn from(v: &'a Value) -> Self {
        match v {
            Value::ShortString(len, buf) => &buf[..*len as usize],
            Value::MidString(s) => &s.1[..s.0 as usize],
            Value::LongString(s) => s,
            _ => panic!("invalid string Value"),
        }
    }
}

impl<'a> From<&'a Value> for &'a str {
    fn from(v: &'a Value) -> Self {
        std::str::from_utf8(v.into()).unwrap()
    }
}

impl From<&Value> for String {
    fn from(v: &Value) -> Self {
        String::from_utf8_lossy(v.into()).to_string()
    }
}