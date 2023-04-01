use super::{Expression, RawExpression};

#[derive(Debug, PartialEq)]
pub struct ArrayLiteralExpression {
    literal: Vec<Expression>,
}

impl ArrayLiteralExpression {
    #[inline]
    pub const fn new(literal: Vec<Expression>) -> Self {
        Self { literal }
    }

    #[inline]
    pub const fn literal(&self) -> &Vec<Expression> {
        &self.literal
    }
}

impl From<ArrayLiteralExpression> for RawExpression {
    fn from(array: ArrayLiteralExpression) -> Self {
        Self::Array(array)
    }
}
