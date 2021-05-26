use crate::token::{Token, TokenType};
use std::{error::Error, fmt, iter::Peekable, str::Chars};

#[derive(Debug)]
pub enum LexError {
    UnexpectedCharacter { char: char, line: usize },
    UnterminatedString { line: usize },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { char, line } => {
                write!(f, "Unexpected character: {} at line {}", char, line)
            }
            Self::UnterminatedString { line } => {
                write!(f, "Unterminated string at line {}", line)
            }
        }
    }
}

impl Error for LexError {}

pub struct Lexer {
    pub tokens: Vec<Token>,
    line: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            line: 1,
        }
    }

    pub fn scan_tokens<'a>(
        &mut self,
        mut source: Peekable<Chars<'a>>,
    ) -> Result<(), LexError> {
        while let Some(c) = source.next() {
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
                '!' => match source.peek() {
                    Some('=') => self.add_token(TokenType::BangEqual, "!="),
                    _ => self.add_token(TokenType::Bang, "!"),
                },
                '=' => match source.peek() {
                    Some('=') => self.add_token(TokenType::EqualEqual, "=="),
                    _ => self.add_token(TokenType::Equal, "="),
                },
                '<' => match source.peek() {
                    Some('=') => self.add_token(TokenType::LessEqual, "<="),
                    _ => self.add_token(TokenType::Less, "<"),
                },
                '>' => match source.peek() {
                    Some('=') => self.add_token(TokenType::GreaterEqual, ">="),
                    _ => self.add_token(TokenType::Greater, ">"),
                },
                '/' => match source.peek() {
                    Some('/') => loop {
                        match source.next() {
                            Some('\n') | None => break,
                            _ => {}
                        }
                    },
                    _ => self.add_token(TokenType::Slash, "/"),
                },
                '"' => {
                    let mut s = String::new();
                    loop {
                        match source.next() {
                            Some('"') => break,
                            Some('\n') => self.line += 1,
                            Some(c) => s.push(c),
                            None => {
                                return Err(LexError::UnterminatedString {
                                    line: self.line,
                                });
                            }
                        }
                    }
                    self.add_token(TokenType::String { literal: s }, "\"");
                }
                '0'..='9' => {
                    let mut n = String::from(c);
                    while let Some(c) = source.next() {
                        if c.is_ascii_digit() {
                            n.push(c);
                        } else {
                            break;
                        }
                    }
                    if let Some('.') = source.next() {
                        if let Some('0'..='9') = source.peek() {
                            n.push('.');
                        }
                        while let Some(num) = source.next() {
                            if num.is_ascii_digit() {
                                n.push(num);
                            } else {
                                break;
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
                    if let Some('r') = source.next() {
                        self.add_token(TokenType::Or, "or");
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::from(c);
                    while let Some(c) = source.next() {
                        if c.is_ascii_alphabetic()
                            || c.is_ascii_digit()
                            || c == '_'
                        {
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
        Ok(())
    }

    fn add_token(&mut self, r#type: TokenType, lexeme: &str) {
        self.tokens.push(Token::new(r#type, lexeme, self.line))
    }
}
