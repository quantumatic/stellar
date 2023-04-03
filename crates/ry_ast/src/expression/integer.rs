use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct IntegerLiteralExpression {
    pub literal: u64,
}

impl IntegerLiteralExpression {
    #[inline]
    pub const fn new(literal: u64) -> Self {
        Self { literal }
    }
}

impl From<IntegerLiteralExpression> for RawExpression {
    fn from(integer: IntegerLiteralExpression) -> Self {
        Self::Integer(integer)
    }
}
