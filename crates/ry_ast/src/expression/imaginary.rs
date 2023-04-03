use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct ImaginaryNumberLiteralExpression {
    pub literal: f64,
}

impl ImaginaryNumberLiteralExpression {
    #[inline]
    pub const fn new(literal: f64) -> Self {
        Self { literal }
    }
}

impl From<ImaginaryNumberLiteralExpression> for RawExpression {
    fn from(imaginary_number: ImaginaryNumberLiteralExpression) -> Self {
        Self::ImaginaryNumber(imaginary_number)
    }
}
