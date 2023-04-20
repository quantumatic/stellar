use super::{Expression, RawExpression};
use crate::token::Token;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
