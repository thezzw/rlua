use std::{io::{Bytes, Read}, iter::Peekable};

#[derive(Debug, PartialEq)]
pub enum Token {
//  keywards
    And,    Break,  Do,     Else,   Elseif, End,
    False,  For,    Function, Goto, If,     In,
    Local,  Nil,    Not,    Or,     Repeat, Return,
    Then,   True,   Until,  While,

//  +       -       *       /       %       ^       #
    Add,    Sub,    Mul,    Div,    Mod,    Pow,    Len,
//  &       ~       |       <<      >>      //
    BitAnd, BitXor, BitOr,  ShiftL, ShiftR, Idiv,
//  ==       ~=     <=      >=      <       >        =
    Equal,  NotEq,  LesEq,  GreEq,  Less,   Greater, Assign,
//  (       )       {       }       [       ]       ::
    ParL,   ParR,   CurlyL, CurlyR, SqurL,  SqurR,  DoubColon,
//  ;               :       ,       .       ..      ...
    SemiColon,      Colon,  Comma,  Dot,    Concat, Dots,

    Integer(i64),
    Float(f64),

    Ident(String),
    String(Vec<u8>),
    Eos
}

pub struct Lexer<R: Read> {
    bytes: Peekable<Bytes<R>>,
    ahead: Option<Token>,
}

impl<R: Read> Lexer<R> {
    pub fn new(input: R) -> Self {
        Self {
            bytes: input.bytes().into_iter().peekable(),
            ahead: None,
        }
    }

    pub fn peek(&mut self) -> &Token {
        if self.ahead.is_none() {
            self.ahead = Some(self.next());
        }
        self.ahead.as_ref().unwrap()
    }

    pub fn next(&mut self) -> Token {
        if self.ahead.is_some() {
            return self.ahead.take().unwrap();
        }

        loop {
            let next_byte = self.bytes.next();
            match next_byte {
                Some(Ok(byte)) => {
                    match byte {
                        // ignore whitespace
                        b' ' | b'\n' | b'\r' | b'\t' => continue,
                        b'+' => break Token::Add,
                        b'*' => break Token::Mul,
                        b'%' => break Token::Mod,
                        b'^' => break Token::Pow,
                        b'#' => break Token::Len,
                        b'&' => break Token::BitAnd,
                        b'|' => break Token::BitOr,
                        b'(' => break Token::ParL,
                        b')' => break Token::ParR,
                        b'{' => break Token::CurlyL,
                        b'}' => break Token::CurlyR,
                        b'[' => break Token::SqurL,
                        b']' => break Token::SqurR,
                        b';' => break Token::SemiColon,
                        b',' => break Token::Comma,
                        b'/' => break self.try_parse_long(b'/', Token::Idiv, Token::Div),
                        b'=' => break self.try_parse_long(b'=', Token::Equal, Token::Assign),
                        b'~' => break self.try_parse_long(b'=', Token::NotEq, Token::BitXor),
                        b':' => break self.try_parse_long(b':', Token::DoubColon, Token::Colon),
                        b'<' => break self.try_parse_long_alt(b'=', Token::LesEq, b'<', Token::ShiftL, Token::Less),
                        b'>' => break self.try_parse_long_alt(b'=', Token::GreEq, b'>', Token::ShiftR, Token::Greater),
                        // identifier
                        ident_head if ident_head.is_ascii_alphabetic() || ident_head == b'_' => {
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
                            break match &ident {
                                str if str == "and" => Token::And,
                                str if str == "break" => Token::Break,
                                str if str == "do" => Token::Do,
                                str if str == "else" => Token::Else,
                                str if str == "elseif" => Token::Elseif,
                                str if str == "end" => Token::End,
                                str if str == "false" => Token::False,
                                str if str == "for" => Token::For,
                                str if str == "function" => Token::Function,
                                str if str == "goto" => Token::Goto,
                                str if str == "if" => Token::If,
                                str if str == "in" => Token::In,
                                str if str == "local" => Token::Local,
                                str if str == "nil" => Token::Nil,
                                str if str == "not" => Token::Not,
                                str if str == "or" => Token::Or,
                                str if str == "repeat" => Token::Repeat,
                                str if str == "return" => Token::Return,
                                str if str == "then" => Token::Then,
                                str if str == "true" => Token::True,
                                str if str == "until" => Token::Until,
                                str if str == "while" => Token::While,
                                _ => Token::Ident(ident)
                            };
                        },
                        b'.' => break self.parse_dot_token(),
                        // sub or comment
                        b'-' => break match self.bytes.peek() {
                            Some(Ok(comment_byte)) => {
                                match *comment_byte {
                                    b'-' => {
                                        self.bytes.next();
                                        while let Some(Ok(comment_content_byte)) = self.bytes.next() {
                                            if comment_content_byte == b'\n' {
                                                break;
                                            }
                                        }
                                        continue;
                                    },
                                    _ => Token::Sub
                                }
                            },
                            _ => Token::Sub
                        },
                        // string
                        b'"' | b'\'' => break self.parse_string(byte),
                        // number
                        b'0'..=b'9' => break self.parse_number(byte),
                        // unknown
                        unknown => unimplemented!("Unexpected u8: {}", unknown),
                    }
                },
                Some(Err(e)) => panic!("Error reading token: {}", e),
                _ => break Token::Eos
            }
        }
    }

