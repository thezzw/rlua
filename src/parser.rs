use crate::{bytecode::Bytecode, lexer::{Lexer, Token}, value::Value};

pub struct Parser {
    pub constants: Vec<Value>,
    pub bytecodes: Vec<Bytecode>
}

impl Parser {
    pub fn load(mut lexer: Lexer) -> Self {
        let mut constants = Vec::new();
        let mut bytecodes = Vec::new();
    
        loop {
            match lexer.next() {
                Token::Ident(ident) => {
                    constants.push(Value::String(ident));
                    bytecodes.push(Bytecode::GetGlobal(0, (constants.len() - 1) as u8));
    
                    if let Token::String(s) = lexer.next() {
                        constants.push(Value::String(s));
                        bytecodes.push(Bytecode::LoadConst(1, (constants.len() - 1) as u8));
                        bytecodes.push(Bytecode::Call(0, 1));
                    } else {
                        panic!("Expected string after ident.");
                    }
                }
                Token::Eos => break,
                _ => panic!("Unexpected token.")
            }
        }
    
        dbg!(&constants);
        dbg!(&bytecodes);
        Self { constants, bytecodes }
    }
}