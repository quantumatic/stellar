use std::sync::Arc;

use stellar_name_resolution::{DefinitionID, Path};
use stellar_thir::GeneralTypeSignature;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SignatureAnalysisContext {
    /// List of type signatures. To resolve type signature cycles, like this one:
    ///
    /// ```stellar
    /// struct A { b: B }
    /// struct B { a: A }
    /// ```
    type_signature_stack: Vec<Arc<GeneralTypeSignature>>,

    /// List of type aliases, that have been recursivly analyzed. Used to find
    /// type alias cycles.
    type_alias_stack: Vec<DefinitionID>,
}

impl SignatureAnalysisContext {
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn add_type_signature_to_stack(&mut self, type_signature: Arc<GeneralTypeSignature>) {
        self.type_signature_stack.push(type_signature);
    }

    #[inline(always)]
    pub fn drop_type_signature_stack(&mut self) {
        self.type_signature_stack.clear();
    }

    #[inline(always)]
    pub fn add_type_alias_to_stack(&mut self, definition_id: DefinitionID) {
        self.type_alias_stack.push(definition_id);
    }

    #[inline(always)]
    pub fn drop_type_alias_stack(&mut self) {
        self.type_alias_stack.clear();
    }
}
