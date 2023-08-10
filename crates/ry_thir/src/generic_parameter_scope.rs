use ry_filesystem::location::Location;
use ry_fx_hash::FxHashMap;
use ry_interner::IdentifierID;

use crate::ty::Type;

#[derive(Default, PartialEq, Clone, Debug)]
pub struct GenericParameterScope<'p> {
    parent_scope: Option<&'p GenericParameterScope<'p>>,
    parameters: FxHashMap<IdentifierID, GenericParameterData>,
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
    pub fn add_generic_parameter(
        &mut self,
        parameter_name: IdentifierID,
        data: GenericParameterData,
    ) {
        self.parameters.insert(parameter_name, data);
    }

    pub fn resolve(&self, parameter_name: IdentifierID) -> Option<&GenericParameterData> {
        if let Some(data) = self.parameters.get(&parameter_name) {
            Some(data)
        } else if let Some(parent_scope) = self.parent_scope {
            parent_scope.resolve(parameter_name)
        } else {
            None
        }
    }

    #[must_use]
    pub fn exists(&self, parameter_name: IdentifierID) -> bool {
        let exists_in_parent_scope = if let Some(parent_scope) = self.parent_scope {
            parent_scope.exists(parameter_name)
        } else {
            false
        };

        self.parameters.contains_key(&parameter_name) || exists_in_parent_scope
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParameterData {
    pub location: Location,
    pub default_value: Option<Type>,
}
