use super::{Expression, RawExpression};
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub struct UnaryExpression {
    inner: Expression,
    op: Token,
    postfix: bool,
}

impl UnaryExpression {
    #[inline]
    pub const fn new(inner: Expression, op: Token, postfix: bool) -> Self {
        Self { inner, op, postfix }
    }

    #[inline]
    pub const fn inner(&self) -> &Expression {
        &self.inner
    }

    #[inline]
    pub const fn op(&self) -> &Token {
        &self.op
    }

    #[inline]
    pub const fn postfix(&self) -> bool {
        self.postfix
    }
}

impl From<UnaryExpression> for RawExpression {
    fn from(unary: UnaryExpression) -> Self {
        Self::Unary(unary)
    }
}
