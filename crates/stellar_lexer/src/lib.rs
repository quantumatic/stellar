//! This crate provides a lexer for Stellar programming language.
//!
//! Lexer is a first stage of compilation, that converts raw sources
//! into [`Token`]s.
//!
//! ```txt
//! "fun main() { println(\"hello\") }" -> l = Lexer::new(filepath, source)
//!                                          |_ l.next_token() = Keyword(Fun)
//!                                          |_ l.next_token() = Identifier(main)
//!                                          |
//!                                         ...
//!                                          |
//!                                          |_ l.next_token() = EndOfFile
//! ```
//!
//! See [`Lexer`] for more information.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]
#![warn(missing_docs, clippy::dbg_macro)]
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
    clippy::unnested_or_patterns
)]

use std::{mem, str::Chars, string::String};

use stellar_ast::token::{resolve_keyword, LexError, Punctuator, RawLexError, RawToken, Token};
use stellar_filesystem::location::{ByteOffset, Location};
use stellar_interner::{IdentifierId, PathId};
use stellar_stable_likely::unlikely;

mod number;

/// # Lexer
///
/// Lexer is fairly standart. It returns [`type@Token`] and then advances its state on
/// each iteration and stops at eof (always returns [`EndOfFile`]).
/// ```
/// use stellar_lexer::Lexer;
/// use stellar_ast::token::{Token, RawToken::EndOfFile};
/// use stellar_filesystem::location::{Location, ByteOffset};
/// use stellar_interner::DUMMY_PATH_ID;
///
/// let mut lexer = Lexer::new(DUMMY_PATH_ID, "");
///
/// assert_eq!(
///     lexer.next_token(),
///     Token {
///         raw: EndOfFile,
///         location: Location {
///             filepath: DUMMY_PATH_ID,
///             start: ByteOffset(0),
///             end: ByteOffset(1)
///         }
///     }
/// );
/// ```
///
/// If error appeared in the process, [`Error`] token will be returned:
///
/// ```
/// use stellar_lexer::Lexer;
/// use stellar_ast::token::{RawLexError, RawToken::Error};
/// use stellar_interner::DUMMY_PATH_ID;
///
/// let mut lexer = Lexer::new(DUMMY_PATH_ID, "١");
///
/// assert_eq!(lexer.next_token().raw, Error(RawLexError::UnexpectedChar));
/// ```
///
/// # Note
///
/// The lexer makes use of the [`stellar_interner`] crate to perform string interning,
/// a process of deduplicating strings, which can be highly beneficial when dealing with
/// identifiers.
///
/// [`EndOfFile`]: stellar_ast::token::RawToken::EndOfFile
/// [`Error`]: stellar_ast::token::RawToken::Error
#[derive(Debug)]
pub struct Lexer<'s> {
    /// ID of the path of the file being scanned.
    pub filepath: PathId,

    /// Content of the file being scanned.
    pub source: &'s str,

    /// Current character.
    ///
    /// **NOTE**: Can easily be stored as `Option<char>` without worrying about additional discriminant
    /// space, because `None` is represented as `1114112u32` (not all `u32`s are
    /// valid `char`s). See https://godbolt.org/z/5nG9Pjoxh.
    ///
    /// ```
    /// assert!(std::mem::size_of::<char>() == std::mem::size_of::<Option<char>>());
    /// ```
    current: Option<char>,

    /// Next character.
    ///
    /// **NOTE**: Can be stored as `Option<char>` without worrying about additional discriminant
    /// space. See [`Lexer::current`] for more details.
    next: Option<char>,

    /// Iterator through source text characters.
    chars: Chars<'s>,

    /// Offset of the current character being processed.
    offset: ByteOffset,

    /* Temporary buffers */
    /// Symbol corresponding to an identifier being processed early on.
    pub scanned_identifier: IdentifierId,

    /// Buffer for storing scanned characters (after processing escape sequences).
    pub scanned_char: char,

    /// Buffer for storing scanned strings (after processing escape sequences).
    scanned_string: String,
}

