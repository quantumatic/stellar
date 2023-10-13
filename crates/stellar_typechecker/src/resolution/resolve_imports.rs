#[cfg(feature = "debug")]
use std::time::Instant;

use stellar_ast_lowering::LoweredModule;
use stellar_database::{ModuleId, State};
use stellar_filesystem::location::Location;
use stellar_fx_hash::FxHashMap;
#[cfg(feature = "debug")]
use tracing::trace;

use super::resolve_global_path;
use crate::diagnostics::PackageImport;

pub struct ResolveImports<'s> {
    state: &'s mut State,
    module: ModuleId,
}

impl<'s> ResolveImports<'s> {
    pub fn run_all(state: &'s mut State, modules: &FxHashMap<ModuleId, stellar_hir::Module>) {
        for module in modules {
            ResolveImports {
                state,
                module: *module.0,
            }
            .run(module.1)
        }
    }

    fn run(mut self, module: &stellar_hir::Module) {
        for item in &module.items {
            if let stellar_hir::ModuleItem::Import { location, path } = item {
                self.resolve_import(*location, path)
            }
        }
    }

    fn resolve_import(&mut self, location: Location, path: &stellar_ast::ImportPath) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let Some(symbol) = resolve_global_path(self.state, self.module.0, path) else {
            return;
        };

        if let Some(module) = symbol.to_module_or_none() {
            if self
                .module
                .0
                .dependencies(self.state.db())
                .contains_key(&module.name(self.state.db()))
                || module == self.module.0.root_module(self.state.db())
            {
                self.state
                    .diagnostics_mut()
                    .add_diagnostic(PackageImport::new(
                        location,
                        *path.path.identifiers.first().unwrap(),
                    ));
                return;
            }
        }

        let name = if let Some(as_) = path.as_ {
            as_.id
        } else {
            symbol.name(self.state.db()).id
        };

        self.module
            .add_resolved_import(self.state.db_mut(), name, symbol);

        #[cfg(feature = "debug")]
        trace!(
            "resolve_import(path = {:?}, module = {:?}) <{} us>",
            path,
            self.module,
            now.elapsed().as_millis()
        )
    }
}
