use ry_interner::{Interner, Symbol};

pub trait Serialize {
    fn serialize(&self, buffer: &mut String, interner: &Interner);
}

impl Serialize for Symbol {
    fn serialize(&self, buffer: &mut String, interner: &Interner) {
        buffer.push_str(interner.resolve(*self).unwrap());
    }
}
