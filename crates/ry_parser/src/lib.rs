//! This crate provides a iter for Ry programming language
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
mod module;
mod path;
mod pattern;
mod statement;
mod r#type;

use ry_ast::{
    token::{LexError, RawToken, Token},
    Docstring, IdentifierAst,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic, CompilerDiagnostic};
use ry_interner::Interner;
use ry_lexer::Lexer;
use ry_source_file::{
    source_file::SourceFile,
    span::{Span, SpanIndex},
};

pub use module::parse_module;

#[macro_use]
mod macros;

/// Represents token state.
#[derive(Debug)]
pub struct ParseState<'a> {
    source_file: &'a SourceFile<'a>,
    file_id: usize,
    lexer: Lexer<'a>,
    current_token: Token,
    next_token: Token,
    diagnostics: Vec<CompilerDiagnostic>,
}

/// Represents AST node that can be parsed.
pub trait Parse
where
    Self: Sized,
{
    /// Output AST node type.
    type Output;

    /// Parse AST node of type [`Self::Output`].
    fn parse(self, state: &mut ParseState<'_>) -> Self::Output;
}

/// Represents AST node that can optionally be parsed. Optionally
/// in this context means that if some condition is satisfied,
/// the AST node is parsed as usually (`Parse::parse_with(...)`),
/// but if not, it is skipped, token state is not advanced and the
/// default value is returned.
///
/// A great example of this is the where clause, which is found optional
/// in the syntax definition of every item in the Ry programming language.
/// To avoid copying the behaviour described below, this trait must
/// be implemented.
pub trait OptionalParser
where
    Self: Sized,
{
    /// Output AST node type.
    type Output;

    /// Optionally parse AST node of type [`Self::Output`].
    ///
    /// For more information, see [`OptionalParser`].
    fn optionally_parse(self, state: &mut ParseState<'_>) -> Self::Output;
}

impl<'a> ParseState<'a> {
    /// Creates an initial state.
    ///
    /// Note: [`TokenIterator::current`] and [`TokenIterator::next`] are
    /// the same at an initial state.
    #[must_use]
    pub fn new(file_id: usize, source_file: &'a SourceFile<'a>) -> Self {
        let mut lexer = Lexer::new(file_id, source_file.source());

        let current = lexer.next_no_comments();
        let next = current;

        let mut state = Self {
            source_file,
            file_id,
            lexer,
            current_token: current,
            next_token: next,
            diagnostics: vec![],
        };
        state.check_next_token();

        state
    }

    /// Adds diagnostic if the next token has lex error in itself.
    fn check_next_token(&mut self) {
        if let RawToken::Error(error) = self.next_token.raw {
            self.diagnostics.push(
                ParseDiagnostic::LexError(LexError {
                    span: self.next_token.span,
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
        self.resolve_span(self.current_token.span)
    }

    /// Returns a lexer instance used for parsing.
    #[inline]
    #[must_use]
    pub const fn lexer(&self) -> &Lexer<'a> {
        &self.lexer
    }

    /// Returns an interner used for parsing.
    #[inline]
    #[must_use]
    pub const fn interner(&self) -> &Interner {
        self.lexer.interner()
    }

    /// Advances the iter to the next token (skips comment tokens).
    fn advance(&mut self) {
        self.current_token = self.next_token;
        self.next_token = self.lexer.next_no_comments();
        self.check_next_token();
    }

    /// Checks if the next token is [`expected`].
    fn expect<N>(&mut self, expected: RawToken, node: N) -> Option<()>
    where
        N: Into<String>,
    {
        if self.next_token.raw == expected {
            Some(())
        } else {
            self.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: self.next_token,
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
        self.advance();
        Some(())
    }

    fn consume_identifier<N>(&mut self, node: N) -> Option<IdentifierAst>
    where
        N: Into<String>,
    {
        let spanned_symbol = if self.next_token.raw == RawToken::Identifier {
            IdentifierAst {
                span: self.next_token.span,
                symbol: self.lexer.identifier(),
            }
        } else {
            self.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: self.next_token,
                    expected: expected!("identifier"),
                    node: node.into(),
                }
                .build(),
            );
            return None;
        };

        self.advance();

        Some(spanned_symbol)
    }

    /// Consumes the docstrings for the module and the first item in the module, if present.
    pub(crate) fn consume_module_and_first_item_docstrings(&mut self) -> (Docstring, Docstring) {
        let (mut module_docstring, mut local_docstring) = (vec![], vec![]);

        loop {
            match self.next_token.raw {
                RawToken::GlobalDocComment => {
                    module_docstring.push(
                        self.source_file
                            .source()
                            .index(self.next_token.span)
                            .to_owned(),
                    );
                }
                RawToken::LocalDocComment => {
                    local_docstring.push(
                        self.source_file
                            .source()
                            .index(self.next_token.span)
                            .to_owned(),
                    );
                }
                _ => return (module_docstring, local_docstring),
            }

            self.advance();
        }
    }

    /// Consumes the docstring for a local item (i.e., anything that is not the module docstring
    /// or the first item in the module (because it will be already consumed in
    /// [`Parser::consume_module_and_first_item_docstrings()`])).
    pub(crate) fn consume_docstring(&mut self) -> Docstring {
        let mut result = vec![];

        loop {
            if self.next_token.raw == RawToken::LocalDocComment {
                result.push(
                    self.source_file
                        .source()
                        .index(self.next_token.span)
                        .to_owned(),
                );
            } else {
                return result;
            }

            self.advance();
        }
    }
}
