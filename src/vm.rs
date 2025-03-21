use std::collections::HashMap;

use crate::{bytecode::Bytecode, parser::Parser, value::Value};

fn rs_print(state: &mut ExeState) -> i32 {
    println!("{}", state.stack[1]);
    0
}

pub struct ExeState {
    globals: HashMap<String, Value>,
    stack: Vec<Value>
}

impl ExeState {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert("print".to_string(), Value::Function(rs_print));

        Self {
            globals,
            stack: Vec::new()
        }
    }

    pub fn execute(&mut self, proto: &Parser) {
        for bytecode in &proto.bytecodes {
            match *bytecode {
                Bytecode::GetGlobal(stack_dst, const_idx) => {
                    match &proto.constants[const_idx as usize] {
                        Value::String(key) => {
                            let global_value = self.globals.get(key).unwrap_or(&Value::default()).clone();
                            self.set_stack(stack_dst, global_value);
                        },
                        unknown => panic!("Unexpected global key: {:?}", unknown)
                    };
                },
                Bytecode::LoadConst(stack_dst, const_idx) => {
                    let const_value = proto.constants[const_idx as usize].clone();
                    self.set_stack(stack_dst, const_value);
                },
                Bytecode::Call(func, _) => {
                    let func = &self.stack[func as usize];
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