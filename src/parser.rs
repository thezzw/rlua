use crate::{bytecode::Bytecode, lexer::{Lexer, Token}, value::Value};

pub struct ParseProto {
    pub constants: Vec<Value>,
    pub bytecodes: Vec<Bytecode>,

    locals: Vec<String>,
    lexer: Lexer,
}

impl ParseProto {
    pub fn load(lexer: Lexer) -> Self {
        let mut proto = Self {
            constants: Vec::new(),
            bytecodes: Vec::new(),
            locals: Vec::new(),
            lexer,
        };
        proto.chunk();
        
        println!("constants: {:?}", &proto.constants);
        println!("bytecodes: {:?}", &proto.bytecodes);
        proto
    }

    fn chunk(&mut self) {
        loop {
            match self.lexer.next() {
                Token::Ident(ident) => {
                    if self.lexer.peek() == &Token::Assign {
                        self.assignment(ident);
                    } else {
                        self.call_function(ident);
                    }
                }
                Token::Local => self.local(),
                Token::Eos => break,
                _ => panic!("Unexpected token.")
            }
        }
    }

    fn assignment(&mut self, var: String) {
        self.lexer.next(); // consume '='

        if let Some(i) = self.get_local(&var) {
            self.load_exp(i);
        } else {
            let dst = self.add_const(Value::String(var)) as u8;
            let bytecode = match self.lexer.next() {
                Token::Nil => Bytecode::SetGlobalConst(dst, self.add_const(Value::Nil) as u8),
                Token::True => Bytecode::SetGlobalConst(dst, self.add_const(Value::Boolean(true)) as u8),
                Token::False => Bytecode::SetGlobalConst(dst, self.add_const(Value::Boolean(false)) as u8),
                Token::Float(f) => Bytecode::SetGlobalConst(dst, self.add_const(Value::Float(f)) as u8),
                Token::Integer(i) => Bytecode::SetGlobalConst(dst, self.add_const(Value::Integer(i)) as u8),
                Token::String(s) => Bytecode::SetGlobalConst(dst, self.add_const(Value::String(s)) as u8),
                Token::Ident(var) => {
                    if let Some(i) = self.get_local(&var) {
                        Bytecode::SetGlobal(dst, i as u8)
                    } else {
                        Bytecode::SetGlobalGlobal(dst, self.add_const(Value::String(var)) as u8)
                    }
                },
                _ => panic!("unexpected token"),
            };
            self.bytecodes.push(bytecode);
        }
    }

    fn call_function(&mut self, name: String) {
        let ifunc = self.locals.len();
        let iarg = ifunc + 1;

        let code = self.load_var(ifunc, name);
        self.bytecodes.push(code);

        match self.lexer.next() {
            Token::ParL => {
                self.load_exp(iarg);
                if self.lexer.next() != Token::ParR {
                    panic!("expected `)`");
                }
            },
            Token::String(s) => {
                let code = self.load_const(iarg, Value::String(s));
                self.bytecodes.push(code);
            },
            _ => panic!("expected `(` or string")
        }

        self.bytecodes.push(Bytecode::Call(ifunc as u8, 1));
    }

    fn local(&mut self) {
        let ident = if let Token::Ident(ident) = self.lexer.next() {
            ident
        } else {
            panic!("expected variable");
        };

        if self.lexer.next() != Token::Assign {
            panic!("expected `=`");
        }

        self.load_exp(self.locals.len());

        // add to locals after load_exp()
        self.locals.push(ident);
    }

    fn add_const(&mut self, c: Value) -> usize {
        let constants = &mut self.constants;
        constants.iter().position(|v| v == &c).unwrap_or_else(|| {
            constants.push(c);
            constants.len() - 1
        })
    }

    fn load_const(&mut self, dst: usize, c: Value) -> Bytecode {
        Bytecode::LoadConst(dst as u8, self.add_const(c) as u16)
    }

    fn load_var(&mut self, dst: usize, var: String) -> Bytecode {
        if let Some(idx) = self.get_local(&var) {
            Bytecode::Move(dst as u8, idx as u8)
        } else {
            let idx = self.add_const(Value::String(var));
            Bytecode::GetGlobal(dst as u8, idx as u8)
        }
    }

    fn get_local(&self, name: &str) -> Option<usize> {
        self.locals.iter().rposition(|v| v == name)
    }

    fn load_exp(&mut self, dst: usize) {
        let bytecode = match self.lexer.next() {
            Token::Nil => Bytecode::LoadNil(dst as u8),
            Token::True => Bytecode::LoadBool(dst as u8, true),
            Token::False => Bytecode::LoadBool(dst as u8, false),
            Token::Integer(i) => if let Ok(i) = i16::try_from(i) {
                Bytecode::LoadInt(dst as u8, i)
            } else {
                self.load_const(dst, Value::Integer(i))
            },
            Token::Float(f) => self.load_const(dst, Value::Float(f)),
            Token::String(s) => self.load_const(dst, Value::String(s)),
            Token::Ident(var) => self.load_var(dst, var),
            _ => panic!("unexpected token")
        };
        self.bytecodes.push(bytecode);
    }
}