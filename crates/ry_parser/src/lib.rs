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

mod r#enum;
mod expression;
mod function_decl;
mod r#impl;
mod imports;
mod statement;
mod struct_decl;
mod trait_decl;
mod r#type;

use error::*;
use ry_ast::{
    declaration::{Docstring, WithDocstring},
    span::At,
    token::{Keyword::*, RawToken::*, Token},
    *,
};
use ry_interner::Interner;
use ry_lexer::Lexer;

#[macro_use]
mod macros;

/// Represents parser state.
#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    next: Token,
}

impl<'a> Parser<'a> {
    /// Creates initial parser state.
    ///
    /// # Usage
    /// ```
    /// use ry_parser::Parser;
    /// use ry_interner::Interner;
    ///
    /// let mut interner = Interner::default();
    /// let parser = Parser::new("pub fun test() {}", &mut interner);
    /// ```
    #[must_use]
    pub fn new(contents: &'a str, interner: &'a mut Interner) -> Self {
        let mut lexer = Lexer::new(contents, interner);

        let current = lexer.next().unwrap();
        let next = current.clone();

        Self {
            lexer,
            current,
            next,
        }
    }

    /// Checks if the current token being parsed is invalid, and returns
    /// an error if so.
    fn check_scanning_error_for_current_token(&mut self) -> ParseResult<()> {
        if let Error(e) = self.current.unwrap() {
            Err(ParseError::lexer((*e).at(self.current.span())))
        } else {
            Ok(())
        }
    }

    /// Advances the parser to the next token and skips comment tokens.
    fn advance(&mut self) -> ParseResult<()> {
        self.current = self.next.clone();
        self.next = self.lexer.next_no_docstrings_and_comments().unwrap();

        Ok(())
    }

    /// Advances the parser to the next token and doesn't skip comment tokens.
    fn advance_with_docstring(&mut self) -> ParseResult<()> {
        self.current = self.next.clone();
        self.next = self.lexer.next_no_comments().unwrap();

        Ok(())
    }

    /// Consumes the docstrings for the module and the first item in the module, if present.
    pub(crate) fn consume_module_and_first_item_docstrings(
        &mut self,
    ) -> ParseResult<(Docstring, Docstring)> {
        let (mut module_docstring, mut local_docstring) = (vec![], vec![]);

        loop {
            if let DocstringComment { global, content } = self.next.unwrap() {
                if *global {
                    module_docstring.push(content.clone());
                } else {
                    local_docstring.push(content.clone());
                }
            } else {
                return Ok((module_docstring, local_docstring));
            }

            self.advance_with_docstring()?;
        }
    }

    /// Consumes the docstring for a local item (i.e., anything that is not the module docstring
    /// or the first item in the module (because it will be already consumed in
    /// [`Parser::consume_module_and_first_item_docstrings()`])).
    pub(crate) fn consume_non_module_docstring(&mut self) -> ParseResult<Docstring> {
        let mut result = vec![];

        loop {
            if let DocstringComment { global, content } = self.next.unwrap() {
                if !global {
                    result.push(content.clone());
                }
            } else {
                return Ok(result);
            }

            self.advance_with_docstring()?;
        }
    }

    /// Returns [`ParseResult<ProgramUnit>`] where [`ProgramUnit`] represents
    /// AST for a Ry module.
    /// ```
    /// use ry_parser::Parser;
    /// use ry_interner::Interner;
    ///
    /// let mut interner = Interner::default();
    /// let mut parser = Parser::new("fun test() {}", &mut interner);
    /// assert!(parser.parse().is_ok());
    /// ```
    pub fn parse(&mut self) -> ParseResult<ProgramUnit> {
        self.check_scanning_error_for_current_token()?;

        let (global_docstring, fst_docstring) = self.consume_module_and_first_item_docstrings()?;
        Ok(ProgramUnit {
            docstring: global_docstring,
            items: self.parse_items(fst_docstring)?,
        })
    }

    fn parse_items(&mut self, mut local_docstring: Docstring) -> ParseResult<Items> {
        let mut items = vec![];

        loop {
            items.push(
                match self.next.unwrap() {
                    Keyword(Fun) => self.parse_function_item(Visibility::private())?,
                    Keyword(Struct) => self.parse_struct_declaration(Visibility::private())?,
                    Keyword(Trait) => self.parse_trait_declaration(Visibility::private())?,
                    Keyword(Enum) => self.parse_enum_declaration(Visibility::private())?,
                    Keyword(Impl) => self.parse_impl(Visibility::private())?,
                    Keyword(Pub) => {
                        let visibility = Visibility::public(self.next.span());

                        self.advance()?;

                        match self.next.unwrap() {
                            Keyword(Fun) => self.parse_function_item(visibility)?,
                            Keyword(Struct) => self.parse_struct_declaration(visibility)?,
                            Keyword(Trait) => self.parse_trait_declaration(visibility)?,
                            Keyword(Enum) => self.parse_enum_declaration(visibility)?,
                            Keyword(Impl) => self.parse_impl(visibility)?,
                            _ => {
                                return Err(ParseError::unexpected_token(
                                    self.next.clone(),
                                    "`fun`, `trait`, `enum`, `struct`",
                                    "item after `pub`",
                                ));
                            }
                        }
                    }
                    Keyword(Import) => self.parse_import()?,
                    EndOfFile => break,
                    _ => {
                        let err = Err(ParseError::unexpected_token(
                            self.next.clone(),
                            "`import`, `fun`, `trait`, `enum`, `struct`",
                            "item",
                        ));
                        self.advance()?;
                        return err;
                    }
                }
                .with_docstring(local_docstring),
            );

            local_docstring = self.consume_non_module_docstring()?;
        }

        Ok(items)
    }
}
