use ry_ast::{Type, TypeConstructor};
use ry_interner::Symbol;

pub mod expression;
pub mod scope;
pub mod unification;

/// Struct that implements type inferece
pub struct InferenceContext {
    current_unification_variable_index: usize,
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceContext {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_unification_variable_index: 1,
        }
    }

    #[inline]
    #[must_use]
    pub fn new_unification_variable(&mut self) -> Type {
        self.current_unification_variable_index += 2;

        Type::Variable(self.current_unification_variable_index)
    }

    #[inline]
    #[must_use]
    fn primitive_type_constructor(&self, symbol: Symbol) -> Type {
        Type::Constructor(TypeConstructor {
            path: vec![symbol],
            generic_arguments: vec![],
        })
    }
}
