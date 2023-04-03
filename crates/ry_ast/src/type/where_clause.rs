use super::Type;

pub type WhereClause = Vec<WhereClauseUnit>;

#[derive(Debug, PartialEq)]
pub struct WhereClauseUnit {
    pub r#type: Type,
    pub constraint: Type,
}

impl WhereClauseUnit {
    #[inline]
    pub const fn new(r#type: Type, constraint: Type) -> Self {
        Self { r#type, constraint }
    }
}
