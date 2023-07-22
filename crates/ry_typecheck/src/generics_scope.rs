use ry_fx_hash::FxHashMap;
use ry_interner::Symbol;
use ry_thir::ty::Type;

pub struct GenericParametersScope<'p> {
    parent: Option<&'p GenericParametersScope<'p>>,
    generics: FxHashMap<Symbol, GenericData>,
}

pub struct GenericData {
    default_value: Type,
}
