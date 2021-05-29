mod lexer;
mod parser;
mod syntex;
mod token;

use lexer::Lexer;
use parser::Parser;
use syntex::AstPrinter;

use std::{
    error,
    fs::read_to_string,
    io::{self, BufRead},
};

fn main() -> Result<(), Box<dyn error::Error>> {
    match std::env::args().nth(1) {
        Some(path) => run_file(&path)?,
        None => run_prompt()?,
    };
    Ok(())
}

fn run_file(path: &str) -> Result<(), io::Error> {
    let source = read_to_string(path)?;
    run(&source);
    Ok(())
}

const PROMPT: &'static str = "> ";

fn run_prompt() -> Result<(), io::Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        run(&line?);
        print!("{}", PROMPT);
    }
    Ok(())
}

fn run(source: &str) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan().unwrap();
    let mut parser = Parser::new(tokens);
    if let Ok(expr) = parser.parse() {
        println!("{}", AstPrinter.print(expr));
    }
}
