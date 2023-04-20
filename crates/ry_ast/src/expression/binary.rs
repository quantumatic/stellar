use super::{Expression, RawExpression};
use crate::token::Token;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub op: Token,
}

impl From<BinaryExpression> for RawExpression {
    fn from(binary: BinaryExpression) -> Self {
        Self::Binary(binary)
    }
}
