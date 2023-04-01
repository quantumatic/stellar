use crate::{expression::Expression, name::Name, r#type::Type, Mutability};

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct VarStatement {
    mutability: Mutability,
    name: Name,
    r#type: Option<Type>,
    value: Expression,
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

    #[inline]
    pub const fn mutability(&self) -> Mutability {
        self.mutability
    }

    #[inline]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    #[inline]
    pub const fn r#type(&self) -> Option<&Type> {
        self.r#type.as_ref()
    }

    #[inline]
    pub const fn value(&self) -> &Expression {
        &self.value
    }
}

impl From<VarStatement> for Statement {
    fn from(var: VarStatement) -> Self {
        Self::Var(var)
    }
}
