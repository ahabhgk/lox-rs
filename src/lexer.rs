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
                    Some('=') => {
                        self.source.next();
                        self.add_token(TokenType::BangEqual, "!=")
                    }
                    _ => self.add_token(TokenType::Bang, "!"),
                },
                '=' => match self.source.peek() {
                    Some('=') => {
                        self.source.next();
                        self.add_token(TokenType::EqualEqual, "==")
                    }
                    _ => self.add_token(TokenType::Equal, "="),
                },
                '<' => match self.source.peek() {
                    Some('=') => {
                        self.source.next();
                        self.add_token(TokenType::LessEqual, "<=")
                    }
                    _ => self.add_token(TokenType::Less, "<"),
                },
                '>' => match self.source.peek() {
                    Some('=') => {
                        self.source.next();
                        self.add_token(TokenType::GreaterEqual, ">=")
                    }
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
                    self.add_token(
                        TokenType::String { literal: s.clone() },
                        &s,
                    );
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

    #[cfg(test)]
    mod tests {
        use super::*;

        // #[test]
        // fn test_single_tokens() {
        //     let mut lexer = Lexer::new();
        //     let input = "(){},.-+*;";
        //     let expected: [Token; 10] = [
        //         Token::new(TokenType::LeftParen, 1),
        //         Token::new(TokenType::RightParen, 1),
        //         Token::new(TokenType::LeftBrace, 1),
        //         Token::new(TokenType::RightBrace, 1),
        //         Token::new(TokenType::Comma, 1),
        //         Token::new(TokenType::Dot, 1),
        //         Token::new(TokenType::Minus, 1),
        //         Token::new(TokenType::Plus, 1),
        //         Token::new(TokenType::Star, 1),
        //         Token::new(TokenType::Semicolon, 1),
        //     ];
        //     lexer.scan_tokens(&mut input.chars().peekable()).unwrap();
        //     let mut token_iter = lexer.tokens.iter();
        //     for exp in expected.iter() {
        //         if let Some(actual) = token_iter.next() {
        //             assert_eq!(exp.token, actual.token);
        //         } else {
        //             panic!();
        //         }
        //     }
        // }

        // #[test]
        // fn test_double_tokens() {
        //     let mut lexer = Lexer::new();
        //     let input = "! != = == > >= < <=";
        //     let expected: [Token; 8] = [
        //         Token::new(TokenType::Bang, 1),
        //         Token::new(TokenType::BangEqual, 1),
        //         Token::new(TokenType::Equal, 1),
        //         Token::new(TokenType::EqualEqual, 1),
        //         Token::new(TokenType::Greater, 1),
        //         Token::new(TokenType::GreaterEqual, 1),
        //         Token::new(TokenType::Less, 1),
        //         Token::new(TokenType::LessEqual, 1),
        //     ];
        //     lexer.scan_tokens(&mut input.chars().peekable()).unwrap();
        //     let mut token_iter = lexer.tokens.iter();
        //     for exp in expected.iter() {
        //         if let Some(actual) = token_iter.next() {
        //             assert_eq!(exp.token, actual.token);
        //         } else {
        //             panic!();
        //         }
        //     }
        // }

        #[test]
        fn test_literal_tokens() {
            let input = r#"Test_Class _unused "my string" 0.1 123 123.45"#;
            let mut lexer = Lexer::new(input);
            let expected = vec![
                Token::new(TokenType::Identifier, "Test_Class", 1),
                Token::new(TokenType::Identifier, "_unused", 1),
                Token::new(
                    TokenType::String {
                        literal: "my string".to_string(),
                    },
                    "my string",
                    1,
                ),
                Token::new(TokenType::Number { literal: 0.1 }, "0.1", 1),
                Token::new(TokenType::Number { literal: 123f64 }, "123", 1),
                Token::new(TokenType::Number { literal: 123.45 }, "123.45", 1),
            ];
            let tokens = lexer.scan().unwrap();
            for (i, token) in expected.iter().enumerate() {
                assert_eq!(&tokens[i], token);
            }
        }

        // #[test]
        // fn test_reserved_tokens() {
        //     let mut lexer = Lexer::new();
        //     let input = "and class else false fun for if nil or print return super this true var while";
        //     let expected: [Token; 16] = [
        //         Token::new(TokenType::And, 1),
        //         Token::new(TokenType::Class, 1),
        //         Token::new(TokenType::Else, 1),
        //         Token::new(TokenType::False, 1),
        //         Token::new(TokenType::Fun, 1),
        //         Token::new(TokenType::For, 1),
        //         Token::new(TokenType::If, 1),
        //         Token::new(TokenType::Nil, 1),
        //         Token::new(TokenType::Or, 1),
        //         Token::new(TokenType::Print, 1),
        //         Token::new(TokenType::Return, 1),
        //         Token::new(TokenType::Super, 1),
        //         Token::new(TokenType::This, 1),
        //         Token::new(TokenType::True, 1),
        //         Token::new(TokenType::Var, 1),
        //         Token::new(TokenType::While, 1),
        //     ];
        //     lexer.scan_tokens(&mut input.chars().peekable()).unwrap();
        //     let mut token_iter = lexer.tokens.iter();
        //     for exp in expected.iter() {
        //         if let Some(token) = token_iter.next() {
        //             assert_eq!(exp.token, token.token);
        //         } else {
        //             panic!()
        //         }
        //     }
        // }

        // #[test]
        // fn test_mixed_tokens() {
        //     let mut lexer = Lexer::new();
        //     let input = r#"
        // if(i == 6)
        // {
        //     print "hey mom";
        // }
        // "#;
        //     let expected: [Token; 11] = [
        //         Token::new(TokenType::If, 1),
        //         Token::new(TokenType::LeftParen, 1),
        //         Token::new(
        //             TokenType::Literal(LiteralType::Identifier(
        //                 "i".to_string(),
        //             )),
        //             1,
        //         ),
        //         Token::new(TokenType::EqualEqual, 1),
        //         Token::new(TokenType::Literal(LiteralType::Number(6.)), 1),
        //         Token::new(TokenType::RightParen, 1),
        //         Token::new(TokenType::LeftBrace, 1),
        //         Token::new(TokenType::Print, 1),
        //         Token::new(
        //             TokenType::Literal(LiteralType::String(
        //                 "hey mom".to_string(),
        //             )),
        //             1,
        //         ),
        //         Token::new(TokenType::Semicolon, 1),
        //         Token::new(TokenType::RightBrace, 1),
        //     ];
        //     lexer.scan_tokens(&mut input.chars().peekable()).unwrap();
        //     let mut token_iter = lexer.tokens.iter();
        //     for exp in expected.iter() {
        //         if let Some(token) = token_iter.next() {
        //             assert_eq!(exp.token, token.token);
        //         } else {
        //             panic!()
        //         }
        //     }
        // }

        // #[test]
        // fn test_unexpected_tokens() {
        //     let mut lexer = Lexer::new();
        //     // Fails on first unexpected
        //     let input = "class main$";

        //     let result = lexer.scan_tokens(&mut input.chars().peekable());
        //     let expected = LexError::UnexpectedToken {
        //         found: '$',
        //         line: 1,
        //     };
        //     assert_eq!(
        //         result.unwrap_err().downcast::<LexError>().unwrap(),
        //         expected
        //     );
        // }

        // #[test]
        // fn test_invalid_number_format() {
        //     let mut lexer = Lexer::new();
        //     // Fails because non-digit comes after decimal point
        //     let input = "1.XXX";

        //     let result = lexer.scan_tokens(&mut input.chars().peekable());
        //     let expected = LexError::InvalidNumberFormat;
        //     assert_eq!(
        //         result.unwrap_err().downcast::<LexError>().unwrap(),
        //         expected
        //     );
        // }

        // #[test]
        // fn test_line_count() {
        //     let mut lexer = Lexer::new();
        //     let input = "1
        // // random comment2
        // 3
        // 4
        // 5";
        //     lexer.scan_tokens(&mut input.chars().peekable()).unwrap();

        //     assert_eq!(lexer.line, 5);
        // }
    }
}
