use crate::serialize::Serialize;
use ry_interner::Interner;
use std::sync::Arc;

pub type Docstring = Vec<Arc<str>>;

impl Serialize for (bool, &Docstring) {
    fn serialize(&self, buffer: &mut String, _interner: &Interner) {
        let mut prefix = '/';

        if self.0 {
            prefix = '!';
        }

        for docstring in self.1 {
            buffer.push_str("//");
            buffer.push(prefix);
            buffer.push_str(docstring);
            buffer.push('\n');
        }
    }
}

#[derive(Debug, PartialEq)]
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

pub trait WithDocstring {
    fn with_docstring(self, docstring: Vec<Arc<str>>) -> Documented<Self>
    where
        Self: Sized,
    {
        Documented {
            value: self,
            docstring,
        }
    }
}

impl<T: Sized> WithDocstring for T {}
