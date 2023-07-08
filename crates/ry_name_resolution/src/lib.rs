pub mod resolution_tree_builder;

use std::collections::HashMap;

use ry_ast::Item;
use ry_interner::Symbol;

pub struct NameResolutionTree {
    pub projects: Vec<ProjectNode>,
}

pub type Workspace = HashMap<Symbol, ProjectNode>;

pub type ProjectNode = HashMap<Symbol, ModuleNode>;

pub type ModuleNode = HashMap<Symbol, ModuleItem>;

pub enum ModuleItem {
    NotAnalyzed(Item),
}
