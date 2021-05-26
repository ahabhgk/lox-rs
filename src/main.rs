mod lexer;
mod syntex;
mod token;

use lexer::{LexError, Lexer};
use std::{
    error,
    fs::read_to_string,
    io::{self, BufRead, Write},
};

fn main() -> Result<(), Box<dyn error::Error>> {
    match std::env::args().nth(1) {
        Some(path) => run_file(&path)?,
        None => run_prompt()?,
    };
    Ok(())
}

fn run_file(path: &str) -> Result<(), Box<dyn error::Error>> {
    let source = read_to_string(path)?;
    run(&source)?;
    Ok(())
}

const PROMPT: &'static str = "> ";

fn run_prompt() -> Result<(), io::Error> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        writer.write(PROMPT.as_bytes())?;
        writer.flush()?;

        let mut line = String::new();
        reader.read_line(&mut line)?;

        if let Err(e) = run(&line) {
            eprintln!("{}", e);
        }
    }
}

fn run(source: &str) -> Result<(), LexError> {
    let mut lexer = Lexer::new();
    lexer.scan_tokens(source.chars().peekable())?;
    for token in lexer.tokens {
        println!("{:?} ", token);
    }
    Ok(())
}