impl<'s> Lexer<'s> {
    /// Creates a [`Lexer`] state instantiated at the first character.
    #[inline]
    #[must_use]
    pub fn new(filepath: PathId, source: &'s str) -> Self {
        let mut chars = source.chars();

        Self {
            filepath,
            source,
            current: chars.next(),
            next: chars.next(),
            chars,
            offset: ByteOffset(0),
            scanned_identifier: IdentifierId(0),
            scanned_char: '\0',
            scanned_string: String::new(),
        }
    }

    /// Returns a string being scanned early on (after processing escape sequences) and
    /// cleans internal lexer string buffer. So it must be used only once!
    #[inline]
    #[must_use]
    pub fn scanned_string(&mut self) -> String {
        mem::take(&mut self.scanned_string)
    }

    /// Returns a string being scanned early on (after processing escape sequences).
    #[inline]
    #[must_use]
    pub fn scanned_string_slice(&self) -> &str {
        &self.scanned_string
    }

    /// Returns `true` if current character is EOF.
    ///
    /// **NOTE**: Null bytes are taken into account, so if current character is `null`,
    /// then `false` is returned.
    const fn eof(&self) -> bool {
        self.current.is_none()
    }

    /// Skips whitespace characters.
    fn eat_whitespaces(&mut self) {
        while is_whitespace(self.current) {
            self.advance();
        }
    }

    /// Advances the lexer state to the next character.
    fn advance(&mut self) {
        let previous = self.current;

        self.current = self.next;
        self.next = self.chars.next();

        self.offset += match previous {
            Some(c) => c.len_utf8(),
            None => 0, // if it's EOF, stay at the same offset
        };
    }

    /// Advances the lexer state to the next 2 characters (calls
    /// [`Lexer::advance()`] twice).
    fn advance_twice(&mut self) {
        self.advance();
        self.advance();
    }

    /// Advances the lexer state to the next character, and returns the token
    /// with location being the current character location in the source text.
    fn advance_with(&mut self, raw: impl Into<RawToken>) -> Token {
        let token = Token {
            raw: raw.into(),
            location: self.current_char_location(),
        };

        self.advance();
        token
    }

    /// Returns a location with given start and end [`ByteOffset`] offsets
    /// and with the lexer's currently processed file path id.
    const fn make_location(&self, start: ByteOffset, end: ByteOffset) -> Location {
        Location {
            filepath: self.filepath,
            start,
            end,
        }
    }

    /// Returns a location of the current character.
    #[allow(clippy::missing_const_for_fn)] // `+` for `ByteOffset` is not a const operator.
    fn current_char_location(&self) -> Location {
        self.make_location(self.offset, self.offset + 1)
    }

    /// Returns a location ending with the current character's location.
    const fn location_from(&self, start_offset: ByteOffset) -> Location {
        self.make_location(start_offset, self.offset)
    }

    /// Advances the lexer state to the next 2 characters, and returns the token
    /// with location being `self.location..self.location + 2`.
    fn advance_twice_with(&mut self, raw: impl Into<RawToken>) -> Token {
        let token = Token {
            raw: raw.into(),
            location: self.make_location(self.offset, self.offset + 2),
        };

        self.advance_twice();
        token
    }

    /// Advances the lexer state to the next character while `f` returns `true`,
    /// where its arguments are the current and next characters.
    /// Returns the string slice of source text between `start_offset`
    /// and `self.offset` when `f` returns `false` OR `self.eof() == true`.
    #[inline]
    fn advance_while<F>(&mut self, start_offset: ByteOffset, mut f: F) -> &'s str
    where
        F: FnMut(Option<char>, Option<char>) -> bool,
    {
        while f(self.current, self.next) && !self.eof() {
            self.advance();
        }

        &self.source[start_offset.0..self.offset.0]
    }

