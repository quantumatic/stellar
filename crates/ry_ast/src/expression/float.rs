use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct FloatLiteralExpression {
    pub literal: f64,
}

impl FloatLiteralExpression {
    #[inline]
    pub const fn new(literal: f64) -> Self {
        Self { literal }
    }
}

impl From<FloatLiteralExpression> for RawExpression {
    fn from(float: FloatLiteralExpression) -> Self {
        Self::Float(float)
    }
}
