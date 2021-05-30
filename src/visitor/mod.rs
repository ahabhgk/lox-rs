pub mod ast_printer;
pub mod interpreter;

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
