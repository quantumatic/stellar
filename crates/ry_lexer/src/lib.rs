//! This crate provides a lexer for Ry programming language.
//!
//! Lexer is a first stage of compilation, state machine that converts
//! source text into [`type@Token`]s.
//!
//! See [`Lexer`] for more information.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
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

use ry_ast::token::{get_keyword, LexError, Punctuator, RawLexError, RawToken, Token};
use ry_filesystem::location::{ByteOffset, Location};
use ry_interner::{IdentifierID, IdentifierInterner, PathID};
use ry_stable_likely::unlikely;

mod number;

/// Represents a lexer state machine.
/// Lexer is fairly standart. It returns [`type@Token`] and then advances its state on
/// each iteration and stops at eof (always returns [`EndOfFile`]).
/// ```
/// # use ry_lexer::Lexer;
/// # use ry_ast::token::{Token, RawToken::EndOfFile};
/// # use ry_interner::IdentifierInterner;
/// # use ry_filesystem::location::{Location, ByteOffsetOffset};
/// # use ry_interner::DUMMY_PATH_ID;
/// let mut identifier_interner = IdentifierInterner::default();
/// let mut lexer = Lexer::new(DUMMY_PATH_ID, "", &mut identifier_interner);
///
/// assert_eq!(
///     lexer.next_token(),
///     Token {
///         raw: EndOfFile,
///         location: Location {
///             file_path_id: DUMMY_PATH_ID,
///             start: ByteOffsetOffset(0),
///             end: ByteOffsetOffset(1)
///         }
///     }
/// );
/// ```
///
/// If error appeared in the process, [`Error`] token will be returned:
///
/// ```
/// # use ry_lexer::Lexer;
/// # use ry_ast::token::{RawLexError, RawToken::Error};
/// # use ry_interner::{IdentifierInterner, DUMMY_PATH_ID};
/// let mut identifier_interner = IdentifierInterner::default();
/// let mut lexer = Lexer::new(DUMMY_PATH_ID, "١", &mut identifier_interner);
///
/// assert_eq!(lexer.next_token().raw, Error(RawLexError::UnexpectedChar));
/// ```
///
/// # Note
/// The lexer makes use of the `ry_interner` crate to perform string interning,
/// a process of deduplicating strings, which can be highly beneficial when dealing with
/// identifiers.
///
/// [`EndOfFile`]: ry_ast::token::RawToken::EndOfFile
/// [`Error`]: ry_ast::token::RawToken::Error
#[derive(Debug)]
pub struct Lexer<'s, 'i> {
    /// ID of the path of the file being scanned.
    pub file_path_id: PathID,

    /// Content of the file being scanned.
    pub source: &'s str,

    /// Identifier interner.
    pub identifier_interner: &'i mut IdentifierInterner,

    /// Current character.
    current: char,
    /// Next character.
    next: char,

    /// Iterator through source text characters.
    chars: Chars<'s>,

    /// Offset of the current character being processed.
    offset: ByteOffset,

    /// Symbol corresponding to an identifier being processed early on.
    pub scanned_identifier: IdentifierID,
    /// Buffer for storing scanned characters (after processing escape sequences).
    pub scanned_char: char,
    /// Buffer for storing scanned strings (after processing escape sequences).
    scanned_string: String,
}

impl<'s, 'i> Lexer<'s, 'i> {
    /// Creates a new [`Lexer`] instance.
    #[inline]
    #[must_use]
    pub fn new(
        file_path_id: PathID,
        source: &'s str,
        identifier_interner: &'i mut IdentifierInterner,
    ) -> Self {
        let mut chars = source.chars();

        let current = chars.next().unwrap_or('\0');
        let next = chars.next().unwrap_or('\0');

        Self {
            file_path_id,
            source,
            current,
            next,
            chars,
            identifier_interner,
            offset: ByteOffset(0),
            scanned_identifier: IdentifierID(0),
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

    /// Returns `true` if current character is EOF (`\0`).
    const fn eof(&self) -> bool {
        self.current == '\0'
    }

    /// Skips whitespace characters. See [`Lexer::is_whitespace()`] for more details.
    fn eat_whitespaces(&mut self) {
        while is_whitespace(self.current) {
            self.advance();
        }
    }

    /// Advances the lexer state to the next character.
    fn advance(&mut self) {
        let previous = self.current;

        self.current = self.next;
        self.next = self.chars.next().unwrap_or('\0');

        self.offset += previous.len_utf8();
    }

    /// Advances the lexer state to the next 2 characters
    /// (calls [`Lexer::advance()`] twice).
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
            file_path_id: self.file_path_id,
            start,
            end,
        }
    }

