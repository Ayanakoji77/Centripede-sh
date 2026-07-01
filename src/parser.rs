use crate::lexer::*;

#[derive(Debug, PartialEq)]
pub enum Operator {
    None,
    And,
    Or,
}
#[derive(Debug, PartialEq)]
pub struct Pipeline {
    pub commands: Vec<Cmd>,
}

#[derive(Debug, PartialEq)]
pub struct Cmd {
    pub argv: Vec<Vec<WordPart>>,          // argv[0] is the program
    pub stdin_from: Option<Vec<WordPart>>, // < file
    pub stdout_to: Option<Vec<WordPart>>,  // > file
    pub append_stdout: bool,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    BadSyntax(String),
    TokenError(String),
    FilenameError,
}

#[derive(Debug, PartialEq)]
pub struct Job {
    pub work_operator: Operator,
    pub pipeline: Pipeline,
}

#[derive(Debug, PartialEq)]
pub struct JobQueue(pub Vec<Job>);

pub fn parse(token_vec: Vec<Token>) -> Result<JobQueue, ParseError> {
    let mut job_queue = JobQueue(Vec::new());
    let mut current_pipeline = Pipeline {
        commands: Vec::new(),
    };
    let mut current_cmd = Cmd {
        argv: Vec::new(),
        stdin_from: None,
        stdout_to: None,
        append_stdout: false,
    };
    let mut token_iter = token_vec.into_iter();
    let mut current_condition = Operator::None;
    while let Some(token) = token_iter.next() {
        match token {
            Token::Word(parts) => {
                current_cmd.argv.push(parts);
            }
            Token::RedirectAppend | Token::RedirectOut => match token_iter.next() {
                Some(Token::Word(parts)) => {
                    current_cmd.stdout_to = Some(parts);
                    current_cmd.append_stdout = token == Token::RedirectAppend;
                }
                _ => return Err(ParseError::FilenameError),
            },
            Token::RedirectIn => match token_iter.next() {
                Some(Token::Word(parts)) => {
                    current_cmd.stdin_from = Some(parts);
                }
                _ => {
                    return Err(ParseError::FilenameError);
                }
            },
            Token::Pipe => {
                if current_cmd.argv.is_empty() {
                    return Err(ParseError::BadSyntax("Unexpected pipe".to_string()));
                }
                current_pipeline.commands.push(current_cmd);
                current_cmd = Cmd {
                    argv: Vec::new(),
                    stdin_from: None,
                    stdout_to: None,
                    append_stdout: false,
                };
            }

            Token::And | Token::Or => {
                if current_cmd.argv.is_empty() && current_pipeline.commands.is_empty() {
                    return Err(ParseError::BadSyntax(
                        "Unexpected logic operator".to_string(),
                    ));
                }

                current_pipeline.commands.push(current_cmd);

                job_queue.0.push(Job {
                    work_operator: current_condition,
                    pipeline: current_pipeline,
                });

                current_cmd = Cmd {
                    argv: Vec::new(),
                    stdin_from: None,
                    stdout_to: None,
                    append_stdout: false,
                };
                current_pipeline = Pipeline {
                    commands: Vec::new(),
                };
                current_condition = if token == Token::And {
                    Operator::And
                } else {
                    Operator::Or
                };
            }
        }
    }
    if !current_cmd.argv.is_empty() {
        current_pipeline.commands.push(current_cmd);
    }
    if !current_pipeline.commands.is_empty() {
        job_queue.0.push(Job {
            work_operator: current_condition,
            pipeline: current_pipeline,
        });
    }
    Ok(job_queue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Token, WordPart};

    fn wp(s: &str) -> Vec<WordPart> {
        vec![WordPart::Word(s.to_string())]
    }

    fn w(s: &str) -> Token {
        Token::Word(wp(s))
    }

    #[test]
    fn test_simple_command() {
        let tokens = vec![w("ls"), w("-l"), w("/tmp")];
        let expected = JobQueue(vec![Job {
            work_operator: Operator::None,
            pipeline: Pipeline {
                commands: vec![Cmd {
                    argv: vec![wp("ls"), wp("-l"), wp("/tmp")],
                    stdin_from: None,
                    stdout_to: None,
                    append_stdout: false,
                }],
            },
        }]);
        assert_eq!(parse(tokens), Ok(expected));
    }

    #[test]
    fn test_pipeline() {
        let tokens = vec![w("ls"), Token::Pipe, w("wc")];
        let expected = JobQueue(vec![Job {
            work_operator: Operator::None,
            pipeline: Pipeline {
                commands: vec![
                    Cmd {
                        argv: vec![wp("ls")],
                        stdin_from: None,
                        stdout_to: None,
                        append_stdout: false,
                    },
                    Cmd {
                        argv: vec![wp("wc")],
                        stdin_from: None,
                        stdout_to: None,
                        append_stdout: false,
                    },
                ],
            },
        }]);
        assert_eq!(parse(tokens), Ok(expected));
    }

    #[test]
    fn test_redirections() {
        let tokens = vec![
            w("cat"),
            Token::RedirectIn,
            w("in.txt"),
            Token::RedirectOut,
            w("out.txt"),
        ];
        let expected = JobQueue(vec![Job {
            work_operator: Operator::None,
            pipeline: Pipeline {
                commands: vec![Cmd {
                    argv: vec![wp("cat")],
                    stdin_from: Some(wp("in.txt")),
                    stdout_to: Some(wp("out.txt")),
                    append_stdout: false,
                }],
            },
        }]);
        assert_eq!(parse(tokens), Ok(expected));
    }

    #[test]
    fn test_append_redirection() {
        let tokens = vec![w("echo"), w("a"), Token::RedirectAppend, w("log.txt")];
        let expected = JobQueue(vec![Job {
            work_operator: Operator::None,
            pipeline: Pipeline {
                commands: vec![Cmd {
                    argv: vec![wp("echo"), wp("a")],
                    stdin_from: None,
                    stdout_to: Some(wp("log.txt")),
                    append_stdout: true, // This should be true!
                }],
            },
        }]);
        assert_eq!(parse(tokens), Ok(expected));
    }

    #[test]
    fn test_logical_chaining() {
        let tokens = vec![
            w("make"),
            Token::Or,
            w("echo"),
            w("failed"),
            Token::And,
            w("exit"),
        ];
        let expected = JobQueue(vec![
            Job {
                work_operator: Operator::None,
                pipeline: Pipeline {
                    commands: vec![Cmd {
                        argv: vec![wp("make")],
                        stdin_from: None,
                        stdout_to: None,
                        append_stdout: false,
                    }],
                },
            },
            Job {
                work_operator: Operator::Or,
                pipeline: Pipeline {
                    commands: vec![Cmd {
                        argv: vec![wp("echo"), wp("failed")],
                        stdin_from: None,
                        stdout_to: None,
                        append_stdout: false,
                    }],
                },
            },
            Job {
                work_operator: Operator::And,
                pipeline: Pipeline {
                    commands: vec![Cmd {
                        argv: vec![wp("exit")],
                        stdin_from: None,
                        stdout_to: None,
                        append_stdout: false,
                    }],
                },
            },
        ]);
        assert_eq!(parse(tokens), Ok(expected));
    }

    #[test]
    fn test_missing_filename_output() {
        let tokens = vec![w("ls"), Token::RedirectOut];
        assert_eq!(parse(tokens), Err(ParseError::FilenameError));
    }

    #[test]
    fn test_missing_filename_input() {
        let tokens = vec![w("cat"), Token::RedirectIn];
        assert_eq!(parse(tokens), Err(ParseError::FilenameError));
    }

    #[test]
    fn test_invalid_redirect_target() {
        let tokens = vec![w("ls"), Token::RedirectOut, Token::Pipe];
        assert_eq!(parse(tokens), Err(ParseError::FilenameError));
    }

    #[test]
    fn test_double_pipe() {
        let tokens = vec![w("ls"), Token::Pipe, Token::Pipe, w("wc")];
        assert_eq!(
            parse(tokens),
            Err(ParseError::BadSyntax("Unexpected pipe".to_string()))
        );
    }

    #[test]
    fn test_leading_pipe() {
        let tokens = vec![Token::Pipe, w("ls")];
        assert_eq!(
            parse(tokens),
            Err(ParseError::BadSyntax("Unexpected pipe".to_string()))
        );
    }

    #[test]
    fn test_leading_logic_operator() {
        let tokens = vec![Token::And, w("ls")];
        assert_eq!(
            parse(tokens),
            Err(ParseError::BadSyntax(
                "Unexpected logic operator".to_string()
            ))
        );
    }
}
