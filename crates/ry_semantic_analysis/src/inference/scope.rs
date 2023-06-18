use ry_ast::{Identifier, Type};
use ry_interner::Symbol;

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    variable: Identifier,
    r#type: Type,
}

impl Assignment {
    #[inline]
    #[must_use]
    pub const fn new(variable: Identifier, r#type: Type) -> Self {
        Self { variable, r#type }
    }

    #[inline]
    #[must_use]
    pub const fn variable(&self) -> &Identifier {
        &self.variable
    }

    #[inline]
    #[must_use]
    pub const fn r#type(&self) -> &Type {
        &self.r#type
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    type_context: Vec<Assignment>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    #[inline]
    #[must_use]
    pub const fn new(parent: Option<Box<Scope>>) -> Self {
        Self {
            type_context: Vec::new(),
            parent,
        }
    }

    #[inline]
    #[must_use]
    pub const fn new_global() -> Self {
        Self::new(None)
    }

    pub fn extend_type_context(&mut self, assignment: Assignment) {
        self.type_context.push(assignment);
    }

    pub fn lookup_type_context(&self, symbol: Symbol) -> Option<&Assignment> {
        if let Some(assignment) = self
            .type_context
            .iter()
            .find(|assignment| assignment.variable().unwrap() == &symbol)
        {
            Some(assignment)
        } else {
            self.parent
                .as_ref()
                .and_then(|parent| parent.lookup_type_context(symbol))
        }
    }
}
