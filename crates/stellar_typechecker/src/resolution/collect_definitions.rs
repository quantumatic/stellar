use std::collections::BTreeMap;
#[cfg(feature = "debug")]
use std::time::Instant;

use stellar_ast::IdentifierAST;
use stellar_ast_lowering::LoweredModule;
use stellar_database::{
    EnumData, EnumItemData, FunctionData, InterfaceData, ModuleId, SignatureData, State,
    StructData, Symbol, TupleLikeStructData, TypeAliasData, TypeAliasId,
};
#[cfg(feature = "debug")]
use tracing::trace;

use crate::diagnostics::{EnumItemDefinedMultipleTimes, ItemDefinedMultipleTimes};

pub struct CollectDefinitions<'s> {
    state: &'s mut State,
    module: ModuleId,
    current_node_idx: usize,
}

impl<'s> CollectDefinitions<'s> {
    pub fn run_all(state: &'s mut State, modules: &BTreeMap<ModuleId, stellar_hir::Module>) {
        for module in modules {
            CollectDefinitions {
                state,
                module: *module.0,
                current_node_idx: 0,
            }
            .run(module.1);
        }
    }

    fn run(mut self, module: &stellar_hir::Module) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        for item in &module.items {
            match item {
                stellar_hir::ModuleItem::Enum(enum_) => self.define_enum(enum_),
                stellar_hir::ModuleItem::Function(function) => self.define_function(function),
                stellar_hir::ModuleItem::Struct(struct_) => self.define_struct(struct_),
                stellar_hir::ModuleItem::Interface(interface) => self.define_interface(interface),
                stellar_hir::ModuleItem::TupleLikeStruct(struct_) => {
                    self.define_tuple_like_struct(struct_)
                }
                stellar_hir::ModuleItem::TypeAlias(alias) => self.define_type_alias(alias),
                _ => {}
            }
        }

        #[cfg(feature = "debug")]
        trace!(
            "collect_definitions_in(module = '{}') <{} us>",
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        );
    }

    fn define_enum(&mut self, enum_: &stellar_hir::Enum) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let signature = SignatureData::alloc(
            self.state.db_mut(),
            enum_.visibility,
            enum_.name,
            self.current_node_idx,
            self.module,
        );
        let mut enum_data = EnumData::new(signature);

        for item in &enum_.items {
            let name = item.name();

            #[cfg(feature = "debug")]
            let now = Instant::now();

            self.check_for_duplicate_enum_item(&enum_data, name);

            enum_data.items.insert(
                name.id,
                EnumItemData::alloc(self.state.db_mut(), name, self.module),
            );

            #[cfg(feature = "debug")]
            trace!(
                "define_enum_item(enum_name = '{}', item_name = '{}', module = '{}') <{} us>",
                enum_.name.id,
                name.id,
                self.module.filepath(self.state.db()),
                now.elapsed().as_micros()
            );
        }

        self.check_for_duplicate_definition(enum_.name);

        let id = self.state.db_mut().add_enum(enum_data);

        self.module
            .add_module_item(self.state.db_mut(), enum_.name.id, Symbol::Enum(id));

        #[cfg(feature = "debug")]
        trace!(
            "define_enum(name = '{}', module = '{}') <{} us>",
            enum_.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        )
    }

    fn define_function(&mut self, function: &stellar_hir::Function) {
        let signature = SignatureData::alloc(
            self.state.db_mut(),
            function.signature.visibility,
            function.signature.name,
            self.current_node_idx,
            self.module,
        );

        let id = FunctionData::alloc(self.state.db_mut(), signature);

        self.check_for_duplicate_definition(function.signature.name);

        self.module.add_module_item(
            self.state.db_mut(),
            function.signature.name.id,
            Symbol::Function(id),
        );
    }

    fn define_struct(&mut self, struct_: &stellar_hir::Struct) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let signature = SignatureData::alloc(
            self.state.db_mut(),
            struct_.visibility,
            struct_.name,
            self.current_node_idx,
            self.module,
        );

        let id = StructData::alloc(self.state.db_mut(), signature);

        self.check_for_duplicate_definition(struct_.name);

        self.module
            .add_module_item(self.state.db_mut(), struct_.name.id, Symbol::Struct(id));

        #[cfg(feature = "debug")]
        trace!(
            "define_struct(name = '{}', module = '{}') <{} us>",
            struct_.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        )
    }

    fn define_tuple_like_struct(&mut self, struct_: &stellar_hir::TupleLikeStruct) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let signature = SignatureData::alloc(
            self.state.db_mut(),
            struct_.visibility,
            struct_.name,
            self.current_node_idx,
            self.module,
        );

        let id = TupleLikeStructData::alloc(self.state.db_mut(), signature);

        self.check_for_duplicate_definition(struct_.name);

        self.module.add_module_item(
            self.state.db_mut(),
            struct_.name.id,
            Symbol::TupleLikeStruct(id),
        );

        #[cfg(feature = "debug")]
        trace!(
            "define_tuple_like_struct(name = '{}', module = '{}') <{} us>",
            struct_.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        )
    }

    fn define_interface(&mut self, interface: &stellar_hir::Interface) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let signature = SignatureData::alloc(
            self.state.db_mut(),
            interface.visibility,
            interface.name,
            self.current_node_idx,
            self.module,
        );

        let id = InterfaceData::alloc(self.state.db_mut(), signature);

        self.check_for_duplicate_definition(interface.name);

        self.module.add_module_item(
            self.state.db_mut(),
            interface.name.id,
            Symbol::Interface(id),
        );

        #[cfg(feature = "debug")]
        trace!(
            "define_interface(name = '{}', module = '{}') <{} us>",
            interface.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        )
    }

    fn define_type_alias(&mut self, alias: &stellar_hir::TypeAlias) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let signature = SignatureData::alloc(
            self.state.db_mut(),
            alias.visibility,
            alias.name,
            self.current_node_idx,
            self.module,
        );

        let id = TypeAliasData::alloc(self.state.db_mut(), signature);

        self.check_for_duplicate_definition(alias.name);

        self.module
            .add_module_item(self.state.db_mut(), alias.name.id, Symbol::TypeAlias(id));

        #[cfg(feature = "debug")]
        trace!(
            "define_type_alias(name = '{}', module = '{}') <{} us>",
            alias.name.id,
            self.module.filepath(self.state.db()),
            now.elapsed().as_micros()
        );
    }

    fn check_for_duplicate_definition(&mut self, name: IdentifierAST) {
        if let Some(symbol) = self
            .module
            .module_item_symbol_or_none(self.state.db(), name.id)
        {
            let diagnostic = ItemDefinedMultipleTimes::new(
                name.id,
                symbol.name(self.state.db()).location,
                name.location,
            );

            self.state.diagnostics_mut().add_diagnostic(diagnostic);
        }
    }

    fn check_for_duplicate_enum_item(&mut self, enum_data: &EnumData, item_name: IdentifierAST) {
        if let Some(enum_item) = enum_data.items.get(&item_name.id) {
            let diagnostic = EnumItemDefinedMultipleTimes::new(
                self.state.db().signature(enum_data.signature).name.id,
                item_name.id,
                enum_item.name(self.state.db()).location,
                item_name.location,
            );

            self.state.diagnostics_mut().add_diagnostic(diagnostic);
        }
    }
}
