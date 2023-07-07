use ry_ast::Item;
use ry_interner::Symbol;

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
