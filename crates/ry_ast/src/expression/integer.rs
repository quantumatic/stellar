use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct IntegerLiteralExpression {
    pub literal: u64,
}

impl From<IntegerLiteralExpression> for RawExpression {
    fn from(integer: IntegerLiteralExpression) -> Self {
        Self::Integer(integer)
    }
}
