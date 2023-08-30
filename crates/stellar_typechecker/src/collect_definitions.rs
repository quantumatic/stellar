use stellar_ast::IdentifierAST;
use stellar_database::{
    EnumData, EnumItemData, FunctionData, InterfaceData, ModuleID, State, StructData, Symbol,
};

use crate::diagnostics::{DuplicateEnumItem, DuplicateModuleItem};

pub struct CollectDefinitions<'s> {
    state: &'s State,
    module: ModuleID,
}

impl<'s> CollectDefinitions<'s> {
    pub fn run_all<'a>(
        state: &'s State,
        modules: impl IntoIterator<Item = &'a (ModuleID, stellar_hir::Module)>,
    ) {
        for module in modules {
            CollectDefinitions {
                state,
                module: module.0,
            }
            .run(&module.1);
        }
    }

    fn run(self, module: &stellar_hir::Module) {
        for item in &module.items {
            match item {
                stellar_hir::ModuleItem::Enum(enum_) => self.define_enum(enum_),
                stellar_hir::ModuleItem::Function(function) => self.define_function(function),
                stellar_hir::ModuleItem::Struct(struct_) => self.define_struct(struct_),
                stellar_hir::ModuleItem::Interface(interface) => self.define_interface(interface),
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

#[cfg(test)]
mod tests {
    use stellar_ast_lowering::LowerToHir;
    use stellar_interner::{IdentifierID, PathID};
    use stellar_parser::parse_module;

    use super::*;

    #[test]
    fn test_enum() {
        let state = State::new();
        let filepath_id = PathID::from("test.sr");
        let source_code = "enum A {}\nenum B {}";

        let hir = &LowerToHir::run_all(
            &state,
            vec![parse_module(filepath_id, source_code, state.diagnostics())],
        );

        CollectDefinitions::run_all(&state, hir);

        assert!(state
            .db_lock()
            .get_module_or_panic(hir[0].0)
            .get_symbol_or_panic(IdentifierID::from("A"))
            .is_enum());
        assert!(state
            .db_lock()
            .get_module_or_panic(hir[0].0)
            .get_symbol_or_panic(IdentifierID::from("B"))
            .is_enum());
        assert!(state.diagnostics_inner().is_ok());
    }

    #[test]
    fn test_duplicate_definition() {
        let state = State::new();
        let filepath_id = PathID::from("test.sr");
        let source_code = "enum A {}\nenum A {}";

        CollectDefinitions::run_all(
            &state,
            &LowerToHir::run_all(
                &state,
                vec![parse_module(filepath_id, source_code, state.diagnostics())],
            ),
        );

        assert_eq!(
            state.diagnostics_inner().file_diagnostics[0].code,
            Some("E005".to_owned())
        );
    }

    #[test]
    fn test_enum_items() {
        let state = State::new();
        let filepath_id = PathID::from("test.sr");
        let source_code = "enum A { A, B, C }";

        let hir = &LowerToHir::run_all(
            &state,
            vec![parse_module(filepath_id, source_code, state.diagnostics())],
        );

        CollectDefinitions::run_all(&state, hir);

        let db = state.db_lock();
        let items = &db
            .get_module_or_panic(hir[0].0)
            .get_symbol_or_panic(IdentifierID::from("A"))
            .get_enum_or_panic(&db)
            .items;

        assert!(items.contains_key(&IdentifierID::from("A")));
        assert!(items.contains_key(&IdentifierID::from("B")));
        assert!(items.contains_key(&IdentifierID::from("C")));
    }

    #[test]
    fn test_function() {
        let state = State::new();
        let filepath_id = PathID::from("test.sr");
        let source_code = "fun a() {}";

        let hir = &LowerToHir::run_all(
            &state,
            vec![parse_module(filepath_id, source_code, state.diagnostics())],
        );

        CollectDefinitions::run_all(&state, hir);

        assert!(state
            .db_lock()
            .get_module_or_panic(hir[0].0)
            .get_symbol_or_panic(IdentifierID::from("a"))
            .is_function());
        assert!(state.diagnostics_inner().is_ok());
    }

    #[test]
    fn test_struct() {
        let state = State::new();
        let filepath_id = PathID::from("test.sr");
        let source_code = "struct A {}";

        let hir = &LowerToHir::run_all(
            &state,
            vec![parse_module(filepath_id, source_code, state.diagnostics())],
        );

        CollectDefinitions::run_all(&state, hir);

        assert!(state
            .db_lock()
            .get_module_or_panic(hir[0].0)
            .get_symbol_or_panic(IdentifierID::from("A"))
            .is_struct());
        assert!(state.diagnostics_inner().is_ok());
    }

    #[test]
    fn test_interface() {
        let state = State::new();
        let filepath_id = PathID::from("test.sr");
        let source_code = "interface A {}";

        let hir = &LowerToHir::run_all(
            &state,
            vec![parse_module(filepath_id, source_code, state.diagnostics())],
        );

        CollectDefinitions::run_all(&state, hir);

        assert!(state
            .db_lock()
            .get_module_or_panic(hir[0].0)
            .get_symbol_or_panic(IdentifierID::from("A"))
            .is_interface());
        assert!(state.diagnostics_inner().is_ok());
    }
}