    /// Returns a location of the current character.
    #[allow(clippy::missing_const_for_fn)]
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
        F: FnMut(char, char) -> bool,
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
            'b' => Ok('\u{0008}'),
            'f' => Ok('\u{000C}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '\'' => Ok('\''),
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '\0' => Err(LexError {
                raw: RawLexError::EmptyEscapeSequence,
                location: self.current_char_location(),
            }),
            'u' => {
                self.advance();

                if self.current != '{' {
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

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInUnicodeEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidUnicodeEscapeSequence,
                        location: self.current_char_location(),
                    }),
                }
            }
            'U' => {
                self.advance();

                if self.current != '{' {
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

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInUnicodeEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidUnicodeEscapeSequence,
                        location: self.current_char_location(),
                    }),
                }
            }
            'x' => {
                self.advance();

                if self.current != '{' {
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

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err(LexError {
                        raw: RawLexError::ExpectedCloseBracketInByteEscapeSequence,
                        location: self.current_char_location(),
                    });
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                    Some(c) => Ok(c),
                    None => Err(LexError {
                        raw: RawLexError::InvalidByteEscapeSequence,
                        location: self.make_location(self.offset - 4, self.offset),
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

        while self.current != '\'' {
            if self.current == '\n' || self.eof() {
                return Token {
                    raw: RawToken::Error(RawLexError::UnterminatedCharLiteral),
                    location: self.location_from(start_offset),
                };
            }

            if self.current == '\\' {
                let e = self.process_escape_sequence();

                match e {
                    Ok(c) => {
                        self.scanned_char = c;
                    }
                    Err(e) => {
                        return Token {
                            location: e.location,
                            raw: RawToken::from(e.raw),
                        }
                    }
                }
            } else {
                self.scanned_char = self.current;
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

        while !self.eof() && self.current != '\n' {
            let c = self.current;

            if c == '"' {
                break;
            }

            if c == '\\' {
                let e = self.process_escape_sequence();

                match e {
                    Ok(c) => {
                        self.scanned_string.push(c);
                    }
                    Err(e) => {
                        return Token {
                            location: e.location,
                            raw: RawToken::from(e.raw),
                        }
                    }
                }
            } else {
                self.scanned_string.push(c);
                self.advance();
            }
        }

        if self.eof() || self.current == '\n' {
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
            current.is_alphanumeric() || current == '_'
        })[1..];

        if self.current != '`' {
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

        self.scanned_identifier = self.identifier_interner.get_or_intern(name);

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

        self.advance_while(start_location + 2, |current, _| (current != '\n'));

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

        self.advance_while(start_location + 3, |current, _| (current != '\n'));

        Token {
            location: self.location_from(start_location),
            raw: if global {
                RawToken::GlobalDocComment
            } else {
                RawToken::LocalDocComment
            },
        }
    }

    /// Tokenizes either an identifier or a keyword.
    fn tokenize_identifier_or_keyword(&mut self) -> Token {
        let start_location = self.offset;
        let name = self.advance_while(start_location, |current, _| is_id_continue(current));

        if let Some(reserved) = get_keyword(name) {
            Token {
                raw: reserved.into(),
                location: self.location_from(start_location),
            }
        } else {
            self.scanned_identifier = self.identifier_interner.get_or_intern(name);
            Token {
                raw: RawToken::Identifier,
                location: self.location_from(start_location),
            }
        }
    }

    /// Works the same as [`Lexer::next_token`], but skips comments ([`RawToken::Comment`]).
    pub fn next_no_comments(&mut self) -> Token {
        loop {
            let t = self.next_token();
            if t.raw != RawToken::Comment {
                return t;
            }
        }
    }

    /// Proceeds to the next token and returns it (see [top level documentation](../index.html) for more details).
    pub fn next_token(&mut self) -> Token {
        self.eat_whitespaces();

        if unlikely(self.eof()) {
            return Token {
                raw: RawToken::EndOfFile,
                location: self.current_char_location(),
            };
        }

        match (self.current, self.next) {
            (':', _) => self.advance_with(Punctuator::Colon),
            ('@', _) => self.advance_with(Punctuator::At),

            ('"', _) => self.tokenize_string_literal(),
            ('\'', _) => self.tokenize_char_literal(),
            ('`', _) => self.tokenize_wrapped_identifier(),

            ('+', '+') => self.advance_twice_with(Punctuator::DoublePlus),
            ('+', '=') => self.advance_twice_with(Punctuator::PlusEq),
            ('+', _) => self.advance_with(Punctuator::Plus),

            ('-', '-') => self.advance_twice_with(Punctuator::DoubleMinus),
            ('-', '=') => self.advance_twice_with(Punctuator::MinusEq),
            ('-', _) => self.advance_with(Punctuator::Minus),

            ('*', '*') => self.advance_twice_with(Punctuator::DoubleAsterisk),
            ('*', '=') => self.advance_twice_with(Punctuator::AsteriskEq),
            ('*', _) => self.advance_with(Punctuator::Asterisk),

            ('#', _) => self.advance_with(Punctuator::HashTag),

            ('/', '/') => {
                self.advance();

                match self.next {
                    '!' => self.tokenize_doc_comment(true),
                    '/' => self.tokenize_doc_comment(false),
                    _ => self.tokenize_comment(),
                }
            }

            ('/', '=') => self.advance_twice_with(Punctuator::SlashEq),
            ('/', _) => self.advance_with(Punctuator::Slash),
            ('!', '=') => self.advance_twice_with(Punctuator::BangEq),
            ('!', _) => self.advance_with(Punctuator::Bang),
            ('>', '>') => self.advance_twice_with(Punctuator::RightShift),
            ('>', '=') => self.advance_twice_with(Punctuator::GreaterEq),
            ('>', _) => self.advance_with(Punctuator::Greater),
            ('<', '<') => self.advance_twice_with(Punctuator::LeftShift),
            ('<', '=') => self.advance_twice_with(Punctuator::LessEq),
            ('<', _) => self.advance_with(Punctuator::Less),
            ('=', '=') => self.advance_twice_with(Punctuator::DoubleEq),
            ('=', '>') => self.advance_twice_with(Punctuator::Arrow),
            ('=', _) => self.advance_with(Punctuator::Eq),
            ('|', '=') => self.advance_twice_with(Punctuator::OrEq),
            ('|', '|') => self.advance_twice_with(Punctuator::DoubleOr),
            ('|', _) => self.advance_with(Punctuator::Or),
            ('?', _) => self.advance_with(Punctuator::QuestionMark),
            ('&', '&') => self.advance_twice_with(Punctuator::DoubleAmpersand),
            ('&', _) => self.advance_with(Punctuator::Ampersand),
            ('^', '=') => self.advance_twice_with(Punctuator::CaretEq),
            ('^', _) => self.advance_with(Punctuator::Caret),
            ('~', _) => self.advance_with(Punctuator::Tilde),
            ('(', _) => self.advance_with(Punctuator::OpenParent),
            (')', _) => self.advance_with(Punctuator::CloseParent),
            ('[', _) => self.advance_with(Punctuator::OpenBracket),
            (']', _) => self.advance_with(Punctuator::CloseBracket),
            ('{', _) => self.advance_with(Punctuator::OpenBrace),
            ('}', _) => self.advance_with(Punctuator::CloseBrace),
            (',', _) => self.advance_with(Punctuator::Comma),
            (';', _) => self.advance_with(Punctuator::Semicolon),
            ('%', '=') => self.advance_with(Punctuator::PercentEq),
            ('%', _) => self.advance_with(Punctuator::Percent),

            ('.', '.') => self.advance_twice_with(Punctuator::DoubleDot),

            _ => {
                if self.current.is_ascii_digit()
                    || (self.current == '.' && self.next.is_ascii_digit())
                {
                    return self.eat_number();
                } else if is_id_start(self.current) {
                    return self.tokenize_identifier_or_keyword();
                } else if self.current == '.' {
                    return self.advance_with(Punctuator::Dot);
                }

                self.advance_with(RawToken::Error(RawLexError::UnexpectedChar))
            }
        }
    }
}

/// True if `c` is a whitespace.
const fn is_whitespace(c: char) -> bool {
    // Note that it is ok to hard-code the values, because
    // the set is stable and doesn't change with different
    // Unicode versions.
    matches!(
        c,
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

/// True if `c` is valid as a first character of an identifier.
fn is_id_start(c: char) -> bool {
    c == '_' || unicode_xid::UnicodeXID::is_xid_start(c)
}

/// True if `c` is valid as a non-first character of an identifier.
fn is_id_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}
