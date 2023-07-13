use ry_fx_hash::FxHashMap;
use ry_interner::Symbol;

pub struct ModuleData<'a> {
    project: Symbol,
    name: Symbol,
    parent: Option<&'a ModuleData<'a>>,
    bindings: FxHashMap<Symbol, NameBindingData<'a>>,
}

pub enum NameBindingData<'m> {
    Module(&'m ModuleData<'m>),
    Analyzed(),
}
