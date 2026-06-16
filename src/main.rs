// 1. Declare the modules (this tells Rust to look for these files)
mod builtins;
mod exec;
mod lexer;
mod parser;

// 2. Bring specific structs/functions into scope for main to use
use std::io;

fn main() {
    loop {
        println!("minish>");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to get line");

        let input = input.trim();
    }
}
