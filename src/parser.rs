use std::io::Read;
use crate::{bytecode::Bytecode, lexer::{Lexer, Token}, value::Value};

#[derive(Debug, PartialEq)]
enum ExpDesc {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(Vec<u8>),
    Local(usize),
    Global(usize),
    Index(usize, usize),
    IndexField(usize, usize),
    IndexInt(usize, u8),
    Call
}

enum ConstStack {
    Const(usize),
    Stack(usize)
}

pub struct ParseProto<R: Read> {
    pub constants: Vec<Value>,
    pub bytecodes: Vec<Bytecode>,

    sp: usize,
    locals: Vec<String>,
    lexer: Lexer<R>,
}

impl<R: Read> ParseProto<R> {
    pub fn load(lexer: Lexer<R>) -> Self {
        let mut proto = Self {
            constants: Vec::new(),
            bytecodes: Vec::new(),
            sp: 0,
            locals: Vec::new(),
            lexer,
        };
        proto.chunk();
        
        println!("constants: {:?}", &proto.constants);
        println!("bytecodes: {:?}", &proto.bytecodes);
        proto
    }

    fn chunk(&mut self) {
        self.block();
    }

    fn block(&mut self) {
        loop {
            match self.lexer.next() {
                Token::SemiColon => continue,
                t@Token::Ident(_) | t@Token::ParL => {
                    let desc = self.prefixexp(t);
                    if desc == ExpDesc::Call {
                    } else {
                        self.assignment(desc);
                    }
                }
                Token::Local => self.local(),
                Token::Eos => break,
                Token::Nil => continue,
                t => panic!("Unexpected token: {:?}", t)
            }
        }
    }

    fn local(&mut self) {
        let mut vars = Vec::new();
        let nexp = loop {
            vars.push(self.read_name());
            match self.lexer.peek() {
                Token::Comma => {
                    self.lexer.next();
                    continue;
                }
                Token::Assign => {
                    self.lexer.next();
                    break self.explist();
                }
                _ => break 0,
            }
        };

        if nexp < vars.len() {
            let ivar = self.locals.len() + nexp;
            let nnil = vars.len() - nexp;
            self.bytecodes.push(Bytecode::LoadNil(ivar as u8, nnil as u8));
        }

        self.locals.append(&mut vars);
    }

    fn assignment(&mut self, first_var: ExpDesc) {
        let mut vars = vec![first_var];
        loop {
            match self.lexer.next() {
                Token::Comma => {
                    let token = self.lexer.next();
                    vars.push(self.prefixexp(token));
                }
                Token::Assign => break,
                t => panic!("unexpected token: {:?}", t)
            }
        }

        let exp_sp0 = self.sp;
        let mut nfexp = 0;
        let last_exp = loop {
            let desc = self.exp();

            if self.lexer.peek() == &Token::Comma {
                self.lexer.next();
                self.discharge(exp_sp0 + nfexp, desc);
                nfexp += 1;
            } else {
                break desc;
            }
        };

        match (nfexp + 1).cmp(&vars.len()) {
            std::cmp::Ordering::Less => {
                todo!("handle less expressions than variables");
            }
            std::cmp::Ordering::Equal => {
                let lask_var = vars.pop().unwrap();
                self.assign_var(lask_var, last_exp);
            }
            std::cmp::Ordering::Greater => {
                nfexp = vars.len()
            }
        }

        while let Some(var) = vars.pop() {
            nfexp -= 1;
            self.assign_from_stack(var, exp_sp0 + nfexp);
        }
    }

    fn assign_var(&mut self, var: ExpDesc, value: ExpDesc) {
        if let ExpDesc::Local(i) = var {
            self.discharge(i, value);
        } else {
            match self.discharge_const(value) {
                ConstStack::Const(i) => self.assign_from_const(var, i),
                ConstStack::Stack(i) => self.assign_from_stack(var, i),
            }
        }
    }

    fn assign_from_stack(&mut self, var: ExpDesc, value: usize) {
        let code = match var {
            ExpDesc::Local(i) => Bytecode::Move(i as u8, value as u8),
            ExpDesc::Global(i) => Bytecode::SetGlobal(i as u8, value as u8),
            ExpDesc::Index(t, k) => Bytecode::SetTable(t as u8, k as u8, value as u8),
            ExpDesc::IndexField(t, k) => Bytecode::SetField(t as u8, k as u8, value as u8),
            ExpDesc::IndexInt(t, k) => Bytecode::SetInt(t as u8, k, value as u8),
            _ => panic!("assign from stack"),
        };
        self.bytecodes.push(code);
    }

    fn assign_from_const(&mut self, var: ExpDesc, value: usize) {
        let code = match var {
            ExpDesc::Global(i) => Bytecode::SetGlobalConst(i as u8, value as u8),
            ExpDesc::Index(t, k) => Bytecode::SetTableConst(t as u8, k as u8, value as u8),
            ExpDesc::IndexField(t, k) => Bytecode::SetFieldConst(t as u8, k as u8, value as u8),
            ExpDesc::IndexInt(t, k) => Bytecode::SetIntConst(t as u8, k, value as u8),
            _ => panic!("assign from const"),
        };
        self.bytecodes.push(code);
    }

