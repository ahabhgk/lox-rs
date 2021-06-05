use crate::token::Token;
use std::fmt;

pub enum LiteralValue {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Boolean(b) => write!(f, "{}", b),
            LiteralValue::Nil => write!(f, "nil"),
            LiteralValue::Number(n) => write!(f, "{}", n),
            LiteralValue::String(s) => write!(f, "{}", s),
        }
    }
}

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

impl Expr {
    pub fn accept<R>(&self, visitor: &impl expr::Visitor<R>) -> R {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => {
                visitor.visit_grouping_expr(expression)
            }
            Expr::Literal { value } => visitor.visit_literal_expr(value),
            Expr::Unary { operator, right } => {
                visitor.visit_unary_expr(operator, right)
            }
            Expr::Variable { name } => visitor.visit_variable_expr(name),
        }
    }
}

pub mod expr {
    use super::{Expr, LiteralValue};
    use crate::token::Token;

    pub trait Visitor<R> {
        fn visit_binary_expr(
            &self,
            left: &Expr,
            operator: &Token,
            right: &Expr,
        ) -> R;
        fn visit_grouping_expr(&self, expression: &Expr) -> R;
        fn visit_literal_expr(&self, value: &LiteralValue) -> R;
        fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> R;
        fn visit_variable_expr(&self, name: &Token) -> R;
    }
}

pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Nil,
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut impl stmt::Visitor<R>) -> R {
        match self {
            Stmt::Block { statements } => visitor.visit_block_stmt(statements),
            Stmt::Expression { expression } => {
                visitor.visit_expression_stmt(expression)
            }
            Stmt::Print { expression } => visitor.visit_print_stmt(expression),
            Stmt::Var { name, initializer } => {
                visitor.visit_var_stmt(name, initializer)
            }
            Stmt::Nil => unimplemented!(),
        }
    }
}

pub mod stmt {
    use super::{Expr, Stmt};
    use crate::token::Token;

    pub trait Visitor<R> {
        fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> R;
        fn visit_expression_stmt(&mut self, expression: &Expr) -> R;
        fn visit_print_stmt(&mut self, expression: &Expr) -> R;
        fn visit_var_stmt(
            &mut self,
            name: &Token,
            initializer: &Option<Expr>,
        ) -> R;
    }
}
