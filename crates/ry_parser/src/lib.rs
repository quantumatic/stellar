//! This crate provides a cursor for Ry programming language
//!
//! It uses the lexer from the ry_lexer crate to tokenize the input source
//! code and produces an Abstract Syntax Tree (AST) that represents the parsed code.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::unnested_or_patterns
)]

mod expression;
mod items;
mod literal;
mod path;
mod pattern;
mod statement;
mod r#type;

use codespan_reporting::diagnostic::Diagnostic;
use items::ItemsParser;
use ry_ast::{
    token::{LexError, RawToken, Token},
    Docstring, IdentifierAst, Module,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
use ry_interner::Interner;
use ry_lexer::Lexer;
use ry_source_file::source_file::SourceFile;
use ry_source_file::span::{Span, SpanIndex};

#[macro_use]
mod macros;

/// Represents token iterator.
#[derive(Debug)]
pub struct Cursor<'a> {
    source_file: &'a SourceFile<'a>,
    file_id: usize,
    lexer: Lexer<'a>,
    current: Token,
    next: Token,
    diagnostics: &'a mut Vec<Diagnostic<usize>>,
}

pub(crate) trait Parse
where
    Self: Sized,
{
    /// Output AST node type.
    type Output;

    /// Parse AST node of type [`Self::Output`].
    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output;
}

pub(crate) trait OptionalParser
where
    Self: Sized,
{
    /// Output AST node type.
    type Output;

    /// Optionally parse AST node of type [`Self::Output`].
    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output;
}

impl<'a> Cursor<'a> {
    /// Creates an initial cursor.
    ///
    /// # Usage
    /// ```
    /// use std::path::Path;
    /// use ry_parser::Cursor;
    /// use ry_interner::Interner;
    /// use ry_source_file::source_file::SourceFile;
    ///
    /// let mut diagnostics = vec![];
    /// let mut interner = Interner::default();
    /// let source_file = SourceFile::new(
    ///     Path::new("test.ry"),
    ///     "pub fun test() {}",
    /// );
    ///
    /// let cursor = Cursor::new(
    ///     0,
    ///     &source_file,
    ///     &mut interner,
    ///     &mut diagnostics
    /// );
    /// ```
    #[must_use]
    pub fn new(
        file_id: usize,
        source_file: &'a SourceFile<'a>,
        interner: &'a mut Interner,
        diagnostics: &'a mut Vec<Diagnostic<usize>>,
    ) -> Self {
        let mut lexer = Lexer::new(file_id, source_file.source(), interner);

        let current = lexer.next_no_comments();
        let next = current;

        let mut lexer = Self {
            source_file,
            file_id,
            lexer,
            current,
            next,
            diagnostics,
        };
        lexer.check_next_token();

        lexer
    }

    /// Adds diagnostic if the next token has lex error in itself.
    fn check_next_token(&mut self) {
        if let RawToken::Error(error) = self.next.raw {
            self.diagnostics.push(
                ParseDiagnostic::LexError(LexError {
                    span: self.next.span,
                    raw: error,
                })
                .build(),
            );
        }
    }

    /// Returns string slice corresponding to the given location.
    #[inline]
    #[must_use]
    fn resolve_span(&self, span: Span) -> &str {
        self.source_file.resolve_span(span)
    }

    /// Returns string slice corresponding to the current token's location.
    #[inline]
    #[must_use]
    fn resolve_current(&self) -> &str {
        self.resolve_span(self.current.span)
    }

    /// Returns diagnostics emitted during parsing.
    #[must_use]
    pub fn diagnostics(&self) -> &Vec<Diagnostic<usize>> {
        self.diagnostics
    }

    /// Advances the cursor to the next token (skips comment tokens).
    fn next_token(&mut self) {
        self.current = self.next;
        self.next = self.lexer.next_no_comments();
        self.check_next_token();
    }

    /// Checks if the next token is [`expected`].
    fn expect<N>(&mut self, expected: RawToken, node: N) -> Option<()>
    where
        N: Into<String>,
    {
        if self.next.raw == expected {
            Some(())
        } else {
            self.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: self.next,
                    expected: expected!(expected),
                    node: node.into(),
                }
                .build(),
            );

            None
        }
    }

    fn consume<N>(&mut self, expected: RawToken, node: N) -> Option<()>
    where
        N: Into<String>,
    {
        self.expect(expected, node)?;
        self.next_token();
        Some(())
    }

    fn consume_identifier<N>(&mut self, node: N) -> Option<IdentifierAst>
    where
        N: Into<String>,
    {
        let spanned_symbol = if self.next.raw == RawToken::Identifier {
            IdentifierAst {
                span: self.next.span,
                symbol: self.lexer.identifier(),
            }
        } else {
            self.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: self.next,
                    expected: expected!("identifier"),
                    node: node.into(),
                }
                .build(),
            );
            return None;
        };

        self.next_token();

        Some(spanned_symbol)
    }

    /// Consumes the docstrings for the module and the first item in the module, if present.
    pub(crate) fn consume_module_and_first_item_docstrings(&mut self) -> (Docstring, Docstring) {
        let (mut module_docstring, mut local_docstring) = (vec![], vec![]);

        loop {
            match self.next.raw {
                RawToken::GlobalDocComment => {
                    module_docstring
                        .push(self.source_file.source().index(self.next.span).to_owned());
                }
                RawToken::LocalDocComment => {
                    local_docstring
                        .push(self.source_file.source().index(self.next.span).to_owned());
                }
                _ => return (module_docstring, local_docstring),
            }

            self.next_token();
        }
    }

    /// Consumes the docstring for a local item (i.e., anything that is not the module docstring
    /// or the first item in the module (because it will be already consumed in
    /// [`Parser::consume_module_and_first_item_docstrings()`])).
    pub(crate) fn consume_docstring(&mut self) -> Docstring {
        let mut result = vec![];

        loop {
            if self.next.raw == RawToken::LocalDocComment {
                result.push(self.source_file.source().index(self.next.span).to_owned());
            } else {
                return result;
            }

            self.next_token();
        }
    }

    /// Returns [`ParseResult<ProgramUnit>`] where [`ProgramUnit`] represents
    /// AST for a Ry module.
    /// ```
    /// use std::path::Path;
    /// use ry_parser::Cursor;
    /// use ry_interner::Interner;
    /// use ry_source_file::source_file::SourceFile;
    ///
    /// let mut diagnostics = vec![];
    /// let mut interner = Interner::default();
    ///
    /// let source_file = SourceFile::new(
    ///     Path::new("test.ry"),
    ///     "fun test() {}",
    /// );
    ///
    /// let mut cursor = Cursor::new(
    ///     0,
    ///     &source_file,
    ///     &mut interner,
    ///     &mut diagnostics
    /// );
    /// let ast = cursor.parse();
    ///
    /// assert_eq!(ast.items.len(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// Will return [`Err`] on any parsing error.
    pub fn parse(&mut self) -> Module<'a> {
        let (global_docstring, first_docstring) = self.consume_module_and_first_item_docstrings();

        Module {
            filepath: self.source_file.path(),
            docstring: global_docstring,
            items: ItemsParser { first_docstring }.parse_with(self),
        }
    }
}
