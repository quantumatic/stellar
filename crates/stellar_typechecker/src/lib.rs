use stellar_database::{Database, ModuleData};
use stellar_interner::{IdentifierID, PathID};

pub fn module_name(path: PathID) -> IdentifierID {
    IdentifierID::from(
        path.resolve_or_panic()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap(),
    )
}

pub fn typecheck_module(db: &mut Database, module: stellar_hir::Module) {
    let module_name = module_name(module.filepath);
    let _id = ModuleData::alloc(db, module_name, module.filepath, module.docstring);
}
