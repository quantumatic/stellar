use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct OptionType {
    pub inner: Box<Type>,
}

impl OptionType {
    #[inline]
    pub fn new(inner: Type) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl From<OptionType> for RawType {
    fn from(option: OptionType) -> Self {
        Self::Option(option)
    }
}
