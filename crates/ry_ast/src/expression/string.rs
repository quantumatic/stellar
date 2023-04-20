use super::RawExpression;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StringLiteralExpression {
    pub literal: String,
}

impl From<StringLiteralExpression> for RawExpression {
    fn from(string: StringLiteralExpression) -> Self {
        Self::StringLiteral(string)
    }
}
