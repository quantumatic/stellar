//! This crate provides a iter for Stellar programming language
//!
//! It uses the lexer from the [`stellar_lexer`] crate to tokenize the input source
//! code and produces an Abstract Syntax Tree (AST) that represents the parsed code.
//!
//! [`stellar_lexer`]: ../stellar_lexer/index.html

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]
#![warn(clippy::dbg_macro, missing_docs)]
#![warn(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
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
    clippy::unnested_or_patterns,
    clippy::inline_always
)]

pub mod diagnostics;
mod expression;
mod items;
mod list;
mod literal;
mod path;
mod pattern;
mod statement;
mod r#type;

use std::{fs, io};

use diagnostics::LexErrorDiagnostic;
pub use expression::ExpressionParser;
use items::{ItemParser, ItemsParser};
use parking_lot::RwLock;
use pattern::PatternParser;
use r#type::TypeParser;
use statement::StatementParser;
use stellar_ast::{
    token::{Keyword, LexError, RawToken, Token},
    Expression, IdentifierAST, Module, ModuleItem, Pattern, Statement, Type, Visibility,
};
use stellar_diagnostics::{BuildDiagnostic, Diagnostics};
use stellar_filesystem::location::{ByteOffset, Location, LocationIndex};
use stellar_interner::PathID;
use stellar_lexer::Lexer;
use stellar_stable_likely::unlikely;
use tracing::trace;

use crate::diagnostics::UnexpectedToken;

/// Represents a parse state.
#[derive(Debug)]
pub struct ParseState<'s, 'd> {
    /// Lexer that is used for parsing.
    lexer: Lexer<'s>,

    /// Current token.
    current_token: Token,
    /// Next token.
    next_token: Token,

    /// Diagnostics that is emitted during parsing.
    diagnostics: &'d RwLock<Diagnostics>,
}

/// Represents AST node that can be parsed.
pub trait Parse: Sized {
    /// Output AST node type.
    type Output;

    /// Parse AST node of type [`Self::Output`].
    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output;
}

/// Represents AST node that can optionally be parsed. Optionally
/// in this context means that if some condition is satisfied,
/// the AST node is parsed as usually (`Parse::parse(...)`),
/// but if not, it is skipped, token state is not advanced and the
/// default value is returned.
///
/// A great example of this is the where clause, which is found optional
/// in the syntax definition of every item in the Stellar programming language.
/// To avoid copying the behaviour described below, this trait must
/// be implemented.
pub trait OptionallyParse: Sized {
    /// Output AST node type.
    type Output;

    /// Optionally parse AST node of type [`Self::Output`].
    ///
    /// For more information, see [`OptionallyParse`].
    fn optionally_parse(self, state: &mut ParseState<'_, '_>) -> Self::Output;
}

/// Read and parse a Stellar module.
///
/// # Errors
/// Returns an error if the file contents cannot be read.
///
/// # Panics
/// Panics if the file path cannot be resolved in the path storage.
#[inline(always)]
pub fn read_and_parse_module(
    filepath: PathID,
    diagnostics: &RwLock<Diagnostics>,
) -> Result<Module, io::Error> {
    Ok(parse_module_using(ParseState::new(
        filepath,
        &fs::read_to_string(filepath.resolve_or_panic())?,
        diagnostics,
    )))
}

/// Parse a Stellar module.
#[inline(always)]
#[must_use]
pub fn parse_module(filepath: PathID, source: &str, diagnostics: &RwLock<Diagnostics>) -> Module {
    parse_module_using(ParseState::new(filepath, source, diagnostics))
}

/// Parse a Stellar module using a given parse state.
#[inline(always)]
#[must_use]
pub fn parse_module_using(mut state: ParseState<'_, '_>) -> Module {
    Module {
        filepath: state.lexer.filepath,
        docstring: state.consume_module_docstring(),
        items: ItemsParser.parse(&mut state),
    }
}

