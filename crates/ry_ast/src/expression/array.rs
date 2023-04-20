use super::{Expression, RawExpression};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrayLiteralExpression {
    pub literal: Vec<Expression>,
}

impl From<ArrayLiteralExpression> for RawExpression {
    fn from(array: ArrayLiteralExpression) -> Self {
        Self::Array(array)
    }
}
