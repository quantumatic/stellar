use super::RawExpression;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CharLiteralExpression {
    pub literal: String,
}

impl From<CharLiteralExpression> for RawExpression {
    fn from(char: CharLiteralExpression) -> Self {
        Self::Char(char)
    }
}
