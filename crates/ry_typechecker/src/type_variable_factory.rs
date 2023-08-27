use ry_filesystem::location::Location;
use ry_interner::IdentifierID;
use ry_name_resolution::Path;
use ry_thir::ty::{Type, TypeVariable, TypeVariableID};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TypeVariableFactory {
    last_type_variable_id: usize,
}

impl TypeVariableFactory {
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    fn advance(&mut self) -> TypeVariableID {
        self.last_type_variable_id += 1;
        TypeVariableID(self.last_type_variable_id)
    }

    #[inline(always)]
    #[must_use]
    pub fn make_variable_for_expression_type(&mut self, location: Location) -> TypeVariable {
        TypeVariable::Expression {
            location,
            id: self.advance(),
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn make_expression_type_placeholder(&mut self, location: Location) -> Type {
        Type::Variable(self.make_variable_for_expression_type(location))
    }

    #[inline(always)]
    #[must_use]
    pub fn make_variable_for_type_placeholder(
        &mut self,
        location: Option<Location>,
        origin_type_path: Path,
        origin_location: Location,
    ) -> TypeVariable {
        TypeVariable::TypePlaceholder {
            location,
            origin_type_path,
            origin_location,
            id: self.advance(),
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn make_type_placeholder(
        &mut self,
        location: Option<Location>,
        origin_type_path: Path,
        origin_location: Location,
    ) -> Type {
        Type::Variable(self.make_variable_for_type_placeholder(
            location,
            origin_type_path,
            origin_location,
        ))
    }
}
