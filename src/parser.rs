use std::fs::File;

use crate::{bytecode::Bytecode, lexer::{Lexer, Token}, value::Value};

pub struct ParseProto {
    pub constants: Vec<Value>,
    pub bytecodes: Vec<Bytecode>
}

pub fn load(input: File) -> ParseProto {
    let mut constants = Vec::new();
    let mut bytecodes = Vec::new();
    let mut lexer = Lexer::new(input);

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
    ParseProto { constants, bytecodes }
}