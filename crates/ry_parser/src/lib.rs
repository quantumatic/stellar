//! This crate provides a parser for Ry programming language
//!
//! It uses the lexer from the ry_lexer crate to tokenize the input source
//! code and produces an Abstract Syntax Tree (AST) that represents the parsed code.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::mem_forget,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::mismatched_target_os,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    nonstandard_style,
    unused_import_braces,
    unused_qualifications
)]
#![deny(
    clippy::await_holding_lock,
    clippy::if_let_mutex,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::ok_expect,
    clippy::unimplemented,
    clippy::unwrap_used,
    unsafe_code,
    unstable_features,
    unused_results
)]
#![allow(clippy::match_single_binding, clippy::inconsistent_struct_constructor)]

pub mod error;
mod expression;
mod item;
mod path;
mod statement;
mod r#type;

use error::{expected, ParseError, ParseResult};
use item::ItemsParser;
use ry_ast::{
    declaration::Docstring,
    name::Name,
    span::{At, SpanIndex},
    token::{RawToken, Token},
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

        let current = lexer
            .next_no_comments()
            .unwrap_or(RawToken::EndOfFile.at(0..1));
        let next = current.clone();

        Self {
            lexer,
            current,
            next,
        }
    }

    /// Advances the parser to the next token and skips comment tokens.
    fn next_token(&mut self) {
        self.current = self.next.clone();
        self.next = self
            .lexer
            .next_no_comments()
            .unwrap_or(RawToken::EndOfFile.at(self.current.span.end..self.current.span.end + 1));
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
        self.next_token();
        Ok(())
    }

    fn consume_identifier<N>(&mut self, node: N) -> Result<Name, ParseError>
    where
        N: Into<String>,
    {
        let spanned_symbol = match self.next.inner {
            RawToken::Identifier(symbol) => symbol.at(self.next.span),
            _ => {
                return Err(ParseError::unexpected_token(
                    self.next.clone(),
                    expected!("identifier"),
                    node,
                ));
            }
        };

        self.next_token();

        Ok(spanned_symbol)
    }

    /// Consumes the docstrings for the module and the first item in the module, if present.
    pub(crate) fn consume_module_and_first_item_docstrings(
        &mut self,
    ) -> ParseResult<(Docstring, Docstring)> {
        let (mut module_docstring, mut local_docstring) = (vec![], vec![]);

        loop {
            match self.next.inner {
                RawToken::GlobalDocComment => {
                    module_docstring.push(self.lexer.contents.index(self.next.span).to_owned())
                }
                RawToken::LocalDocComment => {
                    local_docstring.push(self.lexer.contents.index(self.next.span).to_owned())
                }
                _ => return Ok((module_docstring, local_docstring)),
            }

            self.next_token();
        }
    }

    /// Consumes the docstring for a local item (i.e., anything that is not the module docstring
    /// or the first item in the module (because it will be already consumed in
    /// [`Parser::consume_module_and_first_item_docstrings()`])).
    pub(crate) fn consume_docstring(&mut self) -> ParseResult<Docstring> {
        let mut result = vec![];

        loop {
            if self.next.inner == RawToken::LocalDocComment {
                result.push(self.lexer.contents.index(self.next.span).to_owned());
            } else {
                return Ok(result);
            }

            self.next_token();
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
