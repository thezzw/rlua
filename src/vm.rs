use std::{collections::HashMap, io::Read};
use crate::{bytecode::Bytecode, parser::ParseProto, value::Value};

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
                },
                Bytecode::SetGlobal(ident_idx, src) => {
                    let key = &proto.constants[ident_idx as usize];
                    let value = self.stack[src as usize].clone();
                    self.globals.insert(key.into(), value);
                },
                Bytecode::SetGlobalConst(ident_idx, src) => {
                    let key = &proto.constants[ident_idx as usize];
                    let value = proto.constants[src as usize].clone();
                    self.globals.insert(key.into(), value);
                },
                Bytecode::SetGlobalGlobal(ident_idx, src) => {
                    let name = &proto.constants[ident_idx as usize];
                    let src: &str = (&proto.constants[src as usize]).into();
                    let value = self.globals.get(src).unwrap_or(&Value::Nil).clone();
                    self.globals.insert(name.into(), value);
                },
                Bytecode::LoadConst(stack_dst, const_idx) => {
                    let const_value = proto.constants[const_idx as usize].clone();
                    self.set_stack(stack_dst, const_value);
                },
                Bytecode::LoadNil(dst) => {
                    self.set_stack(dst, Value::Nil);
                },
                Bytecode::LoadBool(dst, b) => {
                    self.set_stack(dst, Value::Boolean(b));
                },
                Bytecode::LoadInt(dst, i) => {
                    self.set_stack(dst, Value::Integer(i as i64));
                },
                Bytecode::Move(dst, src) => {
                    let v = self.stack[src as usize].clone();
                    self.set_stack(dst, v);
                },
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
}