use super::{Expression, RawExpression};
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub struct UnaryExpression {
    pub inner: Box<Expression>,
    pub op: Token,
    pub postfix: bool,
}

impl UnaryExpression {
    #[inline]
    pub fn new(inner: Expression, op: Token, postfix: bool) -> Self {
        Self {
            inner: Box::new(inner),
            op,
            postfix,
        }
    }
}

impl From<UnaryExpression> for RawExpression {
    fn from(unary: UnaryExpression) -> Self {
        Self::Unary(unary)
    }
}
