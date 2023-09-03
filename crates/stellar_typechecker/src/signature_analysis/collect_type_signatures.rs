use stellar_ast_lowering::LoweredModule;
use stellar_database::{ModuleID, State};

pub struct CollectTypeSignatures<'s> {
    state: &'s mut State,
    module: ModuleID,
}

impl<'s> CollectTypeSignatures<'s> {
    pub fn run_all(self, state: &'s mut State, lowered_modules: &[LoweredModule]) {
        lowered_modules.iter().for_each(|lowered_module| {
            CollectTypeSignatures {
                state,
                module: lowered_module.module(),
            }
            .run(lowered_module.hir());
        })
    }

    fn run(self, module: &stellar_hir::Module) {
        module.items.iter().for_each(|item| match item {
            stellar_hir::ModuleItem::Enum(enum_) => {
                self.analyze_enum_type_signature(self.module, enum_)
            }
            stellar_hir::ModuleItem::TypeAlias(alias) => {
                self.analyze_type_alias(self.module, alias)
            }
            _ => todo!(),
        })
    }

    fn analyze_enum_type_signature(&self, module: ModuleID, enum_: &stellar_hir::Enum) {}

    fn analyze_type_alias(&self, module: ModuleID, alias: &stellar_hir::TypeAlias) {}
}
