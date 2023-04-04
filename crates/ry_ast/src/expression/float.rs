use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct FloatLiteralExpression {
    pub literal: f64,
}

impl From<FloatLiteralExpression> for RawExpression {
    fn from(float: FloatLiteralExpression) -> Self {
        Self::Float(float)
    }
}
