use super::Type;

pub type WhereClause = Vec<WhereClauseUnit>;

#[derive(Debug, PartialEq)]
pub struct WhereClauseUnit {
    r#type: Type,
    constraint: Type,
}

impl WhereClauseUnit {
    #[inline]
    pub const fn new(r#type: Type, constraint: Type) -> Self {
        Self { r#type, constraint }
    }

    #[inline]
    pub const fn r#type(&self) -> &Type {
        &self.r#type
    }

    #[inline]
    pub const fn constraint(&self) -> &Type {
        &self.constraint
    }
}
