use std::sync::Arc;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use stellar_ast::IdentifierAST;
use stellar_ast_lowering::LoweredModule;
use stellar_database::{
    EnumData, EnumItemData, FunctionData, InterfaceData, ModuleID, State, StructData, Symbol,
    TupleLikeStructData, TypeAliasData,
};
#[cfg(feature = "debug")]
use tracing::trace;

use crate::diagnostics::{DuplicateEnumItem, DuplicateModuleItem};

pub struct CollectDefinitions {
    state: Arc<State>,
    module_id: ModuleID,
}

impl CollectDefinitions {
    pub fn run_all(state: Arc<State>, modules: &[Arc<LoweredModule>]) {
        modules.par_iter().for_each(|module| {
            CollectDefinitions {
                state: state.clone(),
                module_id: module.module_id(),
            }
            .run(module.hir());
        });
    }

    fn run(self, module: &stellar_hir::Module) {
        #[cfg(feature = "debug")]
        trace!(
            "collect_definitions_in(module = {})",
            self.state
                .db_lock()
                .get_module_or_panic(self.module_id)
                .filepath
        );

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
    }

    fn define_enum(&self, enum_: &stellar_hir::Enum) {
        #[cfg(feature = "debug")]
        trace!(
            "define_enum(name = {}, module = {})",
            enum_.name.id,
            self.state
                .db_lock()
                .get_module_or_panic(self.module_id)
                .filepath
        );

        let mut enum_data = EnumData::new(enum_.visibility, enum_.name, self.module_id);

        for item in &enum_.items {
            let name = item.name();

            #[cfg(feature = "debug")]
            trace!(
                "define_enum_item(enum_name = {}, item_name = {}, module = {})",
                enum_.name.id,
                name.id,
                self.state
                    .db_lock()
                    .get_module_or_panic(self.module_id)
                    .filepath
            );

            self.check_for_duplicate_enum_item(&enum_data, name);

            enum_data.items.insert(
                name.id,
                EnumItemData::alloc(self.state.db(), name, self.module_id),
            );
        }

        self.check_for_duplicate_definition(enum_.name);

        let id = self.state.db_lock_write().add_enum(enum_data);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(enum_.name.id, Symbol::Enum(id));
    }

    fn define_function(&self, function: &stellar_hir::Function) {
        #[cfg(feature = "debug")]
        trace!(
            "define_function(name = {}, module = {})",
            function.signature.name.id,
            self.state
                .db_lock()
                .get_module_or_panic(self.module_id)
                .filepath
        );

        let id = FunctionData::alloc(
            self.state.db(),
            function.signature.name,
            function.signature.visibility,
            self.module_id,
        );

        self.check_for_duplicate_definition(function.signature.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(function.signature.name.id, Symbol::Function(id));
    }

    fn define_struct(&self, struct_: &stellar_hir::Struct) {
        #[cfg(feature = "debug")]
        trace!(
            "define_struct(name = {}, module = {})",
            struct_.name.id,
            self.state
                .db_lock()
                .get_module_or_panic(self.module_id)
                .filepath
        );

        let id = StructData::alloc(
            self.state.db(),
            struct_.visibility,
            struct_.name,
            self.module_id,
        );

        self.check_for_duplicate_definition(struct_.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(struct_.name.id, Symbol::Struct(id))
    }

    fn define_tuple_like_struct(&self, struct_: &stellar_hir::TupleLikeStruct) {
        #[cfg(feature = "debug")]
        trace!(
            "define_tuple_like_struct(name = {}, module = {})",
            struct_.name.id,
            self.state
                .db_lock()
                .get_module_or_panic(self.module_id)
                .filepath
        );

        let id = TupleLikeStructData::alloc(
            self.state.db(),
            struct_.visibility,
            struct_.name,
            self.module_id,
        );

        self.check_for_duplicate_definition(struct_.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(struct_.name.id, Symbol::TupleLikeStruct(id))
    }

    fn define_interface(&self, interface: &stellar_hir::Interface) {
        #[cfg(feature = "debug")]
        trace!(
            "define_interface(name = {}, module = {})",
            interface.name.id,
            self.state
                .db_lock()
                .get_module_or_panic(self.module_id)
                .filepath
        );

        let id = InterfaceData::alloc(
            self.state.db(),
            interface.visibility,
            interface.name,
            self.module_id,
        );

        self.check_for_duplicate_definition(interface.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(interface.name.id, Symbol::Interface(id));
    }

    fn define_type_alias(&self, alias: &stellar_hir::TypeAlias) {
        #[cfg(feature = "debug")]
        trace!(
            "define_type_alias(name = {}, module = {})",
            alias.name.id,
            self.state
                .db_lock()
                .get_module_or_panic(self.module_id)
                .filepath
        );

        let id = TypeAliasData::alloc(
            self.state.db(),
            alias.visibility,
            alias.name,
            self.module_id,
        );
        self.check_for_duplicate_definition(alias.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(alias.name.id, Symbol::TypeAlias(id));
    }

    fn check_for_duplicate_definition(&self, name: IdentifierAST) {
        if let Some(symbol) = self
            .state
            .db_lock()
            .get_module_or_panic(self.module_id)
            .get_symbol(name.id)
        {
            self.state.diagnostics().write().add_single_file_diagnostic(
                name.location.filepath_id,
                DuplicateModuleItem::new(
                    name.id.resolve_or_panic(),
                    symbol.name_location_or_panic(&self.state.db_lock()),
                    name.location,
                ),
            );
        }
    }

    fn check_for_duplicate_enum_item(&self, enum_data: &EnumData, item_name: IdentifierAST) {
        if let Some(id) = enum_data.items.get(&item_name.id) {
            self.state
                .diagnostics_lock_write()
                .add_single_file_diagnostic(
                    item_name.location.filepath_id,
                    DuplicateEnumItem::new(
                        enum_data.name.id.resolve_or_panic(),
                        item_name.id.resolve_or_panic(),
                        self.state
                            .db_lock()
                            .get_enum_item_or_panic(*id)
                            .name
                            .location,
                        item_name.location,
                    ),
                );
        }
    }
}
