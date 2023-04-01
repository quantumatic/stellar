use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct BoolLiteralExpression {
    literal: bool,
}

impl BoolLiteralExpression {
    #[inline]
    pub const fn new(literal: bool) -> Self {
        Self { literal }
    }

    #[inline]
    pub const fn literal(&self) -> bool {
        self.literal
    }
}

impl From<BoolLiteralExpression> for RawExpression {
    fn from(bool: BoolLiteralExpression) -> Self {
        Self::Bool(bool)
    }
}