/// Parse an item.
#[inline(always)]
#[must_use]
pub fn parse_item(
    filepath: PathID,
    source: impl AsRef<str>,
    diagnostics: &RwLock<Diagnostics>,
) -> Option<ModuleItem> {
    parse_item_using(&mut ParseState::new(filepath, source.as_ref(), diagnostics))
}

/// Parse an item.
#[inline(always)]
#[must_use]
pub fn parse_item_using(state: &mut ParseState<'_, '_>) -> Option<ModuleItem> {
    ItemParser.parse(state)
}

/// Parse an expression.
#[inline(always)]
#[must_use]
pub fn parse_expression(
    filepath: PathID,
    source: impl AsRef<str>,
    diagnostics: &RwLock<Diagnostics>,
) -> Option<Expression> {
    parse_expression_using(&mut ParseState::new(filepath, source.as_ref(), diagnostics))
}

/// Parse an expression.
#[inline(always)]
#[must_use]
pub fn parse_expression_using(state: &mut ParseState<'_, '_>) -> Option<Expression> {
    ExpressionParser::default().parse(state)
}

/// Parse a statement.
#[inline(always)]
#[must_use]
pub fn parse_statement(
    filepath: PathID,
    source: impl AsRef<str>,
    diagnostics: &RwLock<Diagnostics>,
) -> Option<Statement> {
    parse_statement_using(&mut ParseState::new(filepath, source.as_ref(), diagnostics))
}

/// Parse a statement.
#[inline(always)]
#[must_use]
pub fn parse_statement_using(state: &mut ParseState<'_, '_>) -> Option<Statement> {
    StatementParser.parse(state).map(|s| s.statement)
}

/// Parse a type.
#[inline(always)]
#[must_use]
pub fn parse_type(
    filepath: PathID,
    source: impl AsRef<str>,
    diagnostics: &RwLock<Diagnostics>,
) -> Option<Type> {
    parse_type_using(&mut ParseState::new(filepath, source.as_ref(), diagnostics))
}

/// Parse a type.
#[inline(always)]
#[must_use]
pub fn parse_type_using(state: &mut ParseState<'_, '_>) -> Option<Type> {
    TypeParser.parse(state)
}

/// Parse a pattern.
#[inline(always)]
#[must_use]
pub fn parse_pattern(
    filepath: PathID,
    source: impl AsRef<str>,
    diagnostics: &RwLock<Diagnostics>,
) -> Option<Pattern> {
    parse_pattern_using(&mut ParseState::new(filepath, source.as_ref(), diagnostics))
}

/// Parse a pattern.
#[inline(always)]
#[must_use]
pub fn parse_pattern_using(state: &mut ParseState<'_, '_>) -> Option<Pattern> {
    PatternParser.parse(state)
}

impl<'s, 'd> ParseState<'s, 'd> {
    /// Creates an initial parse state from file source.
    #[must_use]
    pub fn new(filepath_id: PathID, source: &'s str, diagnostics: &'d RwLock<Diagnostics>) -> Self {
        let mut lexer = Lexer::new(filepath_id, source);

        let current_token = lexer.next_no_comments();
        trace!(
            "next_token: {} at {}",
            current_token.raw,
            current_token.location
        );

        let next_token = current_token;

        let mut state = Self {
            lexer,
            current_token,
            next_token,
            diagnostics,
        };
        state.check_next_token();

        state
    }

    /// Adds diagnostic if the next token has lex error in itself.
    #[inline(always)]
    fn check_next_token(&mut self) {
        if let RawToken::Error(error) = self.next_token.raw {
            self.add_diagnostic(LexErrorDiagnostic::new(LexError {
                location: self.next_token.location,
                raw: error,
            }));
        }
    }

    /// Returns string slice corresponding to the given location.
    #[inline(always)]
    #[must_use]
    fn resolve_location(&self, location: Location) -> &str {
        self.lexer.source.index(location)
    }

    /// Returns string slice corresponding to the current token's location.
    #[inline(always)]
    #[must_use]
    fn resolve_current(&self) -> &str {
        self.resolve_location(self.current_token.location)
    }

