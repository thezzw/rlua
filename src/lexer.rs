use std::{fs::File, io::{Bytes, Read}, iter::Peekable};

type LexerBytes = Peekable<Bytes<File>>;

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
    String(String),
    Eos
}

pub struct Lexer {
    bytes: LexerBytes,
    ahead: Option<Token>,
}

impl Lexer {
    pub fn new(input: File) -> Self {
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
                    match byte as char {
                        // ignore whitespace
                        ' ' | '\n' | '\r' | '\t' => continue,
                        '+' => break Token::Add,
                        '*' => break Token::Mul,
                        '%' => break Token::Mod,
                        '^' => break Token::Pow,
                        '#' => break Token::Len,
                        '&' => break Token::BitAnd,
                        '|' => break Token::BitOr,
                        '(' => break Token::ParL,
                        ')' => break Token::ParR,
                        '{' => break Token::CurlyL,
                        '}' => break Token::CurlyR,
                        '[' => break Token::SqurL,
                        ']' => break Token::SqurR,
                        ';' => break Token::SemiColon,
                        ',' => break Token::Comma,
                        '/' => break self.try_parse_long('/', Token::Idiv, Token::Div),
                        '=' => break self.try_parse_long('=', Token::Equal, Token::Assign),
                        '~' => break self.try_parse_long('=', Token::NotEq, Token::BitXor),
                        ':' => break self.try_parse_long(':', Token::DoubColon, Token::Colon),
                        '<' => break self.try_parse_long_alt('=', Token::LesEq, '<', Token::ShiftL, Token::Less),
                        '>' => break self.try_parse_long_alt('=', Token::GreEq, '>', Token::ShiftR, Token::Greater),
                        // identifier
                        ident_head if ident_head.is_ascii_alphabetic() || ident_head == '_' => {
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
                        '.' => break self.parse_dot_token(),
                        // sub or comment
                        '-' => break match self.bytes.peek() {
                            Some(Ok(comment_byte)) => {
                                match *comment_byte as char {
                                    '-' => {
                                        self.bytes.next();
                                        while let Some(Ok(comment_content_byte)) = self.bytes.next() {
                                            if comment_content_byte as char == '\n' {
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
                        '"' | '\'' => break self.parse_string(byte as char),
                        // number
                        '0'..='9' => break self.parse_number(byte as char),
                        // unknown
                        unknown => unimplemented!("Unexpected char: {}", unknown as char),
                    }
                },
                Some(Err(e)) => panic!("Error reading token: {}", e),
                _ => break Token::Eos
            }
        }
    }

    fn try_parse_long(&mut self, second: char, long: Token, short: Token) -> Token {
        let peek_byte = self.bytes.next();
        if let Some(Ok(byte)) = peek_byte {
            if byte as char == second {
                return long
            }
        }
        short
    }

    fn try_parse_long_alt(&mut self, second_a: char, long_a: Token, second_b: char, long_b: Token, short: Token) -> Token {
        let peek_byte = self.bytes.next();
        if let Some(Ok(byte)) = peek_byte {
            if byte as char == second_a {
                return long_a
            } else if byte as char == second_b {
                return long_b
            }
        }
        short
    }

    fn parse_string(&mut self, quote: char) -> Token {
        let mut string = String::new();
        loop {
            let next_byte = self.bytes.next();
            match next_byte {
                Some(Ok(string_byte)) => {
                    match string_byte as char {
                        '\n' => panic!("Unexpected newline in string"),
                        '\\' => {
                            // Consume the escaped character after '\\'
                            while let Some(Ok(escape_byte)) = self.bytes.next() {
                                match escape_byte as char {
                                    ' ' | '\r' | '\t' => continue,
                                    '\n' => break,
                                    escape => panic!("Unexpected escape: {}", escape as char)
                                }
                            }
                            string.push('\n');
                        },
                        end if end == quote => break Token::String(string),
                        content => string.push(content as char)
                    }
                },
                Some(Err(e)) => panic!("Error parsing string: {}", e),
                _ => panic!("Expected closing quote: {}", quote as char)
            }
        }
    }

    fn parse_dot_token(&mut self) -> Token {
        match self.bytes.peek() {
            Some(Ok(dot_byte)) => {
                match *dot_byte as char {
                    '.' => {
                        match self.bytes.peek() {
                            Some(Ok(dots_byte)) => {
                                match *dots_byte as char {
                                    '.' => {
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

    fn parse_number(&mut self, first_byte: char) -> Token {
        if first_byte == '0' {
            match self.bytes.peek() {
                Some(Ok(hex_byte)) => {
                    match *hex_byte as char {
                        'x' | 'X' => {
                            self.bytes.next();
                            return self.parse_number_hex();
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
        }

        let mut n = first_byte.to_digit(10).unwrap() as i64;
        loop {
            let ch = self.bytes.peek();
            match ch {
                Some(Ok(byte)) => {
                    match *byte as char {
                        '0'..='9' => {
                            let int_byte = self.bytes.next().unwrap().unwrap();
                            n = n * 10 + (int_byte as char).to_digit(10).unwrap() as i64;
                        },
                        '.' => return self.parse_number_frac(n as f64),
                        'e' | 'E' => return self.parse_number_exp(n as f64),
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
                match *byte as char {
                    invalid_char if invalid_char.is_alphabetic() || invalid_char == '.' => {
                        panic!("Invalid number end: {}", invalid_char)
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
                    match *byte as char {
                        '0'..='9' => {
                            let int_byte = self.bytes.next().unwrap().unwrap();
                            n = n * 10 + (int_byte as char).to_digit(10).unwrap() as i64;
                            x *= 10.0;   
                        },
                        'e' | 'E' => return self.parse_number_exp(number_base + (n as f64) / x),
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