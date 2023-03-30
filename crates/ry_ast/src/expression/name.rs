use string_interner::DefaultSymbol;

#[derive(Debug, PartialEq)]
pub struct NameExpression {
    name: DefaultSymbol,
}

impl NameExpression {
    #[inline]
    pub const fn name(&self) -> DefaultSymbol {
        self.name
    }
}
