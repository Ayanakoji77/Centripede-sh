use crate::builtins::run_builtin;
use crate::lexer::WordPart;
use crate::parser::{JobQueue, Operator, Pipeline};
use std::fs::{File, OpenOptions};
use std::io;
use std::process::{Child, ChildStdout, Command, Stdio};

pub struct ShellState {
    pub last_status: i32,
}

impl ShellState {
    pub fn new() -> Self {
        Self { last_status: 0 }
    }
}

fn expand_dollars(s: &str, state: &ShellState) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '$' {
            result.push(c);
            continue;
        }

        match chars.peek().copied() {
            Some('?') => {
                chars.next();
                result.push_str(&state.last_status.to_string());
            }
            Some('{') => {
                chars.next(); // consume '{'
                let mut name = String::new();
                let mut closed = false;
                for ch in chars.by_ref() {
                    if ch == '}' {
                        closed = true;
                        break;
                    }
                    name.push(ch);
                }
                if closed {
                    if let Ok(val) = std::env::var(&name) {
                        result.push_str(&val);
                    }
                } else {
                    result.push_str("${");
                    result.push_str(&name);
                }
            }
            Some(next) if next.is_alphabetic() || next == '_' => {
                let mut name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        name.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(val) = std::env::var(&name) {
                    result.push_str(&val);
                }
            }
            _ => {
                result.push('$');
            }
        }
    }

    result
}

pub fn expand_word(parts: &[WordPart], state: &ShellState) -> String {
    let mut result = String::new();
    for part in parts {
        match part {
            WordPart::Word(s) | WordPart::DoubleQuoted(s) => {
                result.push_str(&expand_dollars(s, state));
            }
            WordPart::SingleQuoted(s) => {
                result.push_str(s);
            }
        }
    }
    result
}

pub fn execute_queue(queue: JobQueue, state: &mut ShellState) {
    for job in queue.0 {
        let should_run = match job.work_operator {
            Operator::None => true,
            Operator::And => state.last_status == 0,
            Operator::Or => state.last_status != 0,
        };

        if !should_run {
            continue;
        }

        match execute_pipeline(&job.pipeline, state) {
            Ok(status) => state.last_status = status,
            Err(e) => {
                eprintln!("minish: {}", e);
                state.last_status = 1;
            }
        }
    }
}

fn reap_all(children: Vec<Child>) {
    for mut c in children {
        let _ = c.wait();
    }
}

fn execute_pipeline(pipeline: &Pipeline, state: &mut ShellState) -> io::Result<i32> {
    if pipeline.commands.is_empty() {
        return Ok(0);
    }

    if pipeline.commands.len() == 1 {
        let cmd = &pipeline.commands[0];
        let argv: Vec<String> = cmd.argv.iter().map(|p| expand_word(p, state)).collect();

        if let Some(name) = argv.first() {
            if is_builtin(name) {
                run_builtin(&argv, state);
                return Ok(state.last_status);
            }
        }
    }

    let n = pipeline.commands.len();
    let mut children: Vec<Child> = Vec::with_capacity(n);
    let mut prev_stdout: Option<ChildStdout> = None;

    for (i, cmd) in pipeline.commands.iter().enumerate() {
        let argv: Vec<String> = cmd.argv.iter().map(|p| expand_word(p, state)).collect();
        let Some(program) = argv.first() else {
            continue;
        };

        let mut command = Command::new(program);
        if argv.len() > 1 {
            command.args(&argv[1..]);
        }

        if let Some(prev) = prev_stdout.take() {
            command.stdin(Stdio::from(prev));
        } else if let Some(infile_parts) = &cmd.stdin_from {
            let path = expand_word(infile_parts, state);
            let file = match File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("minish: {}: {}", path, e);
                    reap_all(children);
                    return Ok(1);
                }
            };
            command.stdin(Stdio::from(file));
        }

        if i < n - 1 {
            command.stdout(Stdio::piped());
        } else if let Some(outfile_parts) = &cmd.stdout_to {
            let path = expand_word(outfile_parts, state);
            let opened = if cmd.append_stdout {
                OpenOptions::new().create(true).append(true).open(&path)
            } else {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&path)
            };
            let file = match opened {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("minish: {}: {}", path, e);
                    reap_all(children);
                    return Ok(1);
                }
            };
            command.stdout(Stdio::from(file));
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                eprintln!("minish: command not found: {}", program);
                reap_all(children);
                return Ok(127);
            }
            Err(e) => {
                eprintln!("minish: {}: {}", program, e);
                reap_all(children);
                return Ok(126);
            }
        };

        if i < n - 1 {
            prev_stdout = child.stdout.take();
        }

        children.push(child);
    }

    let mut last_status = 0;
    for mut child in children {
        let status = child.wait()?;
        last_status = status.code().unwrap_or(1);
    }

    Ok(last_status)
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "cd" | "pwd" | "exit")
}
