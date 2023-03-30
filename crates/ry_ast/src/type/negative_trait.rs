use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct NegativeTraitType {
    r#trait: Type,
}

impl NegativeTraitType {
    #[inline]
    pub const fn r#trait(&self) -> &Type {
        &self.r#trait
    }
}

impl From<NegativeTraitType> for RawType {
    fn from(negative_trait: NegativeTraitType) -> Self {
        Self::NegativeTrait(negative_trait)
    }
}
