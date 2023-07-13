use std::path::PathBuf;

use ry_ast::{IdentifierAst, Impl, ImportPath, ModuleItem};
use ry_filesystem::span::Span;
use ry_fx_hash::FxHashMap;
use ry_interner::Symbol;
use ry_typed_ast::{ty::Type, TypeBounds};

pub mod build_resolution_tree;
pub mod diagnostics;

#[derive(Debug, PartialEq, Clone)]
pub struct NameResolutionTree<'ast> {
    pub projects: FxHashMap<Symbol, ProjectData<'ast>>,
}

impl<'ast> NameResolutionTree<'ast> {
    pub fn new() -> Self {
        NameResolutionTree {
            projects: FxHashMap::default(),
        }
    }

    pub fn resolve_absolute_path(&self, path: Path) -> Option<NameBindingData<'_, 'ast>> {
        fn split_first_and_last<T>(a: &[T]) -> Option<(&T, &[T], &T)> {
            let (first, rest) = a.split_first()?;
            let (last, middle) = rest.split_last()?;

            Some((first, middle, last))
        }

        let (first, middle, last) = split_first_and_last(&path.symbols)?;

        let mut module = &self.projects.get(first)?.root;

        for symbol in middle {
            module = module.submodules.get(symbol)?;
        }

        if let Some(binding) = module.bindings.get(last) {
            return Some(NameBindingData::Item(&binding));
        } else if let Some(submodule) = module.submodules.get(last) {
            return Some(NameBindingData::Module(&submodule));
        } else {
            return None;
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProjectData<'ast> {
    pub path: PathBuf,
    pub root: ModuleData<'ast>,
    pub dependencies: Vec<Symbol>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleData<'ast> {
    pub path: PathBuf,
    pub docstring: Option<&'ast str>,
    pub bindings: FxHashMap<Symbol, ModuleItemNameBindingData<'ast>>,
    pub submodules: FxHashMap<Symbol, ModuleData<'ast>>,
    pub implementations: Vec<&'ast Impl>,
    pub imports: Vec<(Span, &'ast ImportPath)>,
}

impl<'ast> ModuleData<'ast> {
    pub fn resolve_path<'tree>(
        &'tree self,
        path: Path,
        tree: &'tree NameResolutionTree<'ast>,
    ) -> Option<NameBindingData<'tree, 'ast>> {
        if path.symbols.len() == 1 {
            self.bindings
                .get(&path.symbols[0])
                .map(NameBindingData::Item)
        } else {
            None
        }
        .or({
            let (first_path_symbol, rest) = path.symbols.split_first()?;

            let first_symbol_resolved = self.resolve_symbol(*first_path_symbol, tree)?;

            match first_symbol_resolved {
                NameBindingData::Module(module) => {
                    if rest.is_empty() {
                        Some(NameBindingData::Module(module))
                    } else {
                        module.resolve_path(
                            Path {
                                symbols: rest.to_vec(),
                            },
                            tree,
                        )
                    }
                }
                NameBindingData::Item(item) => {
                    if rest.is_empty() {
                        Some(NameBindingData::Item(item))
                    } else {
                        None
                    }
                }
            }
        })
    }

    fn resolve_symbol<'tree>(
        &self,
        symbol: Symbol,
        tree: &'tree NameResolutionTree<'ast>,
    ) -> Option<NameBindingData<'tree, 'ast>> {
        for (_, import) in &self.imports {
            match import.r#as {
                Some(IdentifierAst { symbol, .. }) => {
                    if symbol != symbol {
                        continue;
                    }

                    return tree.resolve_absolute_path(import.path.clone().into());
                }
                None => {
                    let import_last_symbol = import.path.identifiers.last()?.symbol;

                    if import_last_symbol != symbol {
                        continue;
                    }

                    return tree.resolve_absolute_path(import.path.clone().into());
                }
            }
        }

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NameBindingData<'tree, 'ast> {
    Module(&'tree ModuleData<'ast>),
    Item(&'tree ModuleItemNameBindingData<'ast>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ModuleItemNameBindingData<'ast> {
    Analyzed(AnalyzedNameBindingData<'ast>),
    NotAnalyzed(&'ast ModuleItem),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AnalyzedNameBindingData<'ast> {
    Alias(AliasModuleItemData<'ast>),
    Enum(EnumData<'ast>),
    Trait(TraitData<'ast>),
    Function(FunctionData<'ast>),
    Struct(StructData<'ast>),
    TupleLikeStruct(TupleLikeStructData<'ast>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AliasModuleItemData<'ast> {
    pub span: Span,
    pub docstring: Option<&'ast str>,
    pub generic_parameters: Vec<Type>,
    pub value: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TraitData<'ast> {
    pub span: Span,
    pub docstring: Option<&'ast str>,
    pub generic_parameters: Vec<Type>,
    pub constraints: Vec<ConstraintPair>,
    pub items: FxHashMap<Symbol, TraitItemData<'ast>>,
    pub implementations: TraitImplementationData<'ast>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TraitItemData<'ast> {
    Alias {
        span: Span,
        docstring: Option<&'ast str>,
        generic_parameters: Vec<Type>,
        constraints: Vec<ConstraintPair>,
    },
    Function(FunctionData<'ast>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct TraitImplementationData<'ast> {
    pub span: Span,
    pub docstring: Option<&'ast str>,
    pub generic_parameters: Vec<Type>,
    pub constraints: Vec<ConstraintPair>,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeImplementationData<'ast> {
    pub span: Span,
    pub docstring: Option<&'ast str>,
    pub generic_parameters: Vec<Type>,
    pub constraints: Vec<ConstraintPair>,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionData<'ast> {
    pub span: Span,
    pub docstring: Option<&'ast str>,
    pub generic_parameters: Vec<Type>,
    pub constraints: Vec<ConstraintPair>,
    pub parameters: Vec<(Symbol, Type)>,
    pub return_type: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructData<'ast> {
    pub span: Span,
    pub docstring: Option<&'ast str>,
    pub generic_parameters: Vec<Type>,
    pub constraints: Vec<ConstraintPair>,
    pub fields: FxHashMap<Symbol, Type>,
    pub implementations: TraitImplementationData<'ast>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TupleLikeStructData<'ast> {
    pub span: Span,
    pub docstring: Option<&'ast str>,
    pub generic_parameters: Vec<Type>,
    pub constraints: Vec<ConstraintPair>,
    pub fields: FxHashMap<Symbol, Type>,
    pub implementations: TraitImplementationData<'ast>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumData<'ast> {
    pub span: Span,
    pub docstring: Option<&'ast str>,
    pub generic_parameters: Vec<Type>,
    pub constraints: Vec<ConstraintPair>,
    pub items: FxHashMap<Symbol, EnumItemData>,
    pub implementations: TraitImplementationData<'ast>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnumItemData {
    Identifier {
        span: Span,
    },
    TupleLike {
        span: Span,
        ty: Vec<Type>,
    },
    Struct {
        span: Span,
        fields: FxHashMap<Symbol, Type>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstraintPair {
    Satisfies { left: Type, right: TypeBounds },
    Eq { left: Type, right: Type },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    pub symbols: Vec<Symbol>,
}

impl From<ry_ast::Path> for Path {
    fn from(value: ry_ast::Path) -> Self {
        Self {
            symbols: value.identifiers.iter().map(|i| i.symbol).collect(),
        }
    }
}
