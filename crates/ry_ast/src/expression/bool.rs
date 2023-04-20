use super::RawExpression;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BoolLiteralExpression {
    pub literal: bool,
}

impl From<BoolLiteralExpression> for RawExpression {
    fn from(bool: BoolLiteralExpression) -> Self {
        Self::Bool(bool)
    }
}
