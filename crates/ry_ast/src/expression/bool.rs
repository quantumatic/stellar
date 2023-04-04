use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct BoolLiteralExpression {
    pub literal: bool,
}

impl From<BoolLiteralExpression> for RawExpression {
    fn from(bool: BoolLiteralExpression) -> Self {
        Self::Bool(bool)
    }
}
