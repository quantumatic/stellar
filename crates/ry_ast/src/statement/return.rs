use super::Statement;
use crate::expression::Expression;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub return_value: Expression,
}

impl From<ReturnStatement> for Statement {
    fn from(r#return: ReturnStatement) -> Self {
        Self::Return(r#return)
    }
}
