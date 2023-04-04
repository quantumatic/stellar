use super::{Expression, RawExpression};

#[derive(Debug, PartialEq)]
pub struct CallExpression {
    pub left: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl From<CallExpression> for RawExpression {
    fn from(call: CallExpression) -> Self {
        Self::Call(call)
    }
}
