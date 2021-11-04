use crate::object::Object;
use crate::{
    interpreter::{InterpretError, Result},
    token::Token,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn from(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Environment {
            enclosing: Some(Rc::clone(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }
        Err(InterpretError::UndefinedError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Object {
        let key = &*name.lexeme;
        let obj = match self.ancestor(distance) {
            Some(env) => env.borrow().values.get(key).cloned(),
            None => self.values.get(key).cloned(),
        };
        dbg!(&obj, distance, name, key, &self.values, &self.enclosing);
        obj.expect(&format!("Undefined variable '{}'", key))
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_string(), value);
            return Ok(());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }
        Err(InterpretError::UndefinedError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Object) {
        let key = name.lexeme.clone();
        match self.ancestor(distance) {
            Some(env) => env.borrow_mut().values.insert(key, value),
            None => self.values.insert(key, value),
        };
    }

    fn ancestor(&self, distance: usize) -> Option<Rc<RefCell<Environment>>> {
        let mut environment = None;
        for i in 0..distance {
            let parent = self
                .enclosing
                .clone()
                .expect(&format!("No enclosing environment at {}", i));
            environment = Some(Rc::clone(&parent));
        }
        environment
    }
}
