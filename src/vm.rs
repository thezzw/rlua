use std::{cell::RefCell, cmp::Ordering, collections::HashMap, io::Read, rc::Rc};
use crate::{bytecode::Bytecode, parser::ParseProto, value::{Value, Table}};

fn rs_print(state: &mut ExeState) -> i32 {
    println!("{}", state.stack[state.func_index + 1]);
    0
}

fn rs_dbg_print(state: &mut ExeState) -> i32 {
    println!("{:?}", state.stack[state.func_index + 1]);
    0
}

pub struct ExeState {
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    func_index: usize
}

impl ExeState {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert("print".to_string(), Value::Function(rs_print));
        globals.insert("dbg_print".to_string(), Value::Function(rs_dbg_print));

        Self {
            globals,
            stack: Vec::new(),
            func_index: 0
        }
    }

    pub fn execute<R: Read>(&mut self, proto: &ParseProto<R>) {
        for bytecode in &proto.bytecodes {
            match *bytecode {
                Bytecode::GetGlobal(stack_dst, const_idx) => {
                    let key: &str = (&proto.constants[const_idx as usize]).into();
                    let global_value = self.globals.get(key).unwrap_or(&Value::default()).clone();
                    self.set_stack(stack_dst, global_value);
                }
                Bytecode::SetGlobal(ident_idx, src) => {
                    let key = &proto.constants[ident_idx as usize];
                    let value = self.stack[src as usize].clone();
                    self.globals.insert(key.into(), value);
                }
                Bytecode::SetGlobalConst(ident_idx, src) => {
                    let key = &proto.constants[ident_idx as usize];
                    let value = proto.constants[src as usize].clone();
                    self.globals.insert(key.into(), value);
                }
                Bytecode::LoadConst(stack_dst, const_idx) => {
                    let const_value = proto.constants[const_idx as usize].clone();
                    self.set_stack(stack_dst, const_value);
                }
                Bytecode::LoadNil(dst, n) => {
                    self.fill_stack(dst as usize, n as usize);
                }
                Bytecode::LoadBool(dst, b) => {
                    self.set_stack(dst, Value::Boolean(b));
                }
                Bytecode::LoadInt(dst, i) => {
                    self.set_stack(dst, Value::Integer(i as i64));
                }
                Bytecode::Move(dst, src) => {
                    let v = self.stack[src as usize].clone();
                    self.set_stack(dst, v);
                }
                Bytecode::Call(func, _) => {
                    self.func_index = func as usize;
                    let func = &self.stack[self.func_index];
                    match func {
                        Value::Function(f) => {
                            f(self);
                        },
                        unknown => panic!("Expected function: {:?}", unknown)
                    }
                }
                Bytecode::NewTable(dst, narray, nmap) => {
                    let table = Table::new(narray as usize, nmap as usize);
                    self.set_stack(dst, Value::Table(Rc::new(RefCell::new(table))));
                }
                Bytecode::SetInt(t, i, v) => {
                    let value = self.stack[v as usize].clone();
                    self.set_table_int(t, i as i64, value);
                }
                Bytecode::GetInt(dst, t, k) => {
                    let value = self.get_table_int(t, k as i64);
                    self.set_stack(dst, value);
                }
                Bytecode::SetIntConst(t, i, v) => {
                    let value = proto.constants[v as usize].clone();
                    self.set_table_int(t, i as i64, value);
                }
                Bytecode::SetField(t, k, v) => {
                    let key = proto.constants[k as usize].clone();
                    let value = self.stack[v as usize].clone();
                    self.set_table(t, key, value);
                }
                Bytecode::SetFieldConst(t, k, v) => {
                    let key = proto.constants[k as usize].clone();
                    let value = proto.constants[v as usize].clone();
                    self.set_table(t, key, value);
                }
                Bytecode::SetTable(t, k, v) => {
                    let key = self.stack[k as usize].clone();
                    let value = self.stack[v as usize].clone();
                    self.set_table(t, key, value);
                }
                Bytecode::SetTableConst(t, k, v) => {
                    let key = self.stack[k as usize].clone();
                    let value: Value = proto.constants[v as usize].clone();
                    self.set_table(t, key, value);
                }
                Bytecode::SetList(table, tostore, nelems) => {
                    let ivalue = table as usize + 1;
                    if let Value::Table(table) = self.stack[table as usize].clone() {
                        let array = &mut table.borrow_mut().array;

                        let cur_size = array.len();
                        let new_size = cur_size + tostore as usize;
                        array.reserve(new_size);

                        let values = self.stack.drain(ivalue .. ivalue + tostore as usize);
                        assert_eq!(values.len(), tostore as usize);
                        for (i, v) in values.enumerate() {
                            set_vec(array, nelems as usize + i, v);
                        }
                    } else {
                        panic!("not table");
                    }
                }
                Bytecode::GetField(dst, t, k) => {
                    let key = &proto.constants[k as usize];
                    let value = self.get_table(t, key);
                    self.set_stack(dst, value);
                }
                Bytecode::GetTable(dst, t, k) => {
                    let key = &self.stack[k as usize];
                    let value = self.get_table(t, key);
                    self.set_stack(dst, value);
                }
            }
        }
    }

    fn set_stack(&mut self, dst: u8, value: Value) {
        let dst = dst as usize;
        match dst.cmp(&self.stack.len()) {
            std::cmp::Ordering::Less => self.stack[dst] = value,
            std::cmp::Ordering::Equal => self.stack.push(value),
            std::cmp::Ordering::Greater => panic!("Invalid stack index: {}", dst)
        }
    }

    fn fill_stack(&mut self, begin: usize, num: usize) {
        let end = begin + num;
        let len = self.stack.len();
        if begin < len {
            self.stack[begin .. len].fill(Value::Nil);
        }
        if end > len {
            self.stack.resize(end, Value::Nil);
        }
    }

    fn set_table(&mut self, t: u8, key: Value, value: Value) {
        match &key {
            Value::Integer(i) => self.set_table_int(t, *i, value), // TODO Float
            _ => self.do_set_table(t, key, value),
        }
    }

    fn set_table_int(&mut self, t: u8, i: i64, value: Value) {
        if let Value::Table(table) = &self.stack[t as usize] {
            let mut table = table.borrow_mut();
            // this is not same with Lua's official implement
            if i > 0 && (i < 4 || i < table.array.capacity() as i64 * 2) {
                set_vec(&mut table.array, i as usize - 1, value);
            } else {
                table.map.insert(Value::Integer(i), value);
            }
        } else {
            panic!("invalid table");
        }
    }

    fn do_set_table(&mut self, t: u8, key: Value, value: Value) {
        if let Value::Table(table) = &self.stack[t as usize] {
            table.borrow_mut().map.insert(key, value);
        } else {
            panic!("invalid table");
        }
    }

    fn get_table(&self, t: u8, key: &Value) -> Value {
        match key {
            Value::Integer(i) => self.get_table_int(t, *i), // TODO Float
            _ => self.do_get_table(t, key),
        }
    }

    fn get_table_int(&self, t: u8, i: i64) -> Value {
        if let Value::Table(table) = &self.stack[t as usize] {
            let table = table.borrow();
            table.array.get(i as usize - 1)
                .unwrap_or_else(|| table.map.get(&Value::Integer(i))
                    .unwrap_or(&Value::Nil)).clone()
        } else {
            panic!("set invalid table");
        }
    }

    fn do_get_table(&self, t: u8, key: &Value) -> Value {
        if let Value::Table(table) = &self.stack[t as usize] {
            let table = table.borrow();
            table.map.get(key).unwrap_or(&Value::Nil).clone()
        } else {
            panic!("set invalid table");
        }
    }
}

fn set_vec(vec: &mut Vec<Value>, i: usize, value: Value) {
    match i.cmp(&vec.len()) {
        Ordering::Less => vec[i] = value,
        Ordering::Equal => vec.push(value),
        Ordering::Greater => {
            vec.resize(i, Value::Nil);
            vec.push(value);
        }
    }
}