    /// Advances the iter to the next token (skips comment tokens).
    fn advance(&mut self) {
        self.check_next_token();

        self.current_token = self.next_token;
        self.next_token = self.lexer.next_no_comments();

        trace!(
            "next_token: {} at {}",
            self.next_token.raw,
            self.next_token.location
        );
    }

    /// Checks if the next token is [`expected`].
    fn expect(&mut self, expected: RawToken) -> Option<()> {
        trace!(
            "excepted {} to be {} at: {}",
            self.next_token.raw,
            expected,
            self.next_token.location
        );

        if unlikely(self.next_token.raw.is_error()) {
            return None;
        }

        if self.next_token.raw == expected {
            Some(())
        } else {
            self.add_diagnostic(UnexpectedToken::new(
                self.current_token.location.end,
                self.next_token,
                expected,
            ));

            None
        }
    }

    /// Checks if the next token is [`expected`] and advances the parse state.
    fn consume(&mut self, expected: impl Into<RawToken>) -> Option<()> {
        self.expect(expected.into())?;
        self.advance();
        Some(())
    }

    /// Creates a new location with the parser state's file id and
    /// the given starting and ending byte offsets.
    #[inline(always)]
    pub(crate) const fn make_location(&self, start: ByteOffset, end: ByteOffset) -> Location {
        Location {
            filepath_id: self.lexer.filepath,
            start,
            end,
        }
    }

    /// Creates a new location with the state's file id and
    /// ending with a current token location's end byte location.
    #[inline(always)]
    pub(crate) const fn location_from(&self, start_offset: ByteOffset) -> Location {
        self.make_location(start_offset, self.current_token.location.end)
    }

    /// Checks if the next token is identifiers, advances the parse state and if
    /// everything is ok, returns the identifier symbol.
    fn consume_identifier(&mut self) -> Option<IdentifierAST> {
        trace!(
            "expected next_token {} to be an identifier at: {}",
            self.next_token.raw,
            self.next_token.location
        );

        let locationned_symbol = if self.next_token.raw == RawToken::Identifier {
            IdentifierAST {
                location: self.next_token.location,
                id: self.lexer.scanned_identifier,
            }
        } else {
            self.add_diagnostic(UnexpectedToken::new(
                self.current_token.location.end,
                self.next_token,
                "identifier",
            ));
            return None;
        };

        self.advance();

        Some(locationned_symbol)
    }

    /// Consumes the docstring for a module.
    pub(crate) fn consume_module_docstring(&mut self) -> Option<String> {
        if self.next_token.raw == RawToken::GlobalDocComment {
            let mut module_docstring = String::new();

            while self.next_token.raw == RawToken::GlobalDocComment {
                self.advance();

                module_docstring.push_str(self.resolve_location(self.current_token.location));
            }

            trace!("consumed module level docstring");

            Some(module_docstring)
        } else {
            None
        }
    }

    /// Consumes the docstring for a local item.
    pub(crate) fn consume_local_docstring(&mut self) -> Option<String> {
        if self.next_token.raw == RawToken::LocalDocComment {
            let mut local_docstring = String::new();

            while self.next_token.raw == RawToken::LocalDocComment {
                self.advance();

                local_docstring.push_str(self.resolve_location(self.current_token.location));
            }

            trace!("consumed docstring");

            Some(local_docstring)
        } else {
            None
        }
    }

    /// Saves a single file diagnostic.
    #[inline(always)]
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn add_diagnostic(&mut self, diagnostic: impl BuildDiagnostic) {
        self.diagnostics
            .write()
            .add_single_file_diagnostic(self.lexer.filepath, diagnostic);
    }
}

pub(crate) struct VisibilityParser;

impl Parse for VisibilityParser {
    type Output = Visibility;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        if state.next_token.raw == Keyword::Pub {
            state.advance();

            Visibility::Public(state.current_token.location)
        } else {
            Visibility::Private
        }
    }
}