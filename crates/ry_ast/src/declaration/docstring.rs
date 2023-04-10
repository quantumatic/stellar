use crate::serialize::Serialize;
use ry_interner::Interner;

pub type Docstring = Vec<String>;

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