    /// Processes an escape sequence.
    fn process_escape_sequence(&mut self) -> Result<char, LexError> {
        self.advance(); // `\`

        let r = match self.current {
            Some('b') => Ok('\u{0008}'),
            Some('f') => Ok('\u{000C}'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('\'') => Ok('\''),
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('\0') => Err(LexError {
                raw: RawLexError::EmptyEscapeSequence,
                location: self.current_char_location(),
            }),
            Some('u') => {
                let start_offset = self.offset;

                self.advance();

                if self.current != Some('{') {
                    return Err(LexError {
                        raw: RawLexError::ExpectedOpenBracketInUnicodeEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                self.advance();

                let mut buffer = String::new();

                for _ in 0..4 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err(LexError {
                            raw: RawLexError::ExpectedDigitInUnicodeEscapeSequence,
                            location: self.current_char_location(),
                        });
                    }

                    // SAFETY: `None.is_ascii_hexdigit()` is guaranteed to be `false`.
                    // So if `self.current` is `None`, then the branch will not be executed.
                    buffer.push(unsafe { self.current.unwrap_unchecked() });
                    self.advance();
                }

                if self.current != Some('}') {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInUnicodeEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidUnicodeEscapeSequence,
                        location: self.location_from(start_offset),
                    }),
                }
            }
            Some('U') => {
                let start_offset = self.offset;

                self.advance();

                if self.current != Some('{') {
                    return Err(LexError {
                        raw: RawLexError::ExpectedOpenBracketInUnicodeEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                self.advance();

                let mut buffer = String::new();

                for _ in 0..8 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err(LexError {
                            raw: RawLexError::ExpectedDigitInUnicodeEscapeSequence,
                            location: self.current_char_location(),
                        });
                    }

                    // SAFETY: `None.is_ascii_hexdigit()` is guaranteed to be `false`.
                    // So if `self.current` is `None`, then the branch will not be executed.
                    buffer.push(unsafe { self.current.unwrap_unchecked() });
                    self.advance();
                }

                if self.current != Some('}') {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInUnicodeEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidUnicodeEscapeSequence,
                        location: self.location_from(start_offset),
                    }),
                }
            }
            Some('x') => {
                let start_offset = self.offset;

                self.advance();

                if self.current != Some('{') {
                    return Err(LexError {
                        raw: RawLexError::ExpectedOpenBracketInByteEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                self.advance();

                let mut buffer = String::new();

                for _ in 0..2 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err(LexError {
                            raw: RawLexError::ExpectedDigitInByteEscapeSequence,
                            location: self.current_char_location(),
                        });
                    }

                    // SAFETY: `None.is_ascii_hexdigit()` is guaranteed to be `false`.
                    // So if `self.current` is `None`, then the branch will not be executed.
                    buffer.push(unsafe { self.current.unwrap_unchecked() });
                    self.advance();
                }

                if self.current != Some('}') {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInByteEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidByteEscapeSequence,
                        location: self.location_from(start_offset),
                    }),
                }
            }
            _ => Err(LexError {
                raw: RawLexError::UnknownEscapeSequence,
                location: self.current_char_location(),
            }),
        };

        self.advance();

        r
    }

    /// Tokenize a char literal.
    fn tokenize_char_literal(&mut self) -> Token {
        let start_offset = self.offset;

        self.advance();

        let mut size = 0;

        while self.current != Some('\'') {
            if self.current == Some('\n') || self.eof() {
                return Token {
                    raw: RawToken::Error(RawLexError::UnterminatedCharLiteral),
                    location: self.location_from(start_offset),
                };
            }

            if self.current == Some('\\') {
                let e = self.process_escape_sequence();

                match e {
                    Ok(c) => {
                        self.scanned_char = c;
                    }
                    Err(
                        e @ LexError {
                            raw:
                                RawLexError::InvalidUnicodeEscapeSequence
                                | RawLexError::InvalidByteEscapeSequence,
                            ..
                        },
                    ) => {
                        self.advance();

                        return e.into();
                    }
                    Err(e) => {
                        return e.into();
                    }
                }
            } else {
                // SAFETY: `self.current` is guaranteed to be `Some(..)`. Because if
                // `self.current` is `None`, then the branch will not be executed cause
                // of `self.eof()` condition.
                self.scanned_char = unsafe { self.current.unwrap_unchecked() };
                self.advance();
            }

            size += 1;
        }

        self.advance();

        match size {
            2..=usize::MAX => {
                return Token {
                    raw: RawToken::Error(RawLexError::MoreThanOneCharInCharLiteral),
                    location: self.location_from(start_offset),
                };
            }
            0 => {
                return Token {
                    raw: RawToken::Error(RawLexError::EmptyCharacterLiteral),
                    location: self.location_from(start_offset),
                };
            }
            _ => {}
        }

        Token {
            raw: RawToken::CharLiteral,
            location: self.location_from(start_offset),
        }
    }

    /// Tokenizes a string literal.
    fn tokenize_string_literal(&mut self) -> Token {
        self.scanned_string.clear();
        let start_offset = self.offset;

        self.advance();

        while !self.eof() && self.current != Some('\n') {
            let c = self.current;

            if c == Some('"') {
                break;
            }

            if c == Some('\\') {
                let e = self.process_escape_sequence();

                match e {
                    Ok(c) => {
                        self.scanned_string.push(c);
                    }
                    Err(
                        e @ LexError {
                            raw:
                                RawLexError::InvalidUnicodeEscapeSequence
                                | RawLexError::InvalidByteEscapeSequence,
                            ..
                        },
                    ) => {
                        self.advance();

                        return e.into();
                    }
                    Err(e) => {
                        return e.into();
                    }
                }
            } else {
                // SAFETY: `self.current` is guaranteed to be `Some(..)`. Because if
                // `self.current` is `None`, then the branch will not be executed cause
                // of `!self.eof()` condition.
                self.scanned_string.push(unsafe { c.unwrap_unchecked() });
                self.advance();
            }
        }

        if self.eof() || self.current == Some('\n') {
            return Token {
                raw: RawToken::Error(RawLexError::UnterminatedStringLiteral),
                location: self.location_from(start_offset),
            };
        }

        self.advance();

        Token {
            raw: RawToken::StringLiteral,
            location: self.location_from(start_offset),
        }
    }

    /// Tokenizes a wrapped identifier.
    fn tokenize_wrapped_identifier(&mut self) -> Token {
        let start_location = self.offset;

        self.advance();

        let name = &self.advance_while(start_location, |current, _| {
            current != Some('\n') && current != Some('`')
        })[1..];

        if self.current != Some('`') {
            return Token {
                raw: RawToken::Error(RawLexError::UnterminatedWrappedIdentifier),
                location: self.location_from(start_location),
            };
        }

        if name.is_empty() {
            return Token {
                raw: RawToken::Error(RawLexError::EmptyWrappedIdentifier),
                location: self.location_from(start_location),
            };
        }

        self.advance();

        self.scanned_identifier = IdentifierId::from(name);

        Token {
            raw: RawToken::Identifier,
            location: self.location_from(start_location),
        }
    }

    /// Tokenizes a usual comment (prefix is `//`).
    fn tokenize_comment(&mut self) -> Token {
        // first `/` character is already advanced
        let start_location = self.offset - 1;
        self.advance();

        self.advance_while(start_location + 2, |current, _| (current != Some('\n')));

        Token {
            raw: RawToken::Comment,
            location: self.location_from(start_location),
        }
    }

    /// Tokenizes a doc comment.
    ///
    /// When [`global`] is true,  doc comment is describing
    /// the whole module (3-rd character is `!`) and
    /// when not doc comment is corresponding to trait method, enum variant, etc.
    /// (everything else and the character is `/`).
    fn tokenize_doc_comment(&mut self, global: bool) -> Token {
        // first `/` character is already consumed
        let start_location = self.offset - 1;
        self.advance_twice(); // `/` and (`!` or `/`)

        self.advance_while(start_location + 3, |current, _| (current != Some('\n')));

        Token {
            location: self.location_from(start_location),
            raw: if global {
                RawToken::GlobalDocComment
            } else {
                RawToken::LocalDocComment
            },
        }
    }

    /// Tokenizes either an identifier, a keyword or an underscore.
    fn tokenize_identifier_keyword_or_underscore(&mut self) -> Token {
        let start_location = self.offset;
        let name = self.advance_while(start_location, |current, _| is_id_continue(current));

        if name == "_" {
            return Token {
                raw: RawToken::Punctuator(Punctuator::Underscore),
                location: self.location_from(start_location),
            };
        }

        if let Some(reserved) = resolve_keyword(name) {
            Token {
                raw: reserved.into(),
                location: self.location_from(start_location),
            }
        }
        // Both `true` and `false` are considered boolean literals and are not
        // included in the `Keyword` enum.
        else if name == "true" {
            Token {
                raw: RawToken::TrueBoolLiteral,
                location: self.location_from(start_location),
            }
        } else if name == "false" {
            Token {
                raw: RawToken::FalseBoolLiteral,
                location: self.location_from(start_location),
            }
        } else {
            self.scanned_identifier = IdentifierId::from(name);

            Token {
                raw: RawToken::Identifier,
                location: self.location_from(start_location),
            }
        }
    }

    /// Works the same as [`Lexer::next_token`], but skips comments ([`RawToken::Comment`]).
    pub fn next_no_comments(&mut self) -> Token {
        loop {
            let token = self.next_token();
            if token.raw != RawToken::Comment {
                return token;
            }
        }
    }

    /// Proceeds to the next token and returns it (see [top level documentation](../index.html) for more details).
    pub fn next_token(&mut self) -> Token {
        self.eat_whitespaces();

        // EOF will be processed only once throughout the scanning process compared to all other
        // characters in the input. So this condition is not likely to be `true`, because input files
        // are rarely empty or contain <10 bytes.
        if unlikely(self.eof()) {
            return Token {
                raw: RawToken::EndOfFile,
                location: self.current_char_location(),
            };
        }

        match (self.current, self.next) {
            (Some(':'), _) => self.advance_with(Punctuator::Colon),
            (Some('@'), _) => self.advance_with(Punctuator::At),
            (Some('"'), _) => self.tokenize_string_literal(),
            (Some('\''), _) => self.tokenize_char_literal(),
            (Some('`'), _) => self.tokenize_wrapped_identifier(),
            (Some('+'), Some('+')) => self.advance_twice_with(Punctuator::DoublePlus),
            (Some('+'), Some('=')) => self.advance_twice_with(Punctuator::PlusEq),
            (Some('+'), _) => self.advance_with(Punctuator::Plus),
            (Some('-'), Some('>')) => self.advance_twice_with(Punctuator::Arrow),
            (Some('-'), Some('-')) => self.advance_twice_with(Punctuator::DoubleMinus),
            (Some('-'), Some('=')) => self.advance_twice_with(Punctuator::MinusEq),
            (Some('-'), _) => self.advance_with(Punctuator::Minus),
            (Some('*'), Some('*')) => self.advance_twice_with(Punctuator::DoubleAsterisk),
            (Some('*'), Some('=')) => self.advance_twice_with(Punctuator::AsteriskEq),
            (Some('*'), _) => self.advance_with(Punctuator::Asterisk),
            (Some('#'), _) => self.advance_with(Punctuator::HashTag),
            (Some('/'), Some('/')) => {
                self.advance();

                match self.next {
                    Some('!') => self.tokenize_doc_comment(true),
                    Some('/') => self.tokenize_doc_comment(false),
                    _ => self.tokenize_comment(),
                }
            }
            (Some('/'), Some('=')) => self.advance_twice_with(Punctuator::SlashEq),
            (Some('/'), _) => self.advance_with(Punctuator::Slash),
            (Some('!'), Some('=')) => self.advance_twice_with(Punctuator::BangEq),
            (Some('!'), _) => self.advance_with(Punctuator::Bang),
            (Some('>'), Some('>')) => self.advance_twice_with(Punctuator::RightShift),
            (Some('>'), Some('=')) => self.advance_twice_with(Punctuator::GreaterEq),
            (Some('>'), _) => self.advance_with(Punctuator::Greater),
            (Some('<'), Some('<')) => self.advance_twice_with(Punctuator::LeftShift),
            (Some('<'), Some('=')) => self.advance_twice_with(Punctuator::LessEq),
            (Some('<'), _) => self.advance_with(Punctuator::Less),
            (Some('='), Some('=')) => self.advance_twice_with(Punctuator::DoubleEq),
            (Some('='), _) => self.advance_with(Punctuator::Eq),
            (Some('|'), Some('=')) => self.advance_twice_with(Punctuator::OrEq),
            (Some('|'), Some('|')) => self.advance_twice_with(Punctuator::DoubleOr),
            (Some('|'), _) => self.advance_with(Punctuator::Or),
            (Some('?'), _) => self.advance_with(Punctuator::QuestionMark),
            (Some('&'), Some('&')) => self.advance_twice_with(Punctuator::DoubleAmpersand),
            (Some('&'), _) => self.advance_with(Punctuator::Ampersand),
            (Some('^'), Some('=')) => self.advance_twice_with(Punctuator::CaretEq),
            (Some('^'), _) => self.advance_with(Punctuator::Caret),
            (Some('~'), _) => self.advance_with(Punctuator::Tilde),
            (Some('('), _) => self.advance_with(Punctuator::OpenParent),
            (Some(')'), _) => self.advance_with(Punctuator::CloseParent),
            (Some('['), _) => self.advance_with(Punctuator::OpenBracket),
            (Some(']'), _) => self.advance_with(Punctuator::CloseBracket),
            (Some('{'), _) => self.advance_with(Punctuator::OpenBrace),
            (Some('}'), _) => self.advance_with(Punctuator::CloseBrace),
            (Some(','), _) => self.advance_with(Punctuator::Comma),
            (Some(';'), _) => self.advance_with(Punctuator::Semicolon),
            (Some('%'), Some('=')) => self.advance_with(Punctuator::PercentEq),
            (Some('%'), _) => self.advance_with(Punctuator::Percent),
            (Some('.'), Some('.')) => self.advance_twice_with(Punctuator::DoubleDot),
            _ => {
                if self.current.is_ascii_digit()
                    || (self.current == Some('.') && self.next.is_ascii_digit())
                {
                    return self.tokenize_number();
                } else if is_id_start(self.current) {
                    return self.tokenize_identifier_keyword_or_underscore();
                } else if self.current == Some('.') {
                    return self.advance_with(Punctuator::Dot);
                }

                self.advance_with(RawToken::Error(RawLexError::UnexpectedChar))
            }
        }
    }
}

