use super::{Expression, RawExpression};
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub struct BinaryExpression {
    left: Expression,
    right: Expression,
    op: Token,
}

impl BinaryExpression {
    #[inline]
    pub const fn left(&self) -> &Expression {
        &self.left
    }

    #[inline]
    pub const fn right(&self) -> &Expression {
        &self.right
    }

    #[inline]
    pub const fn op(&self) -> &Token {
        &self.op
    }
}

impl From<BinaryExpression> for RawExpression {
    fn from(binary: BinaryExpression) -> Self {
        Self::Binary(binary)
    }
}
