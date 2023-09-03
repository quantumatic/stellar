use stellar_ast_lowering::LoweredModule;
use stellar_database::{
    GenericParameterData, GenericParameterScopeData, GenericParameterScopeID, ModuleID,
    SignatureID, State,
};
use stellar_thir::ty::Type;

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

    fn analyze_enum_type_signature(&mut self, module: ModuleID, enum_: &stellar_hir::Enum) {
        self.analyze_generic_parameters(
            module,
            module
                .module_item_symbol_or_panic(self.state.db(), enum_.name.id)
                .to_enum_or_panic()
                .signature(self.state.db()),
            None,
            &enum_.generic_parameters,
        );
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

    fn resolve_type(&self, ty: &stellar_hir::Type) -> Option<Type> {
        todo!()
    }

    fn analyze_type_alias(&self, module: ModuleID, alias: &stellar_hir::TypeAlias) {}
}
