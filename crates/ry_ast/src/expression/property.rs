use super::Expression;
use crate::span::WithSpan;
use string_interner::DefaultSymbol;

#[derive(Debug, PartialEq)]
pub struct PropertyAccessExpression {
    left: Expression,
    right: WithSpan<DefaultSymbol>,
}

impl PropertyAccessExpression {
    #[inline]
    pub const fn left(&self) -> &Expression {
        &self.left
    }

    #[inline]
    pub const fn right(&self) -> &WithSpan<DefaultSymbol> {
        &self.right
    }
}
