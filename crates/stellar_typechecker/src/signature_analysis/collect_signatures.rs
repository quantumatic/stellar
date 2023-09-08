use std::mem;
#[cfg(feature = "debug")]
use std::time::Instant;

use stellar_ast_lowering::LoweredModule;
use stellar_database::{
    GenericParameterData, GenericParameterScopeData, GenericParameterScopeID, ModuleID,
    PredicateData, SignatureID, State, Symbol, TypeAliasID,
};
use stellar_interner::SymbolID;
use stellar_thir::{
    ty::{Type, TypeConstructor},
    Path, Predicate,
};
#[cfg(feature = "debug")]
use tracing::trace;

use crate::diagnostics::CycleDetectedWhenComputingSignatureOf;

pub struct CollectSignatures<'s> {
    state: &'s mut State,
    currently_analyzed_symbols: Vec<Symbol>,
    module: ModuleID,
}

impl<'s> CollectSignatures<'s> {
    pub fn run_all(state: &'s mut State, lowered_modules: &[LoweredModule]) {
        for lowered_module in lowered_modules {
            CollectSignatures {
                state,
                currently_analyzed_symbols: Vec::new(),
                module: lowered_module.module(),
            }
            .run(lowered_module.hir());
        }
    }

    fn run(mut self, module: &stellar_hir::Module) {
        for item in &module.items {
            self.analyze_signature(item);
        }
    }

    fn analyze_signature(&mut self, item: &stellar_hir::ModuleItem) {
        match item {
            stellar_hir::ModuleItem::Enum(enum_) => {
                self.analyze_signature_of_enum(self.module, enum_);
            }
            stellar_hir::ModuleItem::Struct(struct_) => {
                self.analyze_signature_of_struct(self.module, struct_)
            }
            stellar_hir::ModuleItem::TupleLikeStruct(struct_) => {
                self.analyze_signature_of_tuple_like_struct(self.module, struct_)
            }
            stellar_hir::ModuleItem::TypeAlias(alias) => {
                self.analyze_type_alias(self.module, alias)
            }
            _ => todo!(),
        }
    }

    fn emit_computation_cycle_diagnostic(&mut self) {
        let diagnostic = CycleDetectedWhenComputingSignatureOf::new(
            mem::take(&mut self.currently_analyzed_symbols)
                .into_iter()
                .map(|symbol| symbol.name(self.state.db()))
                .collect::<Vec<_>>(),
        );

        self.state.diagnostics_mut().add_file_diagnostic(diagnostic);
    }

    fn start_analyzing_signature_of(&mut self, symbol: Symbol) {
        if self.currently_analyzed_symbols.contains(&symbol) {
            #[cfg(feature = "debug")]
            trace!(
                "signature cycle detected when analyzing signature of '{}' in '{}'",
                symbol.name(self.state.db()).id,
                self.module.filepath(self.state.db())
            );

            self.emit_computation_cycle_diagnostic();

            return;
        }

        self.currently_analyzed_symbols.push(symbol);

        #[cfg(feature = "debug")]
        trace!(
            "start_analyzing_signature_of(name = '{}', module = '{}')",
            symbol.name(self.state.db()).id,
            self.module.filepath(self.state.db()),
        );
    }

    fn analyze_signature_of_enum(&mut self, module: ModuleID, enum_hir: &stellar_hir::Enum) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let symbol = module.symbol(self.state.db(), enum_hir.name.id);

        self.start_analyzing_signature_of(symbol);

        let enum_ = symbol.to_enum();
        let signature = enum_.signature(self.state.db());

        self.analyze_generic_parameters(module, signature, None, &enum_hir.generic_parameters);
        self.analyze_where_predicates(module, signature, &enum_hir.where_predicates);

        signature.set_analyzed(self.state.db_mut());

