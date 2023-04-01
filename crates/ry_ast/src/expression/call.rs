use super::{Expression, RawExpression};

#[derive(Debug, PartialEq)]
pub struct CallExpression {
    left: Expression,
    arguments: Vec<Expression>,
}

impl CallExpression {
    #[inline]
    pub const fn new(left: Expression, arguments: Vec<Expression>) -> Self {
        Self { left, arguments }
    }

    #[inline]
    pub const fn left(&self) -> &Expression {
        &self.left
    }

    #[inline]
    pub const fn arguments(&self) -> &Vec<Expression> {
        &self.arguments
    }
}

impl From<CallExpression> for RawExpression {
    fn from(call: CallExpression) -> Self {
        Self::Call(call)
    }
}
