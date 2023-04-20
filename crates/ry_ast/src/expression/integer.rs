use super::RawExpression;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegerLiteralExpression {
    pub literal: u64,
}

impl From<IntegerLiteralExpression> for RawExpression {
    fn from(integer: IntegerLiteralExpression) -> Self {
        Self::Integer(integer)
    }
}
