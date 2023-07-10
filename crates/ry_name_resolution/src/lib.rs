pub mod build_resolution_tree;

use std::collections::HashMap;

use ry_interner::{symbols, Symbol};
use ry_typed_ast::Path;

pub trait Resolve {
    type Node;

    fn resolve(&self, symbol: Symbol) -> Option<&Self::Node>;
}

pub struct ResolutionTree {
    pub projects: HashMap<Symbol, ProjectNode>,
}

impl Resolve for ResolutionTree {
    type Node = ProjectNode;

    fn resolve(&self, symbol: Symbol) -> Option<&Self::Node> {
        self.projects.get(&symbol)
    }
}

impl Default for ResolutionTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolutionTree {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }
}

pub struct ProjectNode {
    pub modules: HashMap<Symbol, ModuleNode>,
}

impl Resolve for ProjectNode {
    type Node = ModuleNode;

    fn resolve(&self, symbol: Symbol) -> Option<&Self::Node> {
        self.modules.get(&symbol)
    }
}

impl Default for ProjectNode {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectNode {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }
}

pub struct ModuleNode {
    pub docstring: Option<String>,
    pub imports: Vec<ImportPath>,
    pub items: HashMap<Symbol, ModuleItem>,
}

impl Resolve for ModuleNode {
    type Node = ModuleItem;

    fn resolve(&self, symbol: Symbol) -> Option<&Self::Node> {
        match symbol {
            symbols::UNDERSCORE => None,
            _ => self.items.get(&symbol),
        }
    }
}

pub struct ImportPath {
    pub left: Path,
    pub r#as: Symbol,
}

pub enum ModuleItem {
    NotAnalyzedItem(ry_ast::Item),
    AnalyzedItem(ry_typed_ast::Item),
    Module(ModuleNode),
}
