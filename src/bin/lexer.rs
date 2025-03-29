use std::env;
use std::fs::File;

use rlua::lexer::{Lexer, Token};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <filename>", args[0]);
        return;
    }

    let file = File::open(&args[1]).unwrap();
    let mut lexer = Lexer::new(file);

    loop {
        match lexer.next() {
            Token::Eos => break,
            any => {println!("{:?}", any);},
        }
    }
}