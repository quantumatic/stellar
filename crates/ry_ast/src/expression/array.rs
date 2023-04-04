use super::{Expression, RawExpression};

#[derive(Debug, PartialEq)]
pub struct ArrayLiteralExpression {
    pub literal: Vec<Expression>,
}

impl From<ArrayLiteralExpression> for RawExpression {
    fn from(array: ArrayLiteralExpression) -> Self {
        Self::Array(array)
    }
}