    fn peek_byte(&mut self) -> u8 {
        match self.bytes.peek() {
            Some(Ok(byt)) => *byt,
            Some(_) => panic!("lex peek error"),
            None => b'\0',
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.bytes.next().and_then(|r|Some(r.unwrap()))
    }

    fn try_parse_long(&mut self, second: u8, long: Token, short: Token) -> Token {
        let peek_byte = self.bytes.next();
        if let Some(Ok(byte)) = peek_byte {
            if byte == second {
                return long
            }
        }
        short
    }

    fn try_parse_long_alt(&mut self, second_a: u8, long_a: Token, second_b: u8, long_b: Token, short: Token) -> Token {
        let peek_byte = self.bytes.next();
        if let Some(Ok(byte)) = peek_byte {
            if byte == second_a {
                return long_a
            } else if byte == second_b {
                return long_b
            }
        }
        short
    }

    fn parse_string(&mut self, quote: u8) -> Token {
        let mut string = Vec::new();
        loop {
            let next_byte = self.bytes.next();
            match next_byte {
                Some(Ok(string_byte)) => {
                    match string_byte {
                        b'\n' => panic!("Unexpected newline in string"),
                        b'\\' => string.push(self.parse_escape()),
                        end if end == quote => break Token::String(string),
                        content => string.push(content)
                    }
                },
                Some(Err(e)) => panic!("Error parsing string: {}", e),
                _ => panic!("Expected closing quote: {}", quote)
            }
        }
    }

    fn parse_escape(&mut self) -> u8 {
        let next_byte = self.bytes.next();
        match next_byte {
            Some(Ok(byte)) => {
                match byte {
                    b'n' => b'\n',
                    b't' => b'\t',
                    b'r' => b'\r',
                    b'b' => b'\x08',
                    b'f' => b'\x0C',
                    b'a' => b'\x07',
                    b'v' => b'\x0B',
                    b'\\' => b'\\',
                    b'"' => b'"',
                    b'\'' => b'\'',
                    b'x' => { // format: \xXX
                        let n1 = char::to_digit(self.next_byte().unwrap() as char, 16).unwrap();
                        let n2 = char::to_digit(self.next_byte().unwrap() as char, 16).unwrap();
                        (n1 * 16 + n2) as u8
                    }
                    ch@b'0'..=b'9' => { // format: \d[d[d]]
                        let mut n = char::to_digit(ch as char, 10).unwrap(); // TODO no unwrap
                        if let Some(d) = char::to_digit(self.peek_byte() as char, 10) {
                            self.next_byte();
                            n = n * 10 + d;
                            if let Some(d) = char::to_digit(self.peek_byte() as char, 10) {
                                self.next_byte();
                                n = n * 10 + d;
                            }
                        }
                        u8::try_from(n).expect("decimal escape too large")
                    }
                    _ => byte
                }
            },
            Some(Err(e)) => panic!("Error parsing escape: {}", e),
            _ => panic!("Expected escape character")
        }
    }

    fn parse_dot_token(&mut self) -> Token {
        match self.bytes.peek() {
            Some(Ok(dot_byte)) => {
                match *dot_byte {
                    b'.' => {
                        match self.bytes.peek() {
                            Some(Ok(dots_byte)) => {
                                match *dots_byte {
                                    b'.' => {
                                        self.bytes.next();
                                        Token::Dots
                                    },
                                    _ => Token::Concat
                                }
                            },
                            _ => Token::Concat
                        }
                    },
                    frac_byte if frac_byte.is_ascii_digit() => self.parse_number_frac(0.0),
                    _ => Token::Dot,
                }
            },
            _ => Token::Dot
        }
    }

    fn parse_number(&mut self, first_byte: u8) -> Token {
        if first_byte == b'0' {
            match self.bytes.peek() {
                Some(Ok(hex_byte)) => {
                    match *hex_byte {
                        b'x' | b'X' => {
                            self.bytes.next();
                            return self.parse_number_hex();
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
        }

        let mut n = (first_byte as char).to_digit(10).unwrap() as i64;
        loop {
            let ch = self.bytes.peek();
            match ch {
                Some(Ok(byte)) => {
                    match *byte {
                        b'0'..=b'9' => {
                            let int_byte = self.bytes.next().unwrap().unwrap();
                            n = n * 10 + (int_byte as char).to_digit(10).unwrap() as i64;
                        },
                        b'.' => return self.parse_number_frac(n as f64),
                        b'e' | b'E' => return self.parse_number_exp(n as f64),
                        _ => break
                    }
                },
                Some(Err(e)) => panic!("Error parsing number: {}", e),
                _ => break
            }
        }

        let follow = self.bytes.peek();
        match follow {
            Some(Ok(byte)) => {
                match *byte {
                    invalid_u8 if (invalid_u8 as char).is_alphabetic() || invalid_u8 == b'.' => {
                        panic!("Invalid number end: {}", invalid_u8)
                    }
                    _ => ()
                }
            },
            _ => ()
        }

        return Token::Integer(n);
    }

    fn parse_number_frac(&mut self, number_base: f64) -> Token {
        let mut n = 0;
        let mut x = 1.0;
        loop {
            let ch = self.bytes.peek();
            match ch {
                Some(Ok(byte)) => {
                    match *byte {
                        b'0'..=b'9' => {
                            let int_byte = self.bytes.next().unwrap().unwrap();
                            n = n * 10 + (int_byte as char).to_digit(10).unwrap() as i64;
                            x *= 10.0;   
                        },
                        b'e' | b'E' => return self.parse_number_exp(number_base + (n as f64) / x),
                        _ => break
                    }
                },
                Some(Err(e)) => panic!("Error parsing number: {}", e),
                _ => break
            }
        }
        Token::Float(number_base as f64 + n as f64 / x)
    }

    fn parse_number_hex(&mut self) -> Token {
        unimplemented!()
    }

    fn parse_number_exp(&mut self, number_base: f64) -> Token {
        unimplemented!()
    }
}