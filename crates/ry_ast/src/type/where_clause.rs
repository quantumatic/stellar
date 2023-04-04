use super::Type;

pub type WhereClause = Vec<WhereClauseUnit>;

#[derive(Debug, PartialEq)]
pub struct WhereClauseUnit {
    pub r#type: Type,
    pub constraint: Type,
}
