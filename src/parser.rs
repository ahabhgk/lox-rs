use crate::{
    ast::{Expr, LiteralValue, Stmt},
    token::{Token, TokenType},
};
use std::{error::Error, fmt, result};

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
    InvalidAssignment { token: Token, message: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken { message, token } => match token.r#type {
                TokenType::EOF => {
                    write!(
                        f,
                        "Unexpected token (line {} at end) {}",
                        token.line, message
                    )
                }
                _ => write!(
                    f,
                    "Unexpected token (line {} at {}) {}",
                    token.line, token.lexeme, message
                ),
            },
            Self::InvalidAssignment { token, message } => write!(
                f,
                "Invalid assignment (line {} at {}) {}",
                token.line, token.lexeme, message
            ),
        }
    }
}

impl Error for ParseError {}

pub type Result<T> = result::Result<T, ParseError>;

pub struct Parser<'a> {
    current: usize,
    tokens: &'a Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self { current: 0, tokens }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        let statement = if matche_types!(self, TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };
        statement
        // match statement {
        //     Err(_) => {
        //         self.synchronize();
        //         Ok(Stmt::Nil)
        //     }
        //     other => other,
        // }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .clone();
        let initializer = if matche_types!(self, TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if matche_types!(self, TokenType::If) {
            self.if_statement()
        } else if matche_types!(self, TokenType::Print) {
            self.print_statement()
        } else if matche_types!(self, TokenType::LeftBrace) {
            Ok(Stmt::Block {
                statements: self.block()?,
            })
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if matche_types!(self, TokenType::Eles) {
            Box::new(Some(self.statement()?))
        } else {
            Box::new(None)
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression { expression: value })
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;
        if matche_types!(self, TokenType::Equal) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            return match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => Err(ParseError::InvalidAssignment {
                    token: equals,
                    message: "Invalid assignment target.".to_string(),
                }),
            };
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;
        while matche_types!(self, TokenType::Or) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;
        while matche_types!(self, TokenType::And) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        while matche_types!(self, TokenType::BangEqual, TokenType::EqualEqual) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;
        while matche_types!(
            self,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
        ) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;
        while matche_types!(self, TokenType::Plus, TokenType::Minus) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;
        while matche_types!(self, TokenType::Slash, TokenType::Star) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if matche_types!(self, TokenType::Bang, TokenType::Minus) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        let expr = match &self.peek().r#type {
            TokenType::False => {
                self.advance();
                Expr::Literal {
                    value: LiteralValue::Boolean(false),
                }
            }
            TokenType::True => {
                self.advance();
                Expr::Literal {
                    value: LiteralValue::Boolean(true),
                }
            }
            TokenType::Nil => {
                self.advance();
                Expr::Literal {
                    value: LiteralValue::Nil,
                }
            }
            TokenType::String { literal } => {
                let literal = literal.clone();
                self.advance();
                Expr::Literal {
                    value: LiteralValue::String(literal),
                }
            }
            TokenType::Number { literal } => {
                let literal = *literal;
                self.advance();
                Expr::Literal {
                    value: LiteralValue::Number(literal),
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(
                    TokenType::RightParen,
                    "Expect ')' after expression.",
                )?;
                Expr::Grouping {
                    expression: Box::new(expr),
                }
            }
            TokenType::Identifier => {
                self.advance();
                Expr::Variable {
                    name: self.previous().clone(),
                }
            }
            _ => {
                self.advance();
                return Err(ParseError::UnexpectedToken {
                    token: self.previous().clone(),
                    message: "Expect expression.".to_string(),
                });
            }
        };
        Ok(expr)
    }

    fn consume(&mut self, r#type: TokenType, message: &str) -> Result<&Token> {
        if self.check(r#type) {
            return Ok(self.advance());
        }
        Err(ParseError::UnexpectedToken {
            message: message.to_string(),
            token: self.peek().clone(),
        })
    }

    // fn synchronize(&mut self) {
    //     self.advance();
    //     while !self.is_at_end() {
    //         if self.previous().r#type == TokenType::Semicolon {
    //             return;
    //         }
    //         match self.peek().r#type {
    //             TokenType::Class
    //             | TokenType::Fun
    //             | TokenType::Var
    //             | TokenType::For
    //             | TokenType::If
    //             | TokenType::While
    //             | TokenType::Print
    //             | TokenType::Return => return,
    //             _ => {}
    //         }
    //         self.advance();
    //     }
    // }

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
    // use super::*;
    // use crate::ast_printer::AstPrinter;
    // use crate::lexer::Lexer;

    // #[test]
    // fn test_expression() {
    //     let mut lexer = Lexer::new("-123 * 45.67");
    //     let tokens = lexer.scan().expect("Could not scan sample code.");

    //     let mut parser = Parser::new(tokens);
    //     let statements = parser.parse().expect("Could not parse sample code.");
    //     let printer = AstPrinter;

    //     assert_eq!(printer.print(statements), "(* (- 123) 45.67)");
    // }
}
