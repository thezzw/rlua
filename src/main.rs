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
    let proto = parser::load(file);
    vm::ExeState::new().execute(&proto);
}
