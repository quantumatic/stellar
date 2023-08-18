//! Defines a [`GenericParameterScope`], to work with generic parameter scopes in
//! THIR.

use ry_filesystem::location::Location;
use ry_fx_hash::FxHashMap;
use ry_interner::IdentifierID;

use crate::ty::Type;

/// A generic parameter scope.
#[derive(Default, PartialEq, Clone, Debug)]
pub struct GenericParameterScope<'p> {
    /// A parent scope, for example:
    ///
    /// ```ry
    /// interface Foo[T] { // self.parent = Scope { parent: None, parameters: [T] }
    ///     fun bar[M]();  // self = Scope { parent: ..., parameters: [M] }
    /// }
    /// ```
    pub parent_scope: Option<&'p GenericParameterScope<'p>>,

    /// A map of generic parameters in the scope.
    pub parameters: FxHashMap<IdentifierID, GenericParameterData>,
}

impl<'p> GenericParameterScope<'p> {
    /// Creates a new empty generic parameter scope.
    #[inline]
    #[must_use]
    pub fn new(parent: Option<&'p GenericParameterScope<'p>>) -> Self {
        Self {
            parent_scope: parent,
            parameters: FxHashMap::default(),
        }
    }

    /// Adds a generic paramater into the scope.
    #[inline]
    pub fn add_generic_parameter(
        &mut self,
        parameter_name: IdentifierID,
        data: GenericParameterData,
    ) {
        self.parameters.insert(parameter_name, data);
    }

    /// Resolves a data about generic parameter in the scope.
    ///
    /// **Note**: the method shouldn't be used to check if the parameter exists
    /// in the scope. Use the [`contains()`] method.
    ///
    /// [`contains()`]: GenericParameterData::contains
    #[must_use]
    pub fn resolve(&self, parameter_name: IdentifierID) -> Option<&GenericParameterData> {
        if let Some(data) = self.parameters.get(&parameter_name) {
            Some(data)
        } else if let Some(parent_scope) = self.parent_scope {
            parent_scope.resolve(parameter_name)
        } else {
            None
        }
    }

    /// Checks if the generic parameter exists in the scope.
    #[must_use]
    pub fn contains(&self, parameter_name: IdentifierID) -> bool {
        self.parameters.contains_key(&parameter_name)
            || if let Some(parent_scope) = self.parent_scope {
                parent_scope.contains(parameter_name)
            } else {
                false
            }
    }
}

/// Data, that the Ry compiler has about a generic parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericParameterData {
    /// Location of the name of the generic parameter.
    ///
    /// ```txt
    /// foo[T: ToString = String]
    ///     ^
    /// ```
    pub location: Location,

    /// Default value of the generic parameter.
    ///
    /// ```txt
    /// foo[T: ToString = String]
    ///                   ^^^^^^
    /// ```
    pub default_value: Option<Type>,
}
