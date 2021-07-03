mod ast;
mod ast_printer;
mod environment;
mod interpreter;
mod lexer;
mod object;
mod parser;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
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

    pub fn run_file(&mut self, path: &str) -> Result<(), io::Error> {
        let source = read_to_string(path)?;
        self.run(&source);
        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<(), io::Error> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        loop {
            writer.write(PROMPT.as_bytes())?;
            writer.flush()?;

            let mut line = String::new();
            reader.read_line(&mut line)?;

            self.run(&line);
        }
    }

    fn run(&mut self, source: &str) {
        let mut lexer = Lexer::new(source);
        let tokens = match lexer.scan() {
            Ok(tokens) => tokens,
            Err(e) => return eprintln!("[Lex Error] {}", e),
        };
        let mut parser = Parser::new(tokens);
        let statements = match parser.parse() {
            Ok(stmts) => stmts,
            Err(e) => return eprintln!("[Parse Error] {}", e),
        };
        match self.interpreter.interpret(&statements) {
            Ok(_) => {}
            Err(e) => return eprintln!("[Runtime Error] {}", e),
        };
    }

    fn debug_run(&mut self, source: &str) -> Result<(), Box<dyn error::Error>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        self.interpreter.interpret(&statements)?;
        Ok(())
    }
}

fn main() {
    let mut lox = Lox::new();
    match std::env::args().nth(1) {
        Some(path) => {
            if let Err(e) = lox.run_file(&path) {
                eprintln!("{}", e);
            }
        }
        None => {
            if let Err(e) = lox.run_prompt() {
                eprintln!("{}", e);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::Lox;
    use std::{error, fs::read_to_string};

    fn run_case(path: &str) -> Result<(), Box<dyn error::Error>> {
        let mut lox = Lox::new();
        let source = read_to_string(path)?;
        lox.debug_run(&source)
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
}