        #[cfg(feature = "debug")]
        trace!(
            "analyze_signature_of_enum(name = '{}', module = '{}') <{} us>",
            enum_hir.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        )
    }

    fn analyze_signature_of_struct(&mut self, module: ModuleID, struct_hir: &stellar_hir::Struct) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let symbol = module.symbol(self.state.db(), struct_hir.name.id);

        self.start_analyzing_signature_of(symbol);

        let struct_ = symbol.to_struct();
        let signature = struct_.signature(self.state.db());

        self.analyze_generic_parameters(module, signature, None, &struct_hir.generic_parameters);
        self.analyze_where_predicates(module, signature, &struct_hir.where_predicates);

        signature.set_analyzed(self.state.db_mut());

        #[cfg(feature = "debug")]
        trace!(
            "analyze_signature_of_struct(name = '{}', module = '{}') <{} us>",
            struct_hir.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        )
    }

    fn analyze_signature_of_tuple_like_struct(
        &mut self,
        module: ModuleID,
        struct_hir: &stellar_hir::TupleLikeStruct,
    ) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let symbol = module.symbol(self.state.db(), struct_hir.name.id);

        self.start_analyzing_signature_of(symbol);

        let struct_ = symbol.to_tuple_like_struct();
        let signature = struct_.signature(self.state.db());

        self.analyze_generic_parameters(module, signature, None, &struct_hir.generic_parameters);
        self.analyze_where_predicates(module, signature, &struct_hir.where_predicates);

        signature.set_analyzed(self.state.db_mut());

        #[cfg(feature = "debug")]
        trace!(
            "analyze_signature_of_tuple_like_struct(name = '{}', module = '{}') <{} us>",
            struct_hir.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        )
    }

    fn analyze_implemented_interfaces(
        &mut self,
        module: ModuleID,
        signature: SignatureID,
        interfaces_hir: &[stellar_hir::TypeConstructor],
    ) {
        for interface_hir in interfaces_hir {
            let interface = self.resolve_interface(interface_hir);

            signature.add_implemented_interface(self.state.db_mut(), interface);
        }
    }

    fn resolve_interface(
        &mut self,
        interface_hir: &stellar_hir::TypeConstructor,
    ) -> TypeConstructor {
        todo!()
    }

    fn analyze_generic_parameters(
        &mut self,
        module: ModuleID,
        signature: SignatureID,
        parent_generic_parameter_scope: Option<GenericParameterScopeID>,
        parameters_hir: &[stellar_hir::GenericParameter],
    ) {
        let generic_parameter_scope = GenericParameterScopeData::alloc(self.state.db_mut());

        for parameter_hir in parameters_hir {
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
        }

        signature.set_generic_parameter_scope(self.state.db_mut(), generic_parameter_scope);
    }

    fn analyze_where_predicates(
        &mut self,
        module: ModuleID,
        signature: SignatureID,
        predicates_hir: &[stellar_hir::WherePredicate],
    ) {
        for predicate_hir in predicates_hir {
            let Some(ty) = self.resolve_type(&predicate_hir.ty) else {
                return;
            };

            let bounds = self.resolve_bounds(&predicate_hir.bounds);

            let predicate = PredicateData::alloc(self.state.db_mut(), ty, bounds);

            signature.add_predicate(self.state.db_mut(), predicate);
        }
    }

    fn resolve_signature(&mut self, symbol: Symbol) -> SignatureID {
        let signature = symbol.signature(self.state.db());

        if !signature.is_analyzed(self.state.db()) {}

        todo!()
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
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let symbol = module.symbol(self.state.db(), alias_hir.name.id);

        self.start_analyzing_signature_of(symbol);

        let alias = symbol.to_type_alias();
        let signature = alias.signature(self.state.db());

        self.analyze_generic_parameters(module, signature, None, &alias_hir.generic_parameters);

        let Some(value) = self.resolve_type(&alias_hir.value) else {
            return;
        };

        *alias.ty_mut(self.state.db_mut()) = value;

        signature.set_analyzed(self.state.db_mut());

        #[cfg(feature = "debug")]
        trace!(
            "analyze_type_alias(name = '{}', module = '{}') <{} us>",
            alias_hir.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        )
    }
}
