use super::RawExpression;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FloatLiteralExpression {
    pub literal: f64,
}

impl From<FloatLiteralExpression> for RawExpression {
    fn from(float: FloatLiteralExpression) -> Self {
        Self::Float(float)
    }
}
