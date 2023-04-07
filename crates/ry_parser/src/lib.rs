//! This crate provides a parser for Ry programming language
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
    clippy::option_if_let_else
)]

pub mod error;
pub(crate) mod expression;
pub(crate) mod item;
pub(crate) mod path;
pub(crate) mod statement;
pub(crate) mod r#type;

use error::*;
use item::ItemsParser;
use ry_ast::{
    declaration::Docstring,
    name::Name,
    span::At,
    token::{
        RawToken::{self, DocstringComment, Identifier},
        Token,
    },
    ProgramUnit,
};
use ry_interner::Interner;
use ry_lexer::Lexer;

#[macro_use]
mod macros;

/// Represents parser state.
#[derive(Debug)]
pub struct ParserState<'a> {
    lexer: Lexer<'a>,
    current: Token,
    next: Token,
}

pub(crate) trait Parser
where
    Self: Sized,
{
    type Output;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output>;
}

pub(crate) trait OptionalParser
where
    Self: Sized,
{
    type Output;

    fn optionally_parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output>;
}

impl<'a> ParserState<'a> {
    /// Creates initial parser state.
    ///
    /// # Usage
    /// ```
    /// use ry_parser::ParserState;
    /// use ry_interner::Interner;
    ///
    /// let mut interner = Interner::default();
    /// let parser = ParserState::new("pub fun test() {}", &mut interner);
    /// ```
    #[must_use]
    pub fn new(contents: &'a str, interner: &'a mut Interner) -> Self {
        let mut lexer = Lexer::new(contents, interner);

        let current = lexer.next_no_comments().unwrap();
        let next = current.clone();

        Self {
            lexer,
            current,
            next,
        }
    }

    /// Advances the parser to the next token and skips comment tokens.
    fn advance(&mut self) {
        self.current = self.next.clone();
        self.next = self.lexer.next_no_comments().unwrap();
    }

    fn expect<N>(&self, expected: RawToken, node: N) -> Result<(), ParseError>
    where
        N: Into<String>,
    {
        if self.next.inner == expected {
            Ok(())
        } else {
            Err(ParseError::unexpected_token(
                self.next.clone(),
                expected!(expected),
                node,
            ))
        }
    }

    fn consume<N>(&mut self, expected: RawToken, node: N) -> Result<(), ParseError>
    where
        N: Into<String>,
    {
        self.expect(expected, node)?;
        self.advance();
        Ok(())
    }

    fn consume_identifier<N>(&mut self, node: N) -> Result<Name, ParseError>
    where
        N: Into<String>,
    {
        let spanned_symbol;

        match self.next.inner {
            Identifier(symbol) => {
                spanned_symbol = symbol.at(self.next.span);
            }
            _ => {
                return Err(ParseError::unexpected_token(
                    self.next.clone(),
                    expected!("identifier"),
                    node,
                ));
            }
        }

        self.advance();

        Ok(spanned_symbol)
    }

    /// Consumes the docstrings for the module and the first item in the module, if present.
    pub(crate) fn consume_module_and_first_item_docstrings(
        &mut self,
    ) -> ParseResult<(Docstring, Docstring)> {
        let (mut module_docstring, mut local_docstring) = (vec![], vec![]);

        loop {
            if let DocstringComment { global, content } = &self.next.inner {
                if *global {
                    module_docstring.push(content.clone());
                } else {
                    local_docstring.push(content.clone());
                }
            } else {
                return Ok((module_docstring, local_docstring));
            }

            self.advance();
        }
    }

    /// Consumes the docstring for a local item (i.e., anything that is not the module docstring
    /// or the first item in the module (because it will be already consumed in
    /// [`Parser::consume_module_and_first_item_docstrings()`])).
    pub(crate) fn consume_docstring(&mut self) -> ParseResult<Docstring> {
        let mut result = vec![];

        loop {
            if let DocstringComment { global, content } = &self.next.inner {
                if !global {
                    result.push(content.clone());
                }
            } else {
                return Ok(result);
            }

            self.advance();
        }
    }

    /// Returns [`ParseResult<ProgramUnit>`] where [`ProgramUnit`] represents
    /// AST for a Ry module.
    /// ```
    /// use ry_parser::ParserState;
    /// use ry_interner::Interner;
    ///
    /// let mut interner = Interner::default();
    /// let mut parser = ParserState::new("fun test() {}", &mut interner);
    /// assert!(parser.parse().is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Will return [`Err`] on any parsing error.
    pub fn parse(&mut self) -> ParseResult<ProgramUnit> {
        let (global_docstring, first_docstring) =
            self.consume_module_and_first_item_docstrings()?;
        Ok(ProgramUnit {
            docstring: global_docstring,
            items: ItemsParser { first_docstring }.parse_with(self)?,
        })
    }
}
