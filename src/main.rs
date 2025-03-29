use std::env;
use std::fs::File;

mod value;
mod bytecode;
mod lexer;
mod parser;
mod vm;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <filename>", args[0]);
        return;
    }

    let file = File::open(&args[1]).unwrap();
    let lexer = lexer::Lexer::new(file);
    let proto = parser::ParseProto::load(lexer);  

    let mut exe_state = vm::ExeState::new();
    exe_state.execute(&proto);
}
