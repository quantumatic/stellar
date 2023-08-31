use std::sync::Arc;

use rayon::{
    prelude::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSlice,
};
use stellar_ast::IdentifierAST;
use stellar_database::{
    EnumData, EnumItemData, FunctionData, InterfaceData, ModuleID, State, StructData, Symbol,
    TupleLikeStructData, TypeAliasData,
};

use crate::diagnostics::{DuplicateEnumItem, DuplicateModuleItem};

pub struct CollectDefinitions {
    state: Arc<State>,
    module: ModuleID,
}

impl CollectDefinitions {
    pub fn run_all(state: Arc<State>, modules: &[Arc<(ModuleID, stellar_hir::Module)>]) {
        modules
            .par_chunks(state.config().threads_amount)
            .for_each(|chunk| {
                chunk.par_iter().for_each(|module| {
                    CollectDefinitions {
                        state: state.clone(),
                        module: module.0,
                    }
                    .run(&module.1);
                })
            });
    }

    fn run(self, module: &stellar_hir::Module) {
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
        let mut enum_data = EnumData::new(enum_.visibility, enum_.name, self.module);

        for item in &enum_.items {
            let name = item.name();

            self.check_for_duplicate_enum_item(&enum_data, name);

            enum_data.items.insert(
                name.id,
                EnumItemData::alloc(&mut self.state.db_lock_write(), name, self.module),
            );
        }

        self.check_for_duplicate_definition(enum_.name);

        let id = self.state.db_lock_write().add_enum(enum_data);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module)
            .add_symbol(enum_.name.id, Symbol::Enum(id));
    }

    fn define_function(&self, function: &stellar_hir::Function) {
        let id = FunctionData::alloc(
            &mut self.state.db_lock_write(),
            function.signature.name,
            function.signature.visibility,
            self.module,
        );

        self.check_for_duplicate_definition(function.signature.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module)
            .add_symbol(function.signature.name.id, Symbol::Function(id));
    }

    fn define_struct(&self, struct_: &stellar_hir::Struct) {
        let id = StructData::alloc(
            &mut self.state.db_lock_write(),
            struct_.visibility,
            struct_.name,
            self.module,
        );

        self.check_for_duplicate_definition(struct_.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module)
            .add_symbol(struct_.name.id, Symbol::Struct(id))
    }

    fn define_tuple_like_struct(&self, struct_: &stellar_hir::TupleLikeStruct) {
        let id = TupleLikeStructData::alloc(
            &mut self.state.db_lock_write(),
            struct_.visibility,
            struct_.name,
            self.module,
        );

        self.check_for_duplicate_definition(struct_.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module)
            .add_symbol(struct_.name.id, Symbol::TupleLikeStruct(id))
    }

    fn define_interface(&self, interface: &stellar_hir::Interface) {
        let id = InterfaceData::alloc(
            &mut self.state.db_lock_write(),
            interface.visibility,
            interface.name,
            self.module,
        );

        self.check_for_duplicate_definition(interface.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module)
            .add_symbol(interface.name.id, Symbol::Interface(id));
    }

    fn define_type_alias(&self, alias: &stellar_hir::TypeAlias) {
        let id = TypeAliasData::alloc(
            &mut self.state.db_lock_write(),
            alias.visibility,
            alias.name,
            self.module,
        );
        self.check_for_duplicate_definition(alias.name);

        self.state
            .db_lock_write()
            .get_module_mut_or_panic(self.module)
            .add_symbol(alias.name.id, Symbol::TypeAlias(id));
    }

    fn check_for_duplicate_definition(&self, name: IdentifierAST) {
        if let Some(symbol) = self
            .state
            .db_lock()
            .get_module_or_panic(self.module)
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
