use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct OptionType {
    inner: Type,
}

impl OptionType {
    #[inline]
    pub const fn inner(&self) -> &Type {
        &self.inner
    }
}

impl From<OptionType> for RawType {
    fn from(option: OptionType) -> Self {
        Self::Option(option)
    }
}
