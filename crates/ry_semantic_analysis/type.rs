pub type Path = Vec<Symbol>;

pub enum Type {
    Unit,
    Primary { path: Path },
    Tuple(Vec<Type>),
    Array(Box<Type>, Box<Type>),
}
