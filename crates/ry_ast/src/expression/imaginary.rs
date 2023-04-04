use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct ImaginaryNumberLiteralExpression {
    pub literal: f64,
}

impl From<ImaginaryNumberLiteralExpression> for RawExpression {
    fn from(imaginary_number: ImaginaryNumberLiteralExpression) -> Self {
        Self::ImaginaryNumber(imaginary_number)
    }
}
