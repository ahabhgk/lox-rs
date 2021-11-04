use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::Stmt,
    environment::Environment,
    interpreter::{Interpreter, InterpretError},
    token::Token,
};

#[derive(Clone, Debug)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
    Callable(Function),
}

impl Object {
    pub fn equals(&self, other: &Object) -> bool {
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

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Object::Nil => "nil".to_string(),
            Object::Number(n) => n.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::String(s) => s.to_string(),
            Object::Callable(f) => f.to_string(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub enum Function {
    Native {
        arity: usize,
        body: Box<fn(&Vec<Object>) -> Object>,
    },
    User {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    },
}

impl Function {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<Object>,
    ) -> Result<Object, InterpretError> {
        match self {
            Function::Native { body, .. } => Ok(body(arguments)),
            Function::User {
                params,
                body,
                closure,
                ..
            } => {
                let environment =
                    Rc::new(RefCell::new(Environment::from(closure)));
                for (param, argument) in params.iter().zip(arguments.iter()) {
                    environment
                        .borrow_mut()
                        .define(param.lexeme.clone(), argument.clone());
                }
                match interpreter.execute_block(body, environment) {
                    Err(InterpretError::Return { value }) => Ok(value),
                    Err(other) => Err(other),
                    Ok(_) => Ok(Object::Nil),
                }
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Native { arity, .. } => *arity,
            Function::User { params, .. } => params.len(),
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native { .. } => write!(f, "<native func>"),
            Function::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Native { .. } => write!(f, "<native func>"),
            Function::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}
