use serde::{Deserialize, Serialize};

pub type Docstring = Vec<String>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Documented<T> {
    value: T,
    docstring: Docstring,
}

impl<T> Documented<T> {
    #[inline]
    pub const fn unwrap(&self) -> &T {
        &self.value
    }

    #[inline]
    pub const fn docstring(&self) -> &Docstring {
        &self.docstring
    }
}

pub trait WithDocComment {
    fn with_doc_comment(self, docstring: Vec<String>) -> Documented<Self>
    where
        Self: Sized,
    {
        Documented {
            value: self,
            docstring,
        }
    }
}

impl<T: Sized> WithDocComment for T {}
