use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct CharLiteralExpression {
    pub literal: char,
}

impl CharLiteralExpression {
    #[inline]
    pub const fn new(literal: char) -> Self {
        Self { literal }
    }
}

impl From<CharLiteralExpression> for RawExpression {
    fn from(char: CharLiteralExpression) -> Self {
        Self::Char(char)
    }
}
