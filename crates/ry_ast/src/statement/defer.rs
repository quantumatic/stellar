use super::Statement;
use crate::expression::Expression;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DeferStatement {
    pub call: Expression,
}

impl From<DeferStatement> for Statement {
    fn from(defer: DeferStatement) -> Self {
        Self::Defer(defer)
    }
}
