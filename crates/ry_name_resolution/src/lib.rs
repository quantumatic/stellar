use ry_ast::ModuleItem;
use ry_filesystem::span::Span;
use ry_fx_hash::FxHashMap;
use ry_interner::Symbol;
use ry_typed_ast::ty::Type;

pub struct ModuleData<'a> {
    pub project: Symbol,
    pub name: Symbol,
    pub parent: Option<&'a ModuleData<'a>>,
    pub bindings: FxHashMap<Symbol, NameBindingData<'a>>,
}

pub enum NameBindingData<'a> {
    Module(&'a ModuleData<'a>),
    Analyzed(AnalyzedNameBindingData),
    NotAnalyzed(&'a ModuleItem),
}

pub enum AnalyzedNameBindingData {
    Function(FunctionData),
}

pub struct FunctionData {
    pub span: Span,
    pub docstring: Option<String>,
    pub generic_parameters: Vec<Type>,
    pub constraints: ConstraintPair,
    pub parameters: Vec<(Symbol, Type)>,
    pub return_type: Type,
}

pub struct ConstraintPair {
    pub kind: ConstraintKind,
    pub left: Type,
    pub right: Type,
}

pub enum ConstraintKind {
    Satisfies,
    Eq,
}
