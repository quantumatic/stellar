use super::{Expression, RawExpression};
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub op: Token,
}

impl BinaryExpression {
    #[inline]
    pub fn new(left: Expression, right: Expression, op: Token) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
            op,
        }
    }
}

impl From<BinaryExpression> for RawExpression {
    fn from(binary: BinaryExpression) -> Self {
        Self::Binary(binary)
    }
}
