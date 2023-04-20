use super::{Expression, RawExpression};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CallExpression {
    pub left: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl From<CallExpression> for RawExpression {
    fn from(call: CallExpression) -> Self {
        Self::Call(call)
    }
}
