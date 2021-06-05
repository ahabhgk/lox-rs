use crate::{
    ast::{expr, stmt, Expr, LiteralValue, Stmt},
    environment::Environment,
    object::Object,
    token::{Token, TokenType},
};
use std::{cell::RefCell, error::Error, fmt, rc::Rc, result};

#[derive(Debug)]
pub enum RuntimeError {
    TypeError { token: Token, message: String },
    UndefinedError { token: Token, message: String },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError { token, message } => write!(
                f,
                "TypeError (line {} at {}) {}",
                token.line, token.lexeme, message
            ),
            Self::UndefinedError { token, message } => write!(
                f,
                "UndefinedError (line {} at {}) {}",
                token.line, token.lexeme, message
            ),
        }
    }
}

impl Error for RuntimeError {}

pub type Result<T> = result::Result<T, RuntimeError>;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        env: Rc<RefCell<Environment>>,
    ) -> Result<()> {
        let previous = Rc::clone(&self.environment);
        self.environment = env;
        for stmt in statements {
            self.execute(stmt)?;
        }
        self.environment = previous;
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
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

impl expr::Visitor<Result<Object>> for Interpreter {
    fn visit_literal_expr(&self, value: &LiteralValue) -> Result<Object> {
        match value {
            LiteralValue::Boolean(b) => Ok(Object::Boolean(*b)),
            LiteralValue::Nil => Ok(Object::Nil),
            LiteralValue::Number(n) => Ok(Object::Number(*n)),
            LiteralValue::String(s) => Ok(Object::String(s.clone())),
        }
    }

    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        operator: &crate::token::Token,
        right: &Expr,
    ) -> Result<Object> {
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

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<Object> {
        self.evaluate(expression)
    }

    fn visit_unary_expr(
        &mut self,
        operator: &Token,
        right: &Expr,
    ) -> Result<Object> {
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

    fn visit_variable_expr(&self, name: &Token) -> Result<Object> {
        self.environment.borrow().get(name)
    }

    fn visit_assign_expr(
        &mut self,
        name: &Token,
        value: &Expr,
    ) -> Result<Object> {
        let value = self.evaluate(value)?;
        self.environment.borrow_mut().assgin(name, value.clone())?;
        Ok(value)
    }
}

impl stmt::Visitor<Result<()>> for Interpreter {
    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        self.execute_block(
            statements,
            Rc::new(RefCell::new(Environment::from(&self.environment))),
        )
    }

    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<()> {
        self.evaluate(expression)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<()> {
        let value = self.evaluate(expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(
        &mut self,
        name: &Token,
        initializer: &Option<Expr>,
    ) -> Result<()> {
        let value = initializer
            .as_ref()
            .map(|v| self.evaluate(&v))
            .unwrap_or(Ok(Object::Nil))?;
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);
        Ok(())
    }
}
