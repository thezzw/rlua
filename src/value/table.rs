use super::Value;
use std::collections::HashMap;

pub struct Table {
    pub array: Vec<Value>,
    pub map: HashMap<Value, Value>
}

impl Table {
    pub fn new(narray: usize, nmap: usize) -> Self {
        Table {
            array: Vec::with_capacity(narray),
            map: HashMap::with_capacity(nmap)
        }
    }
}