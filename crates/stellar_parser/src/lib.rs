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

#[cfg(feature = "debug")]
use std::time::Instant;
use std::{fs, io, path::Path};

use diagnostics::LexErrorDiagnostic;
pub use expression::ExpressionParser;
use items::{ItemParser, ItemsParser};
use pattern::PatternParser;
use r#type::TypeParser;
use statement::StatementParser;
use stellar_ast::{
    token::{Keyword, LexError, RawToken, Token},
    Expression, IdentifierAST, Module, ModuleItem, Pattern, Statement, Type, Visibility,
};
use stellar_database::{ModuleData, ModuleID, State};
use stellar_diagnostics::Diagnostics;
use stellar_filesystem::{
    location::{ByteOffset, Location, LocationIndex},
    path_resolver::PackagePathResolver,
};
use stellar_interner::{IdentifierID, PathID};
use stellar_lexer::Lexer;
use stellar_stable_likely::unlikely;
#[cfg(feature = "debug")]
use tracing::trace;
use walkdir::WalkDir;

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
    diagnostics: &'d mut Diagnostics,
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

/// A structure returned by every module parsing function.
#[derive(Debug)]
pub struct ParsedModule {
    module: ModuleID,
    ast: Module,
}

impl ParsedModule {
    /// Creates a new instance of [`ParsedModule`].
    #[inline(always)]
    #[must_use]
    pub const fn new(module_id: ModuleID, ast: Module) -> Self {
        Self {
            module: module_id,
            ast,
        }
    }

    /// Returns the module AST.
    #[inline(always)]
    #[must_use]
    pub const fn ast(&self) -> &Module {
        &self.ast
    }

    /// Returns the ID of the module in database.
    #[inline(always)]
    #[must_use]
    pub const fn module(&self) -> ModuleID {
        self.module
    }

    /// Returns the module AST.
    #[inline(always)]
    #[must_use]
    pub fn into_ast(self) -> Module {
        self.ast
    }

    /// Returns the module AST.
    #[inline(always)]
    #[must_use]
    pub fn ast_mut(&mut self) -> &mut Module {
        &mut self.ast
    }
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
    state: &mut State,
    module_name: IdentifierID,
    filepath: PathID,
) -> Result<ParsedModule, io::Error> {
    let module = ModuleData::alloc(state.db_mut(), module_name, filepath);
    let source = fs::read_to_string(filepath.resolve_or_panic())?;

    let mut parse_state = ParseState::new(filepath, &source, state.diagnostics_mut());

    Ok(ParsedModule {
        module,
        ast: Module {
            filepath: parse_state.lexer.filepath,
            docstring: parse_state.consume_module_docstring(),
            items: ItemsParser.parse(&mut parse_state),
        },
    })
}

/// Parse a Stellar module.
#[inline(always)]
#[must_use]
pub fn parse_module(
    state: &mut State,
    module_name: IdentifierID,
    filepath: PathID,
    source: &str,
) -> ParsedModule {
    let module = ModuleData::alloc(state.db_mut(), module_name, filepath);
    let mut parse_state = ParseState::new(filepath, source, state.diagnostics_mut());

    ParsedModule {
        module,
        ast: Module {
            filepath: parse_state.lexer.filepath,
            docstring: parse_state.consume_module_docstring(),
            items: ItemsParser.parse(&mut parse_state),
        },
    }
}

/// Parse a Stellar module using a given parse state.
#[inline(always)]
#[must_use]
pub fn parse_module_using(
    state: &mut State,
    module_name: IdentifierID,
    mut parse_state: ParseState<'_, '_>,
) -> ParsedModule {
    ParsedModule::new(
        ModuleData::alloc(state.db_mut(), module_name, parse_state.lexer.filepath),
        Module {
            filepath: parse_state.lexer.filepath,
            docstring: parse_state.consume_module_docstring(),
            items: ItemsParser.parse(&mut parse_state),
        },
    )
}

/// Parse an item.
#[inline(always)]
#[must_use]
pub fn parse_item(
    filepath: PathID,
    source: impl AsRef<str>,
    diagnostics: &mut Diagnostics,
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
    diagnostics: &mut Diagnostics,
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
    diagnostics: &mut Diagnostics,
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
    diagnostics: &mut Diagnostics,
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
    diagnostics: &mut Diagnostics,
) -> Option<Pattern> {
    parse_pattern_using(&mut ParseState::new(filepath, source.as_ref(), diagnostics))
}

/// Parse a pattern.
#[inline(always)]
#[must_use]
pub fn parse_pattern_using(state: &mut ParseState<'_, '_>) -> Option<Pattern> {
    PatternParser.parse(state)
}

/// Traverses, reads and parses all package source files.
///
/// # Errors
/// Returns an error if the package's source directory cannot be read.
pub fn parse_package_source_files(
    state: &mut State,
    root: impl AsRef<Path>,
) -> Result<Vec<ParsedModule>, String> {
    fn module_name(path: PathID) -> IdentifierID {
        IdentifierID::from(
            path.resolve_or_panic()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
        )
    }

    let root = root.as_ref();

    let source_directory = PackagePathResolver::new(root).source_directory();

    if !source_directory.exists() {
        return Err(format!(
            "cannot find package's source directory in {}",
            root.display()
        ));
    }

    Ok(
        WalkDir::new(PackagePathResolver::new(root).source_directory())
            .into_iter()
            .filter_map(|entry| {
                let Ok(entry) = entry else {
                    return None;
                };

                #[cfg(feature = "debug")]
                let now = Instant::now();

                let filepath = PathID::from(entry.path());
                let parsing_result = read_and_parse_module(state, module_name(filepath), filepath);

                #[cfg(feature = "debug")]
                trace!(
                    "parse_module(module = '{}') <{} us>",
                    entry.path().display(),
                    now.elapsed().as_micros()
                );

                parsing_result.ok()
            })
            .collect(),
    )
}

impl<'s, 'd> ParseState<'s, 'd> {
    /// Creates an initial parse state from file source.
    #[must_use]
    pub fn new(filepath: PathID, source: &'s str, diagnostics: &'d mut Diagnostics) -> Self {
        let mut lexer = Lexer::new(filepath, source);

        let current_token = lexer.next_no_comments();
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
            self.diagnostics
                .add_file_diagnostic(LexErrorDiagnostic::new(LexError {
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
    }

    /// Checks if the next token is [`expected`].
    fn expect(&mut self, expected: RawToken) -> Option<()> {
        if unlikely(self.next_token.raw.is_error()) {
            return None;
        }

        if self.next_token.raw == expected {
            Some(())
        } else {
            self.diagnostics.add_file_diagnostic(UnexpectedToken::new(
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
            filepath: self.lexer.filepath,
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
        let locationned_symbol = if self.next_token.raw == RawToken::Identifier {
            IdentifierAST {
                location: self.next_token.location,
                id: self.lexer.scanned_identifier,
            }
        } else {
            self.diagnostics.add_file_diagnostic(UnexpectedToken::new(
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
            Some(local_docstring)
        } else {
            None
        }
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
