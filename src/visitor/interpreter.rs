use crate::{
    ast::{Expr, LiteralValue},
    token::{Token, TokenType},
    visitor::Visitor,
};

use std::{error::Error, fmt};

#[derive(Debug)]
pub enum RuntimeError {
    TypeError { token: Token, message: String },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError { token, message } => write!(
                f,
                "[Runtime Error: {} at {}] TypeError: {}",
                token.line, token.lexeme, message
            ),
        }
    }
}

impl Error for RuntimeError {}

pub struct Interpreter;

pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}

impl Object {
    fn equals(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::Nil, Object::Nil) => true,
            (_, Object::Nil) => false,
            (Object::Nil, _) => false,
            (Object::Boolean(left), Object::Boolean(right)) => left == right,
            (Object::Number(left), Object::Number(right)) => left == right,
            (Object::String(left), Object::String(right)) => left == right,
            _ => false,
        }
    }
}

impl Interpreter {
    pub fn interpret(&self, expression: &Expr) -> Result<String, RuntimeError> {
        self.evaluate(expression).map(|value| self.stringify(value))
    }

    fn stringify(&self, object: Object) -> String {
        match object {
            Object::Nil => "nil".to_string(),
            Object::Number(n) => n.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::String(s) => s,
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<Object, RuntimeError> {
        expr.accept(self)
    }

    fn number_operand_error(&self, operator: &Token) -> RuntimeError {
        RuntimeError::TypeError {
            token: operator.clone(),
            message: "Operand must be a number.".to_string(),
        }
    }

    fn is_truthy(&self, object: &Object) -> bool {
        match object {
            Object::Nil => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl Visitor<Result<Object, RuntimeError>> for Interpreter {
    fn visit_literal_expr(
        &self,
        value: &LiteralValue,
    ) -> Result<Object, RuntimeError> {
        match value {
            LiteralValue::Boolean(b) => Ok(Object::Boolean(*b)),
            LiteralValue::Nil => Ok(Object::Nil),
            LiteralValue::Number(n) => Ok(Object::Number(*n)),
            LiteralValue::String(s) => Ok(Object::String(s.clone())),
        }
    }

    fn visit_binary_expr(
        &self,
        left: &Expr,
        operator: &crate::token::Token,
        right: &Expr,
    ) -> Result<Object, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.r#type {
            TokenType::Greater => match (left, right) {
                (Object::Number(ln), Object::Number(rn)) => {
                    Ok(Object::Boolean(ln > rn))
                }
                _ => Err(self.number_operand_error(operator)),
            },
            TokenType::GreaterEqual => match (left, right) {
                (Object::Number(ln), Object::Number(rn)) => {
                    Ok(Object::Boolean(ln >= rn))
                }
                _ => Err(self.number_operand_error(operator)),
            },
            TokenType::Less => match (left, right) {
                (Object::Number(ln), Object::Number(rn)) => {
                    Ok(Object::Boolean(ln < rn))
                }
                _ => Err(self.number_operand_error(operator)),
            },
            TokenType::LessEqual => match (left, right) {
                (Object::Number(ln), Object::Number(rn)) => {
                    Ok(Object::Boolean(ln <= rn))
                }
                _ => Err(self.number_operand_error(operator)),
            },
            TokenType::Minus => match (left, right) {
                (Object::Number(ln), Object::Number(rn)) => {
                    Ok(Object::Number(ln - rn))
                }
                _ => Err(self.number_operand_error(operator)),
            },
            TokenType::Slash => match (left, right) {
                (Object::Number(ln), Object::Number(rn)) => {
                    Ok(Object::Number(ln / rn))
                }
                _ => Err(self.number_operand_error(operator)),
            },
            TokenType::Star => match (left, right) {
                (Object::Number(ln), Object::Number(rn)) => {
                    Ok(Object::Number(ln * rn))
                }
                _ => Err(self.number_operand_error(operator)),
            },
            TokenType::Plus => match (left, right) {
                (Object::Number(ln), Object::Number(rn)) => {
                    Ok(Object::Number(ln + rn))
                }
                (Object::String(ls), Object::String(rs)) => {
                    Ok(Object::String(ls + &rs))
                }
                _ => Err(RuntimeError::TypeError {
                    token: operator.clone(),
                    message: "Operands must be two numbers or two strings."
                        .to_string(),
                }),
            },
            TokenType::BangEqual => Ok(Object::Boolean(!left.equals(&right))),
            TokenType::EqualEqual => Ok(Object::Boolean(left.equals(&right))),
            _ => unreachable!(),
        }
    }

    fn visit_grouping_expr(
        &self,
        expression: &Expr,
    ) -> Result<Object, RuntimeError> {
        self.evaluate(expression)
    }

    fn visit_unary_expr(
        &self,
        operator: &Token,
        right: &Expr,
    ) -> Result<Object, RuntimeError> {
        let right = self.evaluate(right)?;
        match operator.r#type {
            TokenType::Minus => match right {
                Object::Number(n) => Ok(Object::Number(-n)),
                _ => Err(self.number_operand_error(operator)),
            },
            TokenType::Bang => Ok(Object::Boolean(!self.is_truthy(&right))),
            _ => unreachable!(),
        }
    }
}
