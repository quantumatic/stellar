use ry_ast::Bounds;
use ry_fx_hash::FxHashMap;
use ry_interner::Symbol;
use ry_thir::ty::Type;

#[derive(Default)]
pub struct GenericParameterScope<'p> {
    parent_scope: Option<&'p GenericParameterScope<'p>>,
    parameters: FxHashMap<Symbol, GenericData>,
}

impl<'p> GenericParameterScope<'p> {
    #[inline]
    #[must_use]
    pub fn new(parent: Option<&'p GenericParameterScope<'p>>) -> Self {
        Self {
            parent_scope: parent,
            parameters: FxHashMap::default(),
        }
    }

    #[inline]
    pub fn add_generic_parameter(&mut self, parameter_name: Symbol, data: GenericData) {
        self.parameters.insert(parameter_name, data);
    }

    pub fn resolve(&self, parameter_name: Symbol) -> Option<&GenericData> {
        if let data @ Some(..) = self.parameters.get(&parameter_name) {
            data
        } else if let Some(parent_scope) = self.parent_scope {
            parent_scope.resolve(parameter_name)
        } else {
            None
        }
    }

    pub fn exists(&self, parameter_name: Symbol) -> bool {
        let exists_in_parent_scope = if let Some(parent_scope) = self.parent_scope {
            parent_scope.exists(parameter_name)
        } else {
            false
        };

        self.parameters.contains_key(&parameter_name) || exists_in_parent_scope
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericData {
    pub bounds: Bounds,
    pub default_value: Type,
}