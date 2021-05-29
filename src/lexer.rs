use crate::token::{Token, TokenType};

use std::{error::Error, fmt, iter::Peekable, str::Chars};

#[derive(Debug)]
pub enum LexError {
    UnexpectedCharacter { char: char, line: usize },
    UnterminatedString { char: char, line: usize },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { char, line } => {
                write!(
                    f,
                    "[Lex Error: {} at {}] Unexpected character {}",
                    line, char, char
                )
            }
            Self::UnterminatedString { char, line } => {
                write!(
                    f,
                    "[Lex Error: {} at {}] Unterminated string",
                    line, char
                )
            }
        }
    }
}

impl Error for LexError {}

pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    pub tokens: Vec<Token>,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            tokens: Vec::new(),
            line: 1,
        }
    }

    pub fn scan(&mut self) -> Result<&Vec<Token>, LexError> {
        while let Some(c) = self.source.next() {
            match c {
                '(' => self.add_token(TokenType::LeftParen, "("),
                ')' => self.add_token(TokenType::RightParen, ")"),
                '{' => self.add_token(TokenType::LeftBrace, "{"),
                '}' => self.add_token(TokenType::RightBrace, "}"),
                ',' => self.add_token(TokenType::Comma, ","),
                '.' => self.add_token(TokenType::Dot, "."),
                '-' => self.add_token(TokenType::Minus, "-"),
                '+' => self.add_token(TokenType::Plus, "+"),
                ';' => self.add_token(TokenType::Semicolon, ";"),
                '*' => self.add_token(TokenType::Star, "*"),
                '!' => match self.source.peek() {
                    Some('=') => self.add_token(TokenType::BangEqual, "!="),
                    _ => self.add_token(TokenType::Bang, "!"),
                },
                '=' => match self.source.peek() {
                    Some('=') => self.add_token(TokenType::EqualEqual, "=="),
                    _ => self.add_token(TokenType::Equal, "="),
                },
                '<' => match self.source.peek() {
                    Some('=') => self.add_token(TokenType::LessEqual, "<="),
                    _ => self.add_token(TokenType::Less, "<"),
                },
                '>' => match self.source.peek() {
                    Some('=') => self.add_token(TokenType::GreaterEqual, ">="),
                    _ => self.add_token(TokenType::Greater, ">"),
                },
                '/' => match self.source.peek() {
                    Some('/') => loop {
                        match self.source.next() {
                            Some('\n') | None => break,
                            _ => {}
                        }
                    },
                    _ => self.add_token(TokenType::Slash, "/"),
                },
                '"' => {
                    let mut s = String::new();
                    loop {
                        match self.source.next() {
                            Some('"') => break,
                            Some('\n') => self.line += 1,
                            Some(c) => s.push(c),
                            None => {
                                return Err(LexError::UnterminatedString {
                                    char: '"',
                                    line: self.line,
                                });
                            }
                        }
                    }
                    self.add_token(TokenType::String { literal: s }, "\"");
                }
                '0'..='9' => {
                    let mut n = String::from(c);
                    while let Some(&c) = self.source.peek() {
                        if c.is_ascii_digit() {
                            self.source.next();
                            n.push(c);
                        } else {
                            break;
                        }
                    }
                    if let Some('.') = self.source.peek() {
                        self.source.next();
                        if let Some('0'..='9') = self.source.peek() {
                            n.push('.');
                            while let Some(&num) = self.source.peek() {
                                if num.is_ascii_digit() {
                                    self.source.next();
                                    n.push(num);
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    self.add_token(
                        TokenType::Number {
                            literal: n.parse::<f64>().unwrap(),
                        },
                        &n,
                    );
                }
                'o' => {
                    if let Some('r') = self.source.peek() {
                        self.source.next();
                        self.add_token(TokenType::Or, "or");
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::from(c);
                    while let Some(&c) = self.source.peek() {
                        if c.is_ascii_alphabetic()
                            || c.is_ascii_digit()
                            || c == '_'
                        {
                            self.source.next();
                            ident.push(c);
                        } else {
                            break;
                        }
                    }
                    match Token::get_keyword(&ident) {
                        Some(r#type) => self.add_token(r#type, &ident),
                        None => self.add_token(TokenType::Identifier, &ident),
                    }
                }
                '\n' => self.line += 1,
                ' ' | '\r' | '\t' => {}
                _ => {
                    return Err(LexError::UnexpectedCharacter {
                        char: c,
                        line: self.line,
                    })
                }
            }
        }

        self.add_token(TokenType::EOF, "");
        Ok(&self.tokens)
    }

    fn add_token(&mut self, r#type: TokenType, lexeme: &str) {
        self.tokens.push(Token::new(r#type, lexeme, self.line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let mut lexer = Lexer::new("-123 * 45.67");
        let tokens = lexer.scan().expect("Could not scan sample code.");

        let tokens = tokens
            .iter()
            .map(|t| t.lexeme.clone())
            .collect::<Vec<String>>()
            .join(" ");
        assert_eq!(&tokens, "- 123 * 45.67 ");
    }
}