    fn add_const(&mut self, c: impl Into<Value>) -> usize {
        let c = c.into();
        let constants = &mut self.constants;
        constants.iter().position(|v| v == &c).unwrap_or_else(|| {
            constants.push(c);
            constants.len() - 1
        })
    }

    fn explist(&mut self) -> usize {
        let mut n = 0;
        let sp0 = self.sp;
        loop {
            let desc = self.exp();
            self.discharge(sp0 + n, desc);

            n += 1;
            if self.lexer.peek() == &Token::Comma {
                self.lexer.next();
            } else {
                break n;
            }
        }
    }

    fn exp(&mut self) -> ExpDesc {
        let ahead = self.lexer.next();
        self.exp_with_ahead(ahead)
    }

    fn exp_with_ahead(&mut self, ahead: Token) -> ExpDesc {
        match ahead {
            Token::Nil => ExpDesc::Nil,
            Token::True => ExpDesc::Boolean(true),
            Token::False => ExpDesc::Boolean(false),
            Token::Integer(i) => ExpDesc::Integer(i),
            Token::Float(f) => ExpDesc::Float(f),
            Token::String(s) => ExpDesc::String(s),
            Token::Function => todo!("function"),
            Token::CurlyL => self.table_constructor(),
            Token::Sub | Token::Not | Token::BitXor | Token::Len => todo!("unop"),
            Token::Dots => todo!("dots"),
            t => self.prefixexp(t),
        }
    }

    fn prefixexp(&mut self, ahead: Token) -> ExpDesc {
        let sp0 = self.sp;

        let mut desc = match ahead {
            Token::Ident(name) => self.simple_name(name),
            Token::ParL => {
                let desc = self.exp();
                self.lexer.expect(Token::ParR);
                desc
            }
            t => panic!("prefixexp unexpected token: {:?}", t),
        };

        loop {
            match self.lexer.peek() {
                Token::SqurL => {
                    self.lexer.next();
                    let itable = self.discharge_if_need(sp0, desc);
                    desc = match self.exp() {
                        ExpDesc::String(s) => ExpDesc::IndexField(itable, self.add_const(s)),
                        ExpDesc::Integer(i) if u8::try_from(i).is_ok() => ExpDesc::IndexInt(itable, u8::try_from(i).unwrap()),
                        key => ExpDesc::Index(itable, self.discharge_top(key))
                    };

                    self.lexer.expect(Token::SqurR);
                }
                Token::Dot => {
                    self.lexer.next();
                    let name = self.read_name();
                    let itable = self.discharge_if_need(sp0, desc);
                    desc = ExpDesc::IndexField(itable, self.add_const(name));
                }
                Token::Colon => todo!("args"),
                Token::ParL | Token::CurlyL | Token::String(_) => {
                    self.discharge(sp0, desc);
                    desc = self.args();
                }
                _ => {break desc}
            }
        }
    }

    fn simple_name(&mut self, name: String) -> ExpDesc {
        if let Some(ilocal) = self.locals.iter().rposition(|v| v == &name) {
            ExpDesc::Local(ilocal) 
        } else {
            ExpDesc::Global(self.add_const(name))
        }
    }

    fn args(&mut self) -> ExpDesc {
        let ifunc = self.sp - 1;
        let argn = match self.lexer.next() {
            Token::ParL => {
                if self.lexer.peek() != &Token::ParR {
                    let argn = self.explist();
                    self.lexer.expect(Token::ParR);
                    argn
                } else {
                    self.lexer.next();
                    0
                }
            }
            Token::CurlyL => {
                self.table_constructor();
                1
            }
            Token::String(s) => {
                self.discharge(ifunc + 1, ExpDesc::String(s));
                1
            }
            t => panic!("args unexpected token: {:?}", t),
        };
        self.bytecodes.push(Bytecode::Call(ifunc as u8, argn as u8));
        ExpDesc::Call
    }

    fn discharge_top(&mut self, desc: ExpDesc) -> usize {
        self.discharge_if_need(self.sp, desc)
    }

    fn discharge_if_need(&mut self, dst: usize, desc: ExpDesc) -> usize {
        if let ExpDesc::Local(i) = desc {
            i
        } else {
            self.discharge(dst, desc);
            dst
        }
    }

