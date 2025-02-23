use std::{fs::File, io::{Bytes, Read}, iter::Peekable};

#[derive(Debug)]
pub enum Token {
    Ident(String),
    String(String),
    Eos
}

pub struct Lexer {
    bytes: Peekable<Bytes<File>>
}

impl Lexer {
    pub fn new(input: File) -> Self {
        Self { bytes: input.bytes().into_iter().peekable() }
    }

    pub fn next(&mut self) -> Token {
        loop {
            let peek_byte = self.bytes.next();
            match peek_byte {
                Some(Ok(byte)) => {
                    match byte {
                        // 忽略空白符号
                        b' ' | b'\n' | b'\r' | b'\t' => continue,
                        // 标识符
                        ident_head if ident_head.is_ascii_alphabetic() => {
                            let mut ident = String::from(ident_head as char);
                            while let Some(Ok(ident_byte)) = self.bytes.peek() {
                                if ident_byte.is_ascii_alphanumeric()
                                || ident_byte == &b'_' {
                                    ident.push(*ident_byte as char);
                                    self.bytes.next();
                                } else {
                                    break;
                                }
                            }
                            break Token::Ident(ident);
                        },
                        // 字符串
                        b'"' => {
                            let mut string = String::new();
                            while let Some(Ok(string_byte)) = self.bytes.next() {
                                if string_byte == b'"' {
                                    break;
                                }
                                string.push(string_byte as char);
                            }
                            break Token::String(string);
                        },
                        // 未实现的字符
                        any_char => unimplemented!("Unexpected char: {}", any_char as char),
                    }
                },
                Some(Err(e)) => panic!("Error reading token: {}", e),
                _ => break Token::Eos
            }
        }
    }
}