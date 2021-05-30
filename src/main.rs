mod ast;
mod lexer;
mod parser;
mod token;
mod visitor;

use lexer::Lexer;
use parser::Parser;
use visitor::interpreter::Interpreter;

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
            interpreter: Interpreter,
        }
    }

    pub fn run_file(&self, path: &str) -> Result<(), io::Error> {
        let source = read_to_string(path)?;
        self.run(&source);
        Ok(())
    }

    pub fn run_prompt(&self) -> Result<(), io::Error> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        loop {
            writer.write(PROMPT.as_bytes())?;
            writer.flush()?;

            let mut line = String::new();
            reader.read_line(&mut line)?;

            match self.run(&line) {
                Err(e) => eprintln!("{}", e),
                Ok(res) => println!("{}", res),
            }
        }
    }

    fn run(&self, source: &str) -> Result<String, Box<dyn error::Error>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan()?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse()?;
        let res = self.interpreter.interpret(&expr)?;
        Ok(res)
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let lox = Lox::new();
    match std::env::args().nth(1) {
        Some(path) => lox.run_file(&path)?,
        None => lox.run_prompt()?,
    };
    Ok(())
}
