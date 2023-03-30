use self::import::ImportItem;

pub mod docstring;
pub mod function;
pub mod import;

#[derive(Debug, PartialEq)]
pub enum Item {
    Import(ImportItem),
}
