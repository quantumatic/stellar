use super::Statement;
use crate::{expression::Expression, name::Name, r#type::Type};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VarStatement {
    pub name: Name,
    pub r#type: Option<Type>,
    pub value: Expression,
}

impl From<VarStatement> for Statement {
    fn from(var: VarStatement) -> Self {
        Self::Var(var)
    }
}
