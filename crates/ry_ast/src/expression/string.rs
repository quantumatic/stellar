use super::RawExpression;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub struct StringLiteralExpression {
    pub literal: Arc<str>,
}

impl From<StringLiteralExpression> for RawExpression {
    fn from(string: StringLiteralExpression) -> Self {
        Self::StringLiteral(string)
    }
}
