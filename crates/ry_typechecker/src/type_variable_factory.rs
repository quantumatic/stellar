use ry_filesystem::location::Location;
use ry_interner::IdentifierID;
use ry_name_resolution::Path;
use ry_thir::ty::{Type, TypeVariable};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TypeVariableFactory {
    last_type_variable_id: usize,
}

impl TypeVariableFactory {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn advance(&mut self) -> usize {
        self.last_type_variable_id += 1;
        self.last_type_variable_id
    }

    #[inline]
    #[must_use]
    pub fn make_variable_for_expression_type(&mut self, location: Location) -> TypeVariable {
        TypeVariable::Expression {
            location,
            id: self.advance(),
        }
    }

    #[inline]
    #[must_use]
    pub fn make_expression_type_placeholder(&mut self, location: Location) -> Type {
        Type::Variable(self.make_variable_for_expression_type(location))
    }

    #[inline]
    #[must_use]
    pub fn make_variable_for_unknown_type(&mut self, location: Location) -> TypeVariable {
        TypeVariable::InvalidType {
            location,
            id: self.advance(),
        }
    }

    #[inline]
    #[must_use]
    pub fn make_unknown_type_placeholder(&mut self, location: Location) -> Type {
        Type::Variable(self.make_variable_for_unknown_type(location))
    }

    #[inline]
    #[must_use]
    pub fn make_variable_for_type_argument(
        &mut self,
        symbol: IdentifierID,
        location: Option<Location>,
        origin_type_path: Path,
        origin_location: Location,
    ) -> TypeVariable {
        TypeVariable::TypeArgument {
            symbol,
            location,
            origin_type_path,
            origin_location,
            id: self.advance(),
        }
    }

    #[inline]
    #[must_use]
    pub fn make_type_argument_placeholder(
        &mut self,
        symbol: IdentifierID,
        location: Option<Location>,
        origin_type_path: Path,
        origin_location: Location,
    ) -> Type {
        Type::Variable(self.make_variable_for_type_argument(
            symbol,
            location,
            origin_type_path,
            origin_location,
        ))
    }
}
