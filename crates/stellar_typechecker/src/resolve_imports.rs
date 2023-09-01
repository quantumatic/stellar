use std::sync::Arc;
#[cfg(feature = "debug")]
use std::time::Instant;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use stellar_ast_lowering::LoweredModule;
use stellar_database::{ModuleID, State};
use stellar_filesystem::location::Location;

use crate::resolve::resolve_global_path;

pub struct ResolveImports {
    state: Arc<State>,
    module: ModuleID,
}

impl ResolveImports {
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
        module.items.par_iter().for_each(|item| {
            if let stellar_hir::ModuleItem::Import { location, path } = item {
                self.resolve_import(*location, path)
            }
        })
    }

    fn resolve_import(&self, _: Location, path: &stellar_ast::ImportPath) {
        #[cfg(feature = "debug")]
        let now = Instant::now();

        let Some(symbol) = resolve_global_path(&self.state, path) else {
            return;
        };

        let name = if let Some(as_) = path.as_ {
            as_.id
        } else {
            symbol.name(&self.state.db_read_lock()).id
        };

        self.module
            .add_resolved_import(&mut self.state.db_write_lock(), name, symbol);

        #[cfg(feature = "debug")]
        println!(
            "resolve_import(path = {:?}, module = {}) <{} us>",
            path,
            self.module,
            now.elapsed().as_millis()
        )
    }
}
