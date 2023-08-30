use stellar_database::{
    EnumData, FunctionData, InterfaceData, ModuleID, State, StructData, Symbol,
};

use crate::diagnostics::DuplicateModuleItem;

pub struct CollectDefinitions<'s> {
    state: &'s State,
    module_id: ModuleID,
}

impl<'s> CollectDefinitions<'s> {
    pub fn run_all<'a>(
        state: &'s State,
        modules: impl IntoIterator<Item = &'a (ModuleID, stellar_hir::Module)>,
    ) {
        for module in modules {
            CollectDefinitions {
                state,
                module_id: module.0,
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
        let id = EnumData::alloc(
            &mut self.state.db().write(),
            enum_.visibility,
            enum_.name,
            self.module_id,
        );

        if let Some(symbol) = self
            .state
            .db()
            .read()
            .get_module_or_panic(self.module_id)
            .get_symbol(enum_.name.id)
        {
            self.state.diagnostics().write().add_single_file_diagnostic(
                enum_.name.location.filepath_id,
                DuplicateModuleItem::new(
                    enum_.name.id.resolve_or_panic(),
                    symbol.name_location_or_panic(&self.state.db().read()),
                    enum_.name.location,
                ),
            );
        }

        self.state
            .db()
            .write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(enum_.name.id, Symbol::Enum(id));
    }

    fn define_function(&self, function: &stellar_hir::Function) {
        let id = FunctionData::alloc(
            &mut self.state.db().write(),
            function.signature.name,
            function.signature.visibility,
            self.module_id,
        );

        if let Some(symbol) = self
            .state
            .db()
            .read()
            .get_module_or_panic(self.module_id)
            .get_symbol(function.signature.name.id)
        {
            self.state.diagnostics().write().add_single_file_diagnostic(
                function.signature.name.location.filepath_id,
                DuplicateModuleItem::new(
                    function.signature.name.id.resolve_or_panic(),
                    symbol.name_location_or_panic(&self.state.db().read()),
                    function.signature.name.location,
                ),
            );
        }

        self.state
            .db()
            .write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(function.signature.name.id, Symbol::Function(id));
    }

    fn define_struct(&self, struct_: &stellar_hir::Struct) {
        let id = StructData::alloc(
            &mut self.state.db().write(),
            struct_.visibility,
            struct_.name,
            self.module_id,
        );

        if let Some(symbol) = self
            .state
            .db()
            .read()
            .get_module_or_panic(self.module_id)
            .get_symbol(struct_.name.id)
        {
            self.state.diagnostics().write().add_single_file_diagnostic(
                struct_.name.location.filepath_id,
                DuplicateModuleItem::new(
                    struct_.name.id.resolve_or_panic(),
                    symbol.name_location_or_panic(&self.state.db().read()),
                    struct_.name.location,
                ),
            );
        }

        self.state
            .db()
            .write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(struct_.name.id, Symbol::Struct(id))
    }

    fn define_interface(&self, interface: &stellar_hir::Interface) {
        let id = InterfaceData::alloc(
            &mut self.state.db().write(),
            interface.visibility,
            interface.name,
            self.module_id,
        );

        if let Some(symbol) = self
            .state
            .db()
            .read()
            .get_module_or_panic(self.module_id)
            .get_symbol(interface.name.id)
        {
            self.state.diagnostics().write().add_single_file_diagnostic(
                interface.name.location.filepath_id,
                DuplicateModuleItem::new(
                    interface.name.id.resolve_or_panic(),
                    symbol.name_location_or_panic(&self.state.db().read()),
                    interface.name.location,
                ),
            )
        }

        self.state
            .db()
            .write()
            .get_module_mut_or_panic(self.module_id)
            .add_symbol(interface.name.id, Symbol::Interface(id));
    }
}

#[cfg(test)]
mod tests {
    use stellar_ast_lowering::LowerExt;
    use stellar_database::ModuleData;
    use stellar_interner::{IdentifierID, PathID};
    use stellar_parser::parse_module;

    use super::*;

    #[test]
    fn test_enum() {
        let state = State::new();
        let filepath_id = PathID::from("test.stellar");

        let module_id = ModuleData::alloc(
            &mut state.db().write(),
            IdentifierID::from("test"),
            filepath_id,
        );

        let source_code = "enum A {}\nenum B {}";
        let hir =
            parse_module(filepath_id, source_code, state.diagnostics()).lower(state.diagnostics());

        CollectDefinitions::run_all(&state, &[(module_id, hir)]);

        assert!(state
            .db()
            .read()
            .get_module_or_panic(module_id)
            .get_symbol_or_panic(IdentifierID::from("A"))
            .is_enum());
        assert!(state
            .db()
            .read()
            .get_module_or_panic(module_id)
            .get_symbol_or_panic(IdentifierID::from("B"))
            .is_enum());
        assert!(state.diagnostics().read().is_ok());
    }

    #[test]
    fn test_duplicate_definition() {
        let state = State::new();
        let filepath_id = PathID::from("test.stellar");

        let module_id = ModuleData::alloc(
            &mut state.db().write(),
            IdentifierID::from("test"),
            filepath_id,
        );

        let source_code = "enum A {}\nenum A {}";
        let hir =
            parse_module(filepath_id, source_code, state.diagnostics()).lower(state.diagnostics());

        CollectDefinitions::run_all(&state, &[(module_id, hir)]);

        assert_eq!(
            state.diagnostics().read().file_diagnostics[0].code,
            Some("E005".to_owned())
        );
    }

    #[test]
    fn test_function() {
        let state = State::new();
        let filepath_id = PathID::from("test.stellar");

        let module_id = ModuleData::alloc(
            &mut state.db().write(),
            IdentifierID::from("test"),
            filepath_id,
        );

        let source_code = "fun a() {}";
        let hir =
            parse_module(filepath_id, source_code, state.diagnostics()).lower(state.diagnostics());

        CollectDefinitions::run_all(&state, &[(module_id, hir)]);

        assert!(state
            .db()
            .read()
            .get_module_or_panic(module_id)
            .get_symbol_or_panic(IdentifierID::from("a"))
            .is_function());
        assert!(state.diagnostics().read().is_ok());
    }

    #[test]
    fn test_struct() {
        let state = State::new();
        let filepath_id = PathID::from("test.stellar");

        let module_id = ModuleData::alloc(
            &mut state.db().write(),
            IdentifierID::from("test"),
            filepath_id,
        );

        let source_code = "struct A {}";
        let hir =
            parse_module(filepath_id, source_code, state.diagnostics()).lower(state.diagnostics());

        CollectDefinitions::run_all(&state, &[(module_id, hir)]);

        assert!(state
            .db()
            .read()
            .get_module_or_panic(module_id)
            .get_symbol_or_panic(IdentifierID::from("A"))
            .is_struct());
        assert!(state.diagnostics().read().is_ok());
    }

    #[test]
    fn test_interface() {
        let state = State::new();
        let filepath_id = PathID::from("test.stellar");

        let module_id = ModuleData::alloc(
            &mut state.db().write(),
            IdentifierID::from("test"),
            filepath_id,
        );

        let source_code = "interface A {}";
        let hir =
            parse_module(filepath_id, source_code, state.diagnostics()).lower(state.diagnostics());

        CollectDefinitions::run_all(&state, &[(module_id, hir)]);

        assert!(state
            .db()
            .read()
            .get_module_or_panic(module_id)
            .get_symbol_or_panic(IdentifierID::from("A"))
            .is_interface());
        assert!(state.diagnostics().read().is_ok());
    }
}
