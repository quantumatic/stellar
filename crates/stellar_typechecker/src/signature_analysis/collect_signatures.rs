use stellar_ast_lowering::LoweredModule;
use stellar_database::{
    GenericParameterData, GenericParameterScopeData, GenericParameterScopeID, ModuleID,
    PredicateData, SignatureID, State,
};
use stellar_thir::{
    ty::{Type, TypeConstructor},
    Path, Predicate,
};

pub struct CollectSignatures<'s> {
    state: &'s mut State,
    module: ModuleID,
}

impl<'s> CollectSignatures<'s> {
    pub fn run_all(self, state: &'s mut State, lowered_modules: &[LoweredModule]) {
        lowered_modules.iter().for_each(|lowered_module| {
            CollectSignatures {
                state,
                module: lowered_module.module(),
            }
            .run(lowered_module.hir());
        })
    }

    fn run(mut self, module: &stellar_hir::Module) {
        module.items.iter().for_each(|item| match item {
            stellar_hir::ModuleItem::Enum(enum_) => {
                self.analyze_enum_type_signature(self.module, enum_);
            }
            stellar_hir::ModuleItem::TypeAlias(alias) => {
                self.analyze_type_alias(self.module, alias)
            }
            _ => todo!(),
        })
    }

    fn analyze_enum_type_signature(&mut self, module: ModuleID, enum_hir: &stellar_hir::Enum) {
        let signature = module
            .symbol_or_panic(self.state.db(), enum_hir.name.id)
            .to_enum_or_panic()
            .signature(self.state.db());

        self.analyze_generic_parameters(module, signature, None, &enum_hir.generic_parameters);
        self.analyze_where_predicates(module, signature, &enum_hir.where_predicates);

        signature.analyzed(self.state.db_mut());
    }

    fn analyze_generic_parameters(
        &mut self,
        module: ModuleID,
        signature: SignatureID,
        parent_generic_parameter_scope: Option<GenericParameterScopeID>,
        generic_parameters: &[stellar_hir::GenericParameter],
    ) {
        let generic_parameter_scope = GenericParameterScopeData::alloc(self.state.db_mut());

        generic_parameters.iter().for_each(|parameter_hir| {
            let default_value = if let Some(default_value) = &parameter_hir.default_value {
                self.resolve_type(default_value)
            } else {
                None
            };

            if let Some(bounds) = &parameter_hir.bounds {
                let bounds = self.resolve_bounds(bounds);

                let predicate = PredicateData::alloc(
                    self.state.db_mut(),
                    Type::Constructor(TypeConstructor::new_primitive(parameter_hir.name.id)),
                    bounds,
                );

                signature.add_predicate(self.state.db_mut(), predicate);
            }

            let generic_parameter = GenericParameterData::alloc(
                self.state.db_mut(),
                parameter_hir.name.location,
                default_value,
            );

            generic_parameter_scope.add_generic_parameter(
                self.state.db_mut(),
                parameter_hir.name.id,
                generic_parameter,
            );
        })
    }

    fn analyze_where_predicates(
        &mut self,
        module: ModuleID,
        signature: SignatureID,
        predicates_hir: &[stellar_hir::WherePredicate],
    ) {
        predicates_hir.iter().for_each(|predicate_hir| {
            let Some(ty) = self.resolve_type(&predicate_hir.ty) else {
                return;
            };

            let bounds = self.resolve_bounds(&predicate_hir.bounds);

            let predicate = PredicateData::alloc(self.state.db_mut(), ty, bounds);

            signature.add_predicate(self.state.db_mut(), predicate);
        })
    }

    fn resolve_bounds(&mut self, bounds: &[stellar_hir::TypeConstructor]) -> Vec<TypeConstructor> {
        todo!()
    }

    fn resolve_bound(&mut self, bound: &stellar_hir::TypeConstructor) -> TypeConstructor {
        todo!()
    }

    fn resolve_type(&mut self, ty: &stellar_hir::Type) -> Option<Type> {
        todo!()
    }

    fn analyze_type_signature(&mut self, path: &stellar_ast::Path) {}

    fn analyze_type_alias(&mut self, module: ModuleID, alias_hir: &stellar_hir::TypeAlias) {
        let alias = module
            .symbol_or_panic(self.state.db(), alias_hir.name.id)
            .to_type_alias_or_panic();
        let signature = alias.signature(self.state.db());

        self.analyze_generic_parameters(module, signature, None, &alias_hir.generic_parameters);

        let Some(value) = self.resolve_type(&alias_hir.value) else {
            return;
        };

        *alias.ty_mut(self.state.db_mut()) = value;

        signature.analyzed(self.state.db_mut());
    }
}
