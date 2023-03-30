use string_interner::DefaultSymbol;

use crate::{
    expression::Expression,
    r#type::Type,
    span::{Span, WithSpan},
};

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct VarStatement {
    mutable: Option<Span>,
    name: WithSpan<DefaultSymbol>,
    r#type: Option<Type>,
    value: Expression,
}

impl VarStatement {
    #[inline]
    pub const fn mutable(&self) -> Option<Span> {
        self.mutable
    }

    #[inline]
    pub const fn name(&self) -> &WithSpan<DefaultSymbol> {
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
