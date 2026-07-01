mod builtins;
mod exec;
mod lexer;
mod parser;

use exec::ShellState;
use std::env;
use std::io::{self, Write};

fn main() {
    let mut state = ShellState::new();

    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 && args[1] == "-c" {
        run_line(&args[2], &mut state);
        std::process::exit(state.last_status);
    }

    loop {
        print!("minish> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("exit");
                break;
            }
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                }
                run_line(trimmed, &mut state);
            }
            Err(e) => {
                eprintln!("minish: read error: {}", e);
                break;
            }
        }
    }
}

fn run_line(input: &str, state: &mut ShellState) {
    let tokens = match lexer::tokenize(input) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("minish: lexer error: {:?}", e);
            state.last_status = 1;
            return;
        }
    };

    let job_queue = match parser::parse(tokens) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("minish: parser error: {:?}", e);
            state.last_status = 1;
            return;
        }
    };

    exec::execute_queue(job_queue, state);
}
