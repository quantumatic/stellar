#[cfg(feature = "debug")]
use std::time::Instant;

use stellar_ast_lowering::LoweredModule;
use stellar_database::{ModuleID, State};
use stellar_filesystem::location::Location;
#[cfg(feature = "debug")]
use tracing::trace;

use crate::{diagnostics::PackageImport, resolution::resolve::resolve_global_path};

pub struct ResolveImports<'s> {
    state: &'s mut State,
    module: ModuleID,
}

impl<'s> ResolveImports<'s> {
    pub fn run_all(state: &'s mut State, lowered_modules: &[LoweredModule]) {
        for lowered_module in lowered_modules {
            ResolveImports {
                state,
                module: lowered_module.module(),
            }
            .run(lowered_module.hir())
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

        let Some(symbol) = resolve_global_path(self.state, path) else {
            return;
        };

        if let Some(module) = symbol.to_module_or_none() {
            if self
                .state
                .db()
                .contains_package(module.name(self.state.db()))
            {
                self.state.diagnostics_mut().add_single_file_diagnostic(
                    path.path.location.filepath,
                    PackageImport::new(location, *path.path.identifiers.first().unwrap()),
                );
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
            "resolve_import(path = {:?}, module = {}) <{} us>",
            path,
            self.module,
            now.elapsed().as_millis()
        )
    }
}
