use ry_ast::Type;

pub struct Resolver {
    next_unification_variable_index: usize,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            next_unification_variable_index: 2,
        }
    }

    fn new_unification_variable(&mut self) -> Type {
        let type_variable = Type::Variable(
            self.next_unification_variable_index
        );
        self.next_unification_variable_index += 3;

        type_variable
    }


}
