use std::{collections::HashMap, error::Error, fmt, result};

use crate::{
    ast::{expr, stmt, Expr, Stmt},
    interpreter::Interpreter,
    token::Token,
};

#[derive(Debug)]
pub enum ResolveError {
    AlreadyDeclared { token: Token },
    ReadInOwnInitializer { token: Token },
    TopLevelReturn { token: Token },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyDeclared { token } => write!(
                f,
                "Identifier '{}' has already been declared (line {} at {}).",
                token.lexeme, token.line, token.lexeme
            ),
            Self::ReadInOwnInitializer { token } => write!(
                f,
                "Cannot read local variable '{}' in its own initializer (line {} at {}).",
                token.lexeme, token.line, token.lexeme,
            ),
            Self::TopLevelReturn { token } => write!(
                f,
                "Cannot return from top-level code (line {} at {}).",
                token.line, token.lexeme,
            ),
        }
    }
}

impl Error for ResolveError {}

pub type Result<T> = result::Result<T, ResolveError>;

#[derive(Debug, Clone)]
enum FunctionType {
    None,
    Function,
}

pub struct Resolver<'i> {
    interpreter: &'i mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'i> Resolver<'i> {
    pub fn new(interpreter: &'i mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn resolve_stmts(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<()> {
        statement.accept(self)
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err(ResolveError::AlreadyDeclared {
                    token: name.clone(),
                });
            }
            scope.insert(name.lexeme.clone(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_expr(&mut self, expression: &Expr) -> Result<()> {
        expression.accept(self)
    }

    fn resolve_local(&mut self, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                dbg!(i, name);
                self.interpreter.resolve(name, i);
            }
        }
    }

    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
        func_type: FunctionType,
    ) -> Result<()> {
        let enclosing_function = self.current_function.clone();
        self.current_function = func_type;
        self.begin_scope();

        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_stmts(body)?;

        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }
}

impl<'i> expr::Visitor<Result<()>> for Resolver<'i> {
    fn visit_variable_expr(&mut self, name: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last() {
            if let Some(flag) = scope.get(&name.lexeme) {
                if *flag == false {
                    return Err(ResolveError::ReadInOwnInitializer {
                        token: name.clone(),
                    });
                }
            }
        };
        self.resolve_local(name);
        Ok(())
    }

    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        _operator: &Token,
        right: &Expr,
    ) -> Result<()> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<()> {
        self.resolve_expr(expression)
    }

    fn visit_literal_expr(
        &mut self,
        _value: &crate::ast::LiteralValue,
    ) -> Result<()> {
        Ok(())
    }

    fn visit_logical_expr(
        &mut self,
        left: &Expr,
        _operator: &Token,
        right: &Expr,
    ) -> Result<()> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_unary_expr(
        &mut self,
        _operator: &Token,
        right: &Expr,
    ) -> Result<()> {
        self.resolve_expr(right)
    }

    fn visit_assign_expr(&mut self, name: &Token, expr: &Expr) -> Result<()> {
        self.resolve_expr(expr)?;
        self.resolve_local(name);
        Ok(())
    }

    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        _paren: &Token,
        arguments: &Vec<Expr>,
    ) -> Result<()> {
        self.resolve_expr(callee)?;
        for argument in arguments {
            self.resolve_expr(argument)?;
        }
        Ok(())
    }
}

impl<'i> stmt::Visitor<Result<()>> for Resolver<'i> {
    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        self.begin_scope();
        self.resolve_stmts(statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_var_stmt(
        &mut self,
        name: &Token,
        initializer: &Option<Expr>,
    ) -> Result<()> {
        self.declare(name)?;
        if let Some(init) = initializer {
            self.resolve_expr(init)?;
        }
        self.define(name);
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<()> {
        self.resolve_expr(expression)
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
    ) -> Result<()> {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(params, body, FunctionType::Function)?;
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Stmt>,
    ) -> Result<()> {
        self.resolve_expr(condition)?;
        self.resolve_stmt(then_branch)?;
        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<()> {
        self.resolve_expr(expression)
    }

    fn visit_return_stmt(
        &mut self,
        keyword: &Token,
        value: &Option<Expr>,
    ) -> Result<()> {
        if let FunctionType::None = self.current_function {
            return Err(ResolveError::TopLevelReturn {
                token: keyword.clone(),
            });
        }

        if let Some(value) = value {
            self.resolve_expr(value)?;
        }
        Ok(())
    }

    fn visit_while_stmt(
        &mut self,
        condition: &Expr,
        body: &Stmt,
    ) -> Result<()> {
        self.resolve_expr(condition)?;
        self.resolve_stmt(body)?;
        Ok(())
    }
}
