use crate::{
    syntex::{Expr, LiteralValue},
    token::{Token, TokenType},
};

use std::{error::Error, fmt};

macro_rules! matche_types {
    ($sel:ident, $($x:expr),* ) => {
        {
            if $($sel.check($x))||* {
                $sel.advance();
                true
            } else {
                false
            }
        }
    };
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { message: String, token: Token },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken { message, token } => match token.r#type {
                TokenType::EOF => write!(
                    f,
                    "[Parse Error: {} at end] {}",
                    token.line, message
                ),
                _ => write!(
                    f,
                    "[Parse Error: {} at {}] {}",
                    token.line, token.lexeme, message
                ),
            },
        }
    }
}

impl Error for ParseError {}

pub struct Parser<'a> {
    current: usize,
    tokens: &'a Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self { current: 0, tokens }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while matche_types!(self, TokenType::BangEqual, TokenType::EqualEqual) {
            let operator = (*self.previous()).clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while matche_types!(
            self,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
        ) {
            let operator = (*self.previous()).clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while matche_types!(self, TokenType::Plus, TokenType::Minus) {
            let operator = (*self.previous()).clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while matche_types!(self, TokenType::Slash, TokenType::Star) {
            let operator = (*self.previous()).clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if matche_types!(self, TokenType::Bang, TokenType::Minus) {
            let operator = (*self.previous()).clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let expr = match &self.peek().r#type {
            TokenType::False => Expr::Literal {
                value: LiteralValue::Boolean(false),
            },
            TokenType::True => Expr::Literal {
                value: LiteralValue::Boolean(true),
            },
            TokenType::Nil => Expr::Literal {
                value: LiteralValue::Nil,
            },
            TokenType::String { literal } => Expr::Literal {
                value: LiteralValue::String(literal.clone()),
            },
            TokenType::Number { literal } => Expr::Literal {
                value: LiteralValue::Number(*literal),
            },
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(
                    TokenType::RightParen,
                    "Expect ')' after expression.",
                )?;
                Expr::Grouping {
                    expression: Box::new(expr),
                }
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    token: (*self.peek()).clone(),
                    message: "Expect expression.".to_string(),
                })
            }
        };

        self.advance();
        Ok(expr)
    }

    fn consume(
        &mut self,
        r#type: TokenType,
        message: &str,
    ) -> Result<&Token, ParseError> {
        if self.check(r#type) {
            return Ok(self.advance());
        }
        Err(ParseError::UnexpectedToken {
            message: message.to_string(),
            token: (*self.peek()).clone(),
        })
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().r#type == TokenType::Semicolon {
                return;
            }
            match self.peek().r#type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn check(&self, r#type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().r#type == r#type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().r#type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .expect("Peek into end of token stream.")
    }

    fn previous(&self) -> &Token {
        self.tokens
            .get(self.current - 1)
            .expect("Previous was empty.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::Lexer, syntex::AstPrinter};

    #[test]
    fn test_parser() {
        let mut lexer = Lexer::new("-123 * 45.67");
        let tokens = lexer.scan().expect("Could not scan sample code.");

        let mut parser = Parser::new(tokens);
        let expression = parser.parse().expect("Could not parse sample code.");
        let printer = AstPrinter;

        assert_eq!(printer.print(expression), "(* (- 123) 45.67)");
    }
}