/// Returns `true` if `c` is a whitespace.
const fn is_whitespace(c: Option<char>) -> bool {
    // Note that it is ok to hard-code the values, because
    // the set is stable and doesn't change with different
    // Unicode versions.
    matches!(
        c,
        Some('\u{0009}')   // \t
        | Some('\u{000A}') // \n
        | Some('\u{000B}') // vertical tab
        | Some('\u{000C}') // form feed
        | Some('\u{000D}') // \r
        | Some('\u{0020}') // space

        // NEXT LINE from latin1
        | Some('\u{0085}')

        // Bidi markers
        | Some('\u{200E}') // LEFT-TO-RIGHT MARK
        | Some('\u{200F}') // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | Some('\u{2028}') // LINE SEPARATOR
        | Some('\u{2029}') // PARAGRAPH SEPARATOR
    )
}

/// Returns `true` if `c` is valid as a first character of an identifier.
fn is_id_start(c: Option<char>) -> bool {
    matches!(c, Some(c) if unicode_xid::UnicodeXID::is_xid_start(c) || c == '_')
}

/// Returns `true` if `c` is valid as a non-first character of an identifier.
fn is_id_continue(c: Option<char>) -> bool {
    matches!(c, Some(c) if unicode_xid::UnicodeXID::is_xid_continue(c))
}

/// Extension trait for `Option<char>` to reduce code duplication.
trait IsAsciiExt {
    /// Returns `true` if `self` is an ASCII digit.
    fn is_ascii_digit(&self) -> bool;

    /// Returns `true` if `self` is an ASCII hex digit.
    fn is_ascii_hexdigit(&self) -> bool;
}

impl IsAsciiExt for Option<char> {
    fn is_ascii_digit(&self) -> bool {
        matches!(self, Some(c) if c.is_ascii_digit())
    }

    fn is_ascii_hexdigit(&self) -> bool {
        matches!(self, Some(c) if c.is_ascii_hexdigit())
    }
}
