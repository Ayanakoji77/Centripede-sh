// builtins.rs
use crate::exec::ShellState;
use std::env;
use std::path::Path;
use std::process;

pub fn run_builtin(args: &[String], state: &mut ShellState) -> bool {
    if args.is_empty() {
        return false;
    }

    match args[0].as_str() {
        "cd" => {
            let target = if args.len() > 1 {
                args[1].clone()
            } else {
                env::var("HOME").unwrap_or_else(|_| "/".to_string())
            };

            match env::set_current_dir(Path::new(&target)) {
                Ok(()) => state.last_status = 0,
                Err(e) => {
                    eprintln!("minish: cd: {}: {}", target, e);
                    state.last_status = 1;
                }
            }
            true
        }
        "pwd" => {
            match env::current_dir() {
                Ok(dir) => {
                    println!("{}", dir.display());
                    state.last_status = 0;
                }
                Err(e) => {
                    eprintln!("minish: pwd: {}", e);
                    state.last_status = 1;
                }
            }
            true
        }
        "exit" => {
            let code = if args.len() > 1 {
                match args[1].parse::<i32>() {
                    Ok(n) => n,
                    Err(_) => {
                        eprintln!("minish: exit: {}: numeric argument required", args[1]);
                        2
                    }
                }
            } else {
                state.last_status
            };
            process::exit(code);
        }
        _ => false,
    }
}
