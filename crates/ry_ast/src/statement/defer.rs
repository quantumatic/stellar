use crate::expression::Expression;

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct DeferStatement {
    pub call: Expression,
}

impl From<DeferStatement> for Statement {
    fn from(defer: DeferStatement) -> Self {
        Self::Defer(defer)
    }
}
