use super::{Expression, RawExpression};

#[derive(Debug, PartialEq)]
pub struct CallExpression {
    pub left: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl CallExpression {
    #[inline]
    pub fn new(left: Expression, arguments: Vec<Expression>) -> Self {
        Self {
            left: Box::new(left),
            arguments,
        }
    }
}

impl From<CallExpression> for RawExpression {
    fn from(call: CallExpression) -> Self {
        Self::Call(call)
    }
}
