# minish

`minish` is a lightweight, custom Unix shell written in pure Rust using
only the standard library (`std`). It is designed to act as the process
layer of the OS, handling tokenization, parsing, quoting, multi-stage
concurrent pipelines, and I/O redirection.

## Features

-   **Concurrent Pipelines:** Pipes (`|`) execute all stages
    concurrently, correctly wiring `stdout` to `stdin` without
    deadlocks.
-   **I/O Redirection:** Supports input (`<`), output truncation (`>`),
    and output appending (`>>`).
-   **Quoting:** Supports single (`'...'`) and double (`"..."`) quotes.
    Operators inside quotes are treated as literals.
-   **Logical Chaining:** Supports conditional execution using `&&`
    (AND) and `||` (OR).
-   **Variable Expansion:** Expands environment variables (`$VAR`) and
    the exit status of the last command (`$?`). Single quotes prevent
    expansion.
-   **Builtins:** Executes `cd`, `pwd`, and `exit` directly in the shell
    process.

## Usage

Build and run:

``` bash
cargo run
```

Or run a single command:

``` bash
cargo run -- -c "ls -l | grep .rs | wc -l"
```

## Examples

``` bash
minish> echo "Hello World" > output.txt
minish> cat < output.txt | wc -c
12
minish> ls nosuchfile || echo "Command failed!"
ls: cannot access 'nosuchfile': No such file or directory
Command failed!
minish> cd /tmp
minish> pwd
/tmp
```

## Architecture

The shell is divided into pure, testable stages:

-   **Lexer (`lexer.rs`)**: Converts raw input into tokens while
    handling whitespace and quoted strings.
-   **Parser (`parser.rs`)**: Builds a `JobQueue` AST from tokens,
    creating commands, pipelines, and logical chains.
-   **Execution (`exec.rs`)**: Executes commands and concurrent
    pipelines using `std::process::Command` and `Stdio::piped()`.

## The `cd` Question

### Why can't `cd` be an external program?

In Unix-like operating systems, every process maintains its own current
working directory.

When the shell launches an external program, it creates a child process.
If `cd` were an external program, it would change the working directory
of only that child process. Once the child exits, the parent shell would
still be in its original directory.

For this reason, commands that modify the shell's own state---such as
`cd`, `exit`, and variable assignments---must be implemented as shell
built-ins rather than external executables.
