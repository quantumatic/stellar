use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub struct StringLiteralExpression {
    literal: Arc<str>,
}

impl StringLiteralExpression {
    #[inline]
    pub fn literal(&self) -> Arc<str> {
        self.literal.clone()
    }
}
