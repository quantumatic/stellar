use std::sync::Arc;

use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct StringLiteralExpression {
    pub literal: Arc<str>,
}

impl StringLiteralExpression {
    #[inline]
    pub fn new(literal: Arc<str>) -> Self {
        Self { literal }
    }
}

impl From<StringLiteralExpression> for RawExpression {
    fn from(string: StringLiteralExpression) -> Self {
        Self::StringLiteral(string)
    }
}
