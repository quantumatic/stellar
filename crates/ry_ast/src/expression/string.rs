use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct StringLiteralExpression {
    pub literal: String,
}

impl From<StringLiteralExpression> for RawExpression {
    fn from(string: StringLiteralExpression) -> Self {
        Self::StringLiteral(string)
    }
}