    fn discharge(&mut self, dst: usize, desc: ExpDesc) {
        let code = match desc {
            ExpDesc::Nil => Bytecode::LoadNil(dst as u8, 1),
            ExpDesc::Boolean(b) => Bytecode::LoadBool(dst as u8, b),
            ExpDesc::Integer(i) => if let Ok(i) = i16::try_from(i) {
                Bytecode::LoadInt(dst as u8, i)
            } else {
                Bytecode::LoadConst(dst as u8, self.add_const(i) as u16)
            },
            ExpDesc::Float(f) => Bytecode::LoadConst(dst as u8, self.add_const(f) as u16),
            ExpDesc::String(s) => Bytecode::LoadConst(dst as u8, self.add_const(s) as u16),
            ExpDesc::Local(src) => if dst == src {
                return;
            } else {
                Bytecode::Move(dst as u8, src as u8)
            },
            ExpDesc::Global(iname) => Bytecode::GetGlobal(dst as u8, iname as u8),
            ExpDesc::Index(t, k) => Bytecode::GetTable(dst as u8, t as u8, k as u8),
            ExpDesc::IndexField(t, k) => Bytecode::GetField(dst as u8, t as u8, k as u8),
            ExpDesc::IndexInt(t, k) => Bytecode::GetInt(dst as u8, t as u8, k),
            ExpDesc::Call => panic!("discharge call"),
        };
        self.bytecodes.push(code);
        self.sp = dst + 1;
    }

    fn discharge_const(&mut self, desc: ExpDesc) -> ConstStack {
        match desc {
            ExpDesc::Nil => ConstStack::Const(self.add_const(Value::Nil)),
            ExpDesc::Boolean(b) => ConstStack::Const(self.add_const(b)),
            ExpDesc::Integer(i) => ConstStack::Const(self.add_const(i)),
            ExpDesc::Float(f) => ConstStack::Const(self.add_const(f)),
            ExpDesc::String(s) => ConstStack::Const(self.add_const(s)),

            _ => ConstStack::Stack(self.discharge_top(desc))
        }
    }

    fn table_constructor(&mut self) -> ExpDesc {
        let table = self.sp;
        self.sp += 1;

        let inew = self.bytecodes.len();
        self.bytecodes.push(Bytecode::NewTable(table as u8, 0, 0));

        enum TableEntry {
            Map((fn(u8, u8, u8) -> Bytecode, fn(u8, u8, u8) -> Bytecode, usize)),
            Array(ExpDesc)
        }

        let mut stored = 0;
        let mut tostore = 0;
        let mut narray = 0;
        let mut nmap = 0;
        loop {
            let sp0 = self.sp;

            let entry = match self.lexer.peek() {
                Token::CurlyL => {
                    self.lexer.next();
                    TableEntry::Array(self.table_constructor())
                }
                Token::SqurL => {
                    self.lexer.next();
                    let key = self.exp();
                    self.lexer.expect(Token::SqurR);
                    self.lexer.expect(Token::Assign);

                    TableEntry::Map(
                        match key {
                            ExpDesc::Local(i) => (Bytecode::SetTable, Bytecode::SetTableConst, i),
                            ExpDesc::String(s) => (Bytecode::SetField, Bytecode::SetFieldConst, self.add_const(s)),
                            ExpDesc::Integer(i) if u8::try_from(i).is_ok() => (Bytecode::SetInt, Bytecode::SetIntConst, i as usize),
                            ExpDesc::Nil => panic!("nil can not be table key"),
                            ExpDesc::Float(f) if f.is_nan() => panic!("NaN can not be table key"),
                            _ => (Bytecode::SetTable, Bytecode::SetTableConst, self.discharge_top(key)),
                        }
                    )
                }
                Token::Ident(_) => {
                    let name = self.read_name();
                    if self.lexer.peek() == &Token::Assign {
                        self.lexer.next();
                        TableEntry::Map((Bytecode::SetField, Bytecode::SetFieldConst, self.add_const(name)))
                    } else {
                        TableEntry::Array(self.exp_with_ahead(Token::Ident(name)))
                    }
                },
                _ => {
                    TableEntry::Array(self.exp())
                }
            };

            match entry {
                TableEntry::Map((op, opk, key)) => {
                    let value = self.exp();
                    let code = match self.discharge_const(value) {
                        ConstStack::Const(iv) => opk(table as u8, key as u8, iv as u8),
                        ConstStack::Stack(iv) => op(table as u8, key as u8, iv as u8),
                    };
                    self.bytecodes.push(code);
                    nmap += 1;
                    self.sp = sp0;
                }
                TableEntry::Array(value) => {
                    self.discharge(sp0, value);
                    narray += 1;
                    tostore += 1;
                    if tostore == 50 {
                        self.bytecodes.push(Bytecode::SetList(table as u8, tostore, stored));
                        stored += tostore;
                    }
                }
            }

            match self.lexer.next() {
                Token ::SemiColon | Token::Comma => (),
                Token::CurlyR => break,
                t => panic!("unexpected token in table constructor: {:?}", t)
            }
        }

        if self.sp > table + 1 {
            self.bytecodes.push(Bytecode::SetList(table as u8, (self.sp - (table + 1)) as u8, stored));
        }

        self.bytecodes[inew] = Bytecode::NewTable(table as u8, narray as u8, nmap as u8);

        self.sp = table + 1;
        ExpDesc::Local(table)
    }

    fn read_name(&mut self) -> String {
        if let Token::Ident(name) = self.lexer.next() {
            name
        } else {
            panic!("expected name");
        }
    }
}