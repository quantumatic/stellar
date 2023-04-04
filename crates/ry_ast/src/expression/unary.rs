use super::{Expression, RawExpression};
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub struct UnaryExpression {
    pub inner: Box<Expression>,
    pub op: Token,
    pub postfix: bool,
}

impl From<UnaryExpression> for RawExpression {
    fn from(unary: UnaryExpression) -> Self {
        Self::Unary(unary)
    }
}
