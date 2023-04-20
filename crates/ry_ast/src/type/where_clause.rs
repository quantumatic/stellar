use super::Type;
use serde::{Deserialize, Serialize};

pub type WhereClause = Vec<WhereClauseUnit>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct WhereClauseUnit {
    pub r#type: Type,
    pub constraint: Type,
}
