use crate::expression::Expression;

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct DeferStatement {
    pub call: Expression,
}

impl DeferStatement {
    #[inline]
    pub fn new(call: Expression) -> Self {
        Self { call }
    }
}

impl From<DeferStatement> for Statement {
    fn from(defer: DeferStatement) -> Self {
        Self::Defer(defer)
    }
}
