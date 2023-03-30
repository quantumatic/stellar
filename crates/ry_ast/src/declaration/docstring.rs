use std::sync::Arc;

pub type Docstring = Vec<Arc<str>>;

#[derive(Debug, PartialEq)]
pub struct WithDocstring<T> {
    value: T,
    docstring: Docstring,
}

impl<T> WithDocstring<T> {
    #[inline]
    pub const fn unwrap(&self) -> &T {
        &self.value
    }

    #[inline]
    pub const fn docstring(&self) -> &Docstring {
        &self.docstring
    }
}

pub trait WithDocstringable {
    fn with_docstring(self, docstring: Vec<Arc<str>>) -> WithDocstring<Self>
    where
        Self: Sized,
    {
        WithDocstring {
            value: self,
            docstring,
        }
    }
}

impl<T: Sized> WithDocstringable for T {}
