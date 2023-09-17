use itertools::Itertools;
use stellar_ast::IdentifierAST;
use stellar_database::{ModuleId, PredicateId, SignatureId, Symbol};
use stellar_thir::{
    ty::{Type, TypeConstructor},
    Path,
};

use super::collect_signatures::CollectSignatures;
use crate::{
    diagnostics::UnderscoreTypeInSignature,
    resolution::resolve::resolve_global_path_in_module_context,
};

impl<'s, 'h> CollectSignatures<'s, 'h> {
    pub(crate) fn resolve_or_analyze_type_in_signature(
        &mut self,
        module: ModuleId,
        item_name: IdentifierAST,
        ty: &stellar_hir::Type,
    ) -> Option<Type> {
        match ty {
            stellar_hir::Type::Constructor(constructor) => self
                .resolve_or_analyze_type_constructor(module, item_name, constructor)
                .map(|c| Type::Constructor(c)),
            stellar_hir::Type::Tuple {
                location,
                element_types,
            } => {
                if element_types.is_empty() {
                    Some(Type::Unit)
                } else {
                    element_types
                        .iter()
                        .map(|ty| self.resolve_or_analyze_type_in_signature(module, item_name, ty))
                        .collect::<Option<_>>()
                        .map(|element_types| Type::Tuple { element_types })
                }
            }
            stellar_hir::Type::Function {
                location,
                parameter_types,
                return_type,
            } => parameter_types
                .iter()
                .map(|ty| self.resolve_or_analyze_type_in_signature(module, item_name, ty))
                .collect::<Option<_>>()
                .and_then(|parameter_types| {
                    Some(Type::Function {
                        parameter_types,
                        return_type: return_type
                            .as_ref()
                            .map(|ty| {
                                self.resolve_or_analyze_type_in_signature(
                                    module,
                                    item_name,
                                    ty.as_ref(),
                                )
                            })
                            .unwrap_or(Some(Type::Unit))
                            .map(|ty| Box::new(ty))?,
                    })
                }),
            stellar_hir::Type::InterfaceObject { location, bounds } => {
                let bounds = bounds
                    .iter()
                    .filter_map(|bound| self.resolve_or_analyze_interface(module, bound))
                    .collect::<Vec<_>>();

                if bounds.is_empty() {
                    None
                } else {
                    Some(Type::InterfaceObject { bounds })
                }
            }
            stellar_hir::Type::Underscore { location } => {
                self.state
                    .diagnostics_mut()
                    .add_diagnostic(UnderscoreTypeInSignature::new(item_name, *location));
                None
            }
        }
    }

    fn resolve_or_analyze_type_constructor(
        &mut self,
        module: ModuleId,
        item_name: IdentifierAST,
        constructor: &stellar_hir::TypeConstructor,
    ) -> Option<TypeConstructor> {
        let signature = self.resolve_or_analyze_signature(module, &constructor.path)?;

        let constructor = TypeConstructor {
            path: Path::from(&constructor.path),
            arguments: constructor
                .arguments
                .iter()
                .map(|ty| self.resolve_or_analyze_type_in_signature(module, item_name, ty))
                .collect::<Option<_>>()?,
        };

        self.validate_type_constructor(signature, &constructor);

        Some(constructor)
    }

    fn validate_type_constructor(&mut self, signature: SignatureId, constructor: &TypeConstructor) {
        // for &predicate in signature.predicates(self.state.db()) {
        // self.validate_predicate(predicate);
        // }
    }

    fn satisfies(&mut self, ty: &Type, interface: &TypeConstructor) -> bool {
        todo!()
    }

    fn validate_predicate(&mut self, predicate: PredicateId) -> Option<()> {
        todo!()
    }

    fn resolve_or_analyze_interface(
        &mut self,
        module: ModuleId,
        constructor: &stellar_hir::TypeConstructor,
    ) -> Option<TypeConstructor> {
        todo!()
    }

    fn resolve_or_analyze_signature(
        &mut self,
        module: ModuleId,
        path: &stellar_hir::Path,
    ) -> Option<SignatureId> {
        let symbol = resolve_global_path_in_module_context(self.state, path, module)?;

        let signature = symbol.signature(self.state.db());

        if !signature.is_analyzed(self.state.db()) {
            let module_hir = &self.modules[&symbol.module(self.state.db())];
            let node_idx = symbol.signature(self.state.db()).node_idx(self.state.db());

            let item_hir = &module_hir.items[node_idx];

            self.analyze_signature(symbol.module(self.state.db()), item_hir);
        }

        Some(signature)
    }
}
