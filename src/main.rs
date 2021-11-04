mod ast;
mod ast_printer;
mod environment;
mod interpreter;
mod lexer;
mod object;
mod parser;
mod resolver;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use resolver::Resolver;
use std::{
    error,
    fs::read_to_string,
    io::{self, BufRead, Write},
};

pub struct Lox {
    pub interpreter: Interpreter,
}

const PROMPT: &'static str = "> ";

impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, path: &str) {
        let source = read_to_string(path).unwrap();
        if let Err(e) = self.run(&source) {
            eprintln!("{}", e);
        }
    }

    pub fn run_prompt(&mut self) {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        loop {
            writer.write(PROMPT.as_bytes()).unwrap();
            writer.flush().unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            if let Err(e) = self.run(&line) {
                eprintln!("{}", e);
            }
        }
    }

    fn run(&mut self, source: &str) -> Result<(), Box<dyn error::Error>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan()?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;

        let mut resolver = Resolver::new(&mut self.interpreter);
        resolver.resolve_stmts(&statements)?;

        self.interpreter.interpret(&statements)?;
        Ok(())
    }
}

fn main() {
    let mut lox = Lox::new();
    match std::env::args().nth(1) {
        Some(path) => lox.run_file(&path),
        None => lox.run_prompt(),
    };
}

#[cfg(test)]
mod tests {
    use crate::Lox;
    use std::{error, fs::read_to_string};

    fn run_case(path: &str) -> Result<(), Box<dyn error::Error>> {
        let mut lox = Lox::new();
        let source = read_to_string(path)?;
        lox.run(&source)
    }

    #[test]
    fn test_enclosing() {
        assert!(run_case("./examples/enclosing.lox").is_ok())
    }

    #[test]
    fn test_for() {
        assert!(run_case("./examples/for.lox").is_ok())
    }

    #[test]
    fn test_or_and() {
        assert!(run_case("./examples/or-and.lox").is_ok())
    }

    #[test]
    fn test_fib() {
        assert!(run_case("./examples/fib.lox").is_ok())
    }

    #[test]
    fn test_closure() {
        assert!(run_case("./examples/closure.lox").is_ok())
    }

    #[test]
    fn test_inner_outer() {
        assert!(run_case("./examples/inner_outer.lox").is_ok())
    }
}
