mod lexer


#[derive(Debug, PartialEq)]
pub enum Operator
    {
        None, And, Or,
    }
#[derive(Debug, PartialEq)]
pub struct Pipeline {
    pub commands: Vec<Cmd>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Cmd {
    pub argv: Vec<String>,          // argv[0] is the program
    pub stdin_from: Option<String>, // < file
    pub stdout_to: Option<String>,  // > file
    pub append_stdout: bool,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    BadSyntax(String),
    TokenError(String),
    FilenameError,
}

#[derive(Debug, PartialEq)]
pub struct Job 
    {
        pub work_operator:Operator ,
        pub pipeline:Pipeline
    }

#[derive(Debug, PartialEq)]
pub struct JobQueue 
    {
        pub sequence:Vec<Job>,
    }
pub fn parse(Vec<Token>) -> Result<JobQueue , ParseError>
{
    let mut job_queue = JobQueue {sequence Vec::new()};

    Ok(job_queue)

}
