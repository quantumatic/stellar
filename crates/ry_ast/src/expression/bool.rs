use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct BoolLiteralExpression {
    pub literal: bool,
}

impl BoolLiteralExpression {
    #[inline]
    pub const fn new(literal: bool) -> Self {
        Self { literal }
    }
}

impl From<BoolLiteralExpression> for RawExpression {
    fn from(bool: BoolLiteralExpression) -> Self {
        Self::Bool(bool)
    }
}
