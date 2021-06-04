mod ast;
mod lexer;
mod parser;
mod token;
mod visitor;

use lexer::Lexer;
use parser::Parser;
use std::{
    fs::read_to_string,
    io::{self, BufRead, Write},
};
use visitor::interpreter::Interpreter;

pub struct Lox {
    pub interpreter: Interpreter,
}

const PROMPT: &'static str = "> ";

impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter,
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
