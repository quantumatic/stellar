use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct CharLiteralExpression {
    pub literal: String,
}

impl From<CharLiteralExpression> for RawExpression {
    fn from(char: CharLiteralExpression) -> Self {
        Self::Char(char)
    }
}
