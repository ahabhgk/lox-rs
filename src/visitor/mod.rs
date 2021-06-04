pub mod ast_printer;
pub mod interpreter;

pub mod expr {
    use crate::{
        ast::{Expr, LiteralValue},
        token::Token,
    };

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
    }
}

pub mod stmt {
    use crate::{
        ast::{Expr, Stmt},
        token::Token,
    };

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
