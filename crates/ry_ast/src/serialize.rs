use std::{cell::RefCell, path::Path};

use ry_interner::{Interner, Symbol};
use ry_source_file::{
    source_file::SourceFile,
    source_file_manager::{FileID, SourceFileManager},
    span::{Span, DUMMY_SPAN},
};

/// A struct that allows to serialize a Ry module into a string, for debug purposes.
#[derive(Debug)]
pub struct Serializer<'a> {
    /// An interner used to resolve symbols in an AST.
    interner: &'a Interner,

    /// A source file being serialized.
    source_file: &'a SourceFile<'a>,

    /// An ID of the source file being serialized.
    source_file_id: usize,

    /// A source file manager.
    source_file_manager: &'a SourceFileManager<'a>,

    /// Current indentation level
    identation: usize,

    /// Symbols used for indentation.
    identation_symbols: &'a str,

    /// An output string produced,
    output: String,
}

impl<'a> Serializer<'a> {
    #[inline]
    #[must_use]
    pub fn new(
        interner: &'a Interner,
        source_file_id: usize,
        source_file_manager: &'a SourceFileManager<'a>,
    ) -> Option<Self> {
        Some(Self {
            interner,
            source_file: source_file_manager.get_file_by_id(source_file_id)?,
            source_file_id,
            source_file_manager,
            identation: 0,
            identation_symbols: "\t",
            output: String::new(),
        })
    }

    /// Sets the symbols used for indentation.
    #[inline]
    #[must_use]
    pub fn with_identation_symbols(mut self, identation_symbols: &'a str) -> Self {
        self.identation_symbols = identation_symbols;

        self
    }

    /// Returns the path of the source file being serialized as a string slice.
    #[inline]
    #[must_use]
    pub fn filepath_str(&self) -> &'a str {
        self.source_file.path_str()
    }

    /// Returns the path of the source file being serialized.
    #[inline]
    #[must_use]
    pub const fn filepath(&self) -> &'a Path {
        self.source_file.path()
    }

    /// Returns the source content of the file being serialized.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> &'a str {
        self.source_file.source()
    }

    /// Returns the length of the source content (in bytes).
    #[inline]
    #[must_use]
    pub const fn source_len(&self) -> usize {
        self.source_file.source().len()
    }

    /// Returns the ID of the source file being serialized.
    #[inline]
    #[must_use]
    pub const fn file_id(&self) -> FileID {
        self.source_file_id
    }

    /// Returns the interner used to resolve symbols in the AST of the module being serialized.
    #[inline]
    #[must_use]
    pub const fn interner(&self) -> &'a Interner {
        self.interner
    }

    /// Returns the source file manager.
    #[inline]
    #[must_use]
    pub fn file_manager(&self) -> &'a SourceFileManager<'a> {
        self.source_file_manager
    }

    /// Returns the current indentation level.
    #[inline]
    #[must_use]
    pub const fn identation(&self) -> usize {
        self.identation
    }

    /// Increments the current indentation level.
    #[inline]
    pub fn increment_indentation(&mut self) {
        self.identation += 1;
    }

    /// Decrements the current indentation level.
    #[inline]
    pub fn decrement_indentation(&mut self) {
        self.identation -= 1;
    }

    /// Returns the symbols used for indentation.
    #[inline]
    #[must_use]
    pub const fn identation_symbols(&self) -> &'a str {
        self.identation_symbols
    }

    /// Pushes a string into the output.
    pub fn push<S>(&mut self, str: S)
    where
        S: AsRef<str>,
    {
        self.output.push_str(str.as_ref());
    }

    /// Pushes a newline into the output.
    #[inline]
    pub fn push_newline(&mut self) {
        self.output.push('\n');
    }

    /// Adds indentation symbols into the output.
    pub fn write_identation(&mut self) {
        for _ in 0..self.identation() {
            self.push(self.identation_symbols());
        }
    }

    /// Returns the output string produced.
    #[inline]
    #[must_use]
    pub fn output(&self) -> &str {
        &self.output
    }
}

pub trait Serialize<'a> {
    fn serialize(&self, serializer: RefCell<Serializer<'a>>);
}

impl Serialize<'_> for Span {
    fn serialize(&self, serializer: RefCell<Serializer<'_>>) {
        match self {
            &DUMMY_SPAN => serializer.borrow_mut().push("DUMMY"),
            _ => serializer.borrow_mut().push(
                if self.start() >= self.end() || self.file_id() != serializer.borrow().file_id() {
                    "INVALID".to_owned()
                } else {
                    format!("{}:{}#{}", self.start(), self.end(), self.file_id())
                },
            ),
        }
    }
}

impl Serialize<'_> for Symbol {
    fn serialize(&self, serializer: RefCell<Serializer<'_>>) {
        serializer.borrow_mut().push(
            serializer
                .borrow()
                .interner()
                .resolve(*self)
                .unwrap_or_else(|| panic!("Symbol {self} cannot be resolved")),
        );
    }
}
