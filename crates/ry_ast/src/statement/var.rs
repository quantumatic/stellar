use crate::{expression::Expression, name::Name, r#type::Type, Mutability};

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct VarStatement {
    pub mutability: Mutability,
    pub name: Name,
    pub r#type: Option<Type>,
    pub value: Expression,
}

impl VarStatement {
    #[inline]
    pub const fn new(
        mutability: Mutability,
        name: Name,
        r#type: Option<Type>,
        value: Expression,
    ) -> Self {
        Self {
            mutability,
            name,
            r#type,
            value,
        }
    }
}

impl From<VarStatement> for Statement {
    fn from(var: VarStatement) -> Self {
        Self::Var(var)
    }
}
