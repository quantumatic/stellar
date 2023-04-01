use std::sync::Arc;

use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct StringLiteralExpression {
    literal: Arc<str>,
}

impl StringLiteralExpression {
    #[inline]
    pub const fn new(literal: Arc<str>) -> Self {
        Self { literal }
    }

    #[inline]
    pub fn literal(&self) -> Arc<str> {
        self.literal.clone()
    }
}

impl From<StringLiteralExpression> for RawExpression {
    fn from(string: StringLiteralExpression) -> Self {
        Self::StringLiteral(string)
    }
}
