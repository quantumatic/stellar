pub mod build_resolution_tree;

use std::collections::HashMap;

use ry_interner::Symbol;

pub struct ResolutionTree {
    pub projects: HashMap<Symbol, ProjectNode>,
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
    pub items: HashMap<Symbol, ModuleItem>,
}

pub enum ModuleItem {
    NotAnalyzedItem(ry_ast::Item),
    AnalyzedItem(ry_typed_ast::Item),
    Module(ModuleNode),
}
