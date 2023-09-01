use std::sync::Arc;
#[cfg(feature = "debug")]
use std::time::Instant;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use stellar_ast::IdentifierAST;
use stellar_ast_lowering::LoweredModule;
use stellar_database::{
    EnumData, EnumItemData, FunctionData, InterfaceData, ModuleID, State, StructData, Symbol,
    TupleLikeStructData, TypeAliasData,
};
#[cfg(feature = "debug")]
use tracing::trace;

use crate::diagnostics::{EnumItemDefinedMultipleTimes, ItemDefinedMultipleTimes};

pub struct CollectDefinitions {
    state: Arc<State>,
    module: ModuleID,
}

impl CollectDefinitions {
    pub fn run_all(state: Arc<State>, modules: &[Arc<LoweredModule>]) {
        modules.par_iter().for_each(|module| {
            Self {
                state: state.clone(),
                module: module.module(),
            }
            .run(module.hir());
        });
    }

    fn run(self, module: &stellar_hir::Module) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        module.items.par_iter().for_each(|item| match item {
            stellar_hir::ModuleItem::Enum(enum_) => self.define_enum(enum_),
            stellar_hir::ModuleItem::Function(function) => self.define_function(function),
            stellar_hir::ModuleItem::Struct(struct_) => self.define_struct(struct_),
            stellar_hir::ModuleItem::Interface(interface) => self.define_interface(interface),
            stellar_hir::ModuleItem::TupleLikeStruct(struct_) => {
                self.define_tuple_like_struct(struct_)
            }
            stellar_hir::ModuleItem::TypeAlias(alias) => self.define_type_alias(alias),
            _ => {}
        });

        #[cfg(feature = "debug")]
        trace!(
            "collect_definitions_in(module = '{}') <{} us>",
            self.module.filepath(&self.state.db_read_lock()),
            now.elapsed().as_micros()
        );
    }

    fn define_enum(&self, enum_: &stellar_hir::Enum) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let mut enum_data = EnumData::new(enum_.visibility, enum_.name, self.module);

        for item in &enum_.items {
            let name = item.name();

            #[cfg(feature = "debug")]
            let now = Instant::now();

            self.check_for_duplicate_enum_item(&enum_data, name);

            enum_data.items.insert(
                name.id,
                EnumItemData::alloc(&mut self.state.db_write_lock(), name, self.module),
            );

            #[cfg(feature = "debug")]
            trace!(
                "define_enum_item(enum_name = '{}', item_name = '{}', module = '{}') <{} us>",
                enum_.name.id,
                name.id,
                self.module.filepath(&self.state.db_read_lock()),
                now.elapsed().as_micros()
            );
        }

        self.check_for_duplicate_definition(enum_.name);

        let id = self.state.db_write_lock().add_enum_module_item(enum_data);

        self.module.add_module_item(
            &mut self.state.db_write_lock(),
            enum_.name.id,
            Symbol::Enum(id),
        );

        #[cfg(feature = "debug")]
        trace!(
            "define_enum(name = '{}', module = '{}') <{} us>",
            enum_.name.id,
            self.module.filepath(&self.state.db_read_lock()),
            now.elapsed().as_micros()
        )
    }

    fn define_function(&self, function: &stellar_hir::Function) {
        let id = FunctionData::alloc(
            &mut self.state.db_write_lock(),
            function.signature.name,
            function.signature.visibility,
            self.module,
        );

        self.check_for_duplicate_definition(function.signature.name);

        self.module.add_module_item(
            &mut self.state.db_write_lock(),
            function.signature.name.id,
            Symbol::Function(id),
        );
    }

    fn define_struct(&self, struct_: &stellar_hir::Struct) {
        let id = StructData::alloc(
            &mut self.state.db_write_lock(),
            struct_.visibility,
            struct_.name,
            self.module,
        );

        self.check_for_duplicate_definition(struct_.name);

        self.module.add_module_item(
            &mut self.state.db_write_lock(),
            struct_.name.id,
            Symbol::Struct(id),
        )
    }

    fn define_tuple_like_struct(&self, struct_: &stellar_hir::TupleLikeStruct) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let id = TupleLikeStructData::alloc(
            &mut self.state.db_write_lock(),
            struct_.visibility,
            struct_.name,
            self.module,
        );

        self.check_for_duplicate_definition(struct_.name);

        self.module.add_module_item(
            &mut self.state.db_write_lock(),
            struct_.name.id,
            Symbol::TupleLikeStruct(id),
        );

        #[cfg(feature = "debug")]
        trace!(
            "define_tuple_like_struct(name = '{}', module = '{}') <{} us>",
            struct_.name.id,
            self.module.filepath(&self.state.db_read_lock()),
            now.elapsed().as_micros()
        )
    }

    fn define_interface(&self, interface: &stellar_hir::Interface) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let id = InterfaceData::alloc(
            &mut self.state.db_write_lock(),
            interface.visibility,
            interface.name,
            self.module,
        );

        self.check_for_duplicate_definition(interface.name);

        self.module.add_module_item(
            &mut self.state.db_write_lock(),
            interface.name.id,
            Symbol::Interface(id),
        );

        #[cfg(feature = "debug")]
        trace!(
            "define_interface(name = '{}', module = '{}') <{} us>",
            interface.name.id,
            self.module.filepath(&self.state.db_read_lock()),
            now.elapsed().as_micros()
        )
    }

    fn define_type_alias(&self, alias: &stellar_hir::TypeAlias) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let id = TypeAliasData::alloc(
            &mut self.state.db_write_lock(),
            alias.visibility,
            alias.name,
            self.module,
        );
        self.check_for_duplicate_definition(alias.name);

        self.module.add_module_item(
            &mut self.state.db_write_lock(),
            alias.name.id,
            Symbol::TypeAlias(id),
        );

        #[cfg(feature = "debug")]
        trace!(
            "define_type_alias(name = '{}', module = '{}') <{} us>",
            alias.name.id,
            self.module.filepath(&self.state.db_read_lock()),
            now.elapsed().as_micros()
        );
    }

    fn check_for_duplicate_definition(&self, name: IdentifierAST) {
        if let Some(symbol) = self
            .module
            .module_item_symbol(&self.state.db_read_lock(), name.id)
        {
            self.state.diagnostics().write().add_single_file_diagnostic(
                name.location.filepath,
                ItemDefinedMultipleTimes::new(
                    name.id.resolve_or_panic(),
                    symbol.name(&self.state.db_read_lock()).location,
                    name.location,
                ),
            );
        }
    }

    fn check_for_duplicate_enum_item(&self, enum_data: &EnumData, item_name: IdentifierAST) {
        if let Some(enum_item) = enum_data.items.get(&item_name.id) {
            self.state
                .diagnostics_write_lock()
                .add_single_file_diagnostic(
                    item_name.location.filepath,
                    EnumItemDefinedMultipleTimes::new(
                        enum_data.name.id.resolve_or_panic(),
                        item_name.id.resolve_or_panic(),
                        enum_item.name(&self.state.db_read_lock()).location,
                        item_name.location,
                    ),
                );
        }
    }
}
