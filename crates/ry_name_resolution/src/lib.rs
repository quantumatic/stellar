pub mod resolution_tree_builder;

use ry_ast::Item;
use ry_interner::Symbol;

pub struct NameResolutionTree<'ast> {
    pub projects: Vec<ProjectNode<'ast>>,
}

pub struct Workspace {
    pub projects: Vec<Symbol>,
}

pub struct ProjectNode<'ast> {
    pub name: Symbol,
    pub children: Vec<ModuleNode<'ast>>,
}

pub struct ModuleNode<'ast> {
    pub name: Symbol,
    pub children: Vec<ModuleItemNode<'ast>>,
}

pub struct ModuleItemNode<'ast> {
    pub name: Symbol,
    pub ast: &'ast Item,
}
