use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct ImaginaryNumberLiteralExpression {
    literal: f64,
}

impl ImaginaryNumberLiteralExpression {
    #[inline]
    pub const fn new(literal: f64) -> Self {
        Self { literal }
    }

    #[inline]
    pub const fn literal(&self) -> f64 {
        self.literal
    }
}

impl From<ImaginaryNumberLiteralExpression> for RawExpression {
    fn from(imaginary_number: ImaginaryNumberLiteralExpression) -> Self {
        Self::ImaginaryNumber(imaginary_number)
    }
}
