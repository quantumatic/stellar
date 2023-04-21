use super::{Expression, RawExpression};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ParenthesizedExpression {
    pub inner: Box<Expression>,
}

impl From<ParenthesizedExpression> for RawExpression {
    fn from(parenthesized: ParenthesizedExpression) -> Self {
        RawExpression::Parenthesized(parenthesized)
    }
}
