//! This crate provides a lexer for Ry programming language.
//!
//! Lexer is a first stage of compilation, state machine
//! that converts Ry source text into [`type@Token`]s.
//!
//! Whitespaces are ignored during scanning process.
//!
//! Lexer is fairly standart. It returns [`type@Token`] and then advances its state on
//! each iteration and stops at eof (always returns [`EndOfFile`]).
//! ```
//! use ry_lexer::Lexer;
//! use ry_ast::token::RawToken::EndOfFile;
//! use ry_interner::Interner;
//! use ry_source_file::span::{Span, At};
//!
//! let mut interner = Interner::default();
//! let mut lexer = Lexer::new(0, "", &mut interner);
//!
//! assert_eq!(lexer.next_token(), EndOfFile.at(Span::new(0, 1, 0)));
//! ```
//!
//! > Note: the Ry lexer makes use of the `ry_interner` crate to perform string interning,
//! > a process of deduplicating strings, which can be highly beneficial when dealing with
//! > identifiers.
//!
//! If error appeared in the process, [`Error`] token will be returned:
//!
//! ```
//! use ry_lexer::Lexer;
//! use ry_ast::token::{LexError, RawToken::Error};
//! use ry_interner::Interner;
//!
//! let mut interner = Interner::default();
//! let mut lexer = Lexer::new(0, "١", &mut interner);
//!
//! assert_eq!(lexer.next_token().unwrap(), &Error(LexError::UnexpectedChar));
//! ```

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

use ry_ast::{
    token::{LexError, RawToken, Token, RESERVED},
    Token,
};
use ry_interner::{Interner, Symbol};
use ry_source_file::span::{At, Span, Spanned};
use std::{str::Chars, string::String};

mod number;

/// Represents a lexer state machine.
#[derive(Debug)]
pub struct Lexer<'a> {
    /// Id of the file being scanned.
    file_id: usize,
    /// Content of the file being scanned.
    source: &'a str,
    /// Current character.
    current: char,
    /// Next character.
    next: char,
    /// Iterator through source text characters.
    chars: Chars<'a>,
    /// Location of the current character being processed.
    location: usize,
    /// Identifier interner.
    interner: &'a mut Interner,
    /// Symbol corresponding to an identifier being processed early on.
    identifier: Symbol,
}

impl<'a> Lexer<'a> {
    /// Creates a new [`Lexer`].
    #[must_use]
    pub fn new(file_id: usize, source: &'a str, interner: &'a mut Interner) -> Self {
        let mut chars = source.chars();

        let current = chars.next().unwrap_or('\0');
        let next = chars.next().unwrap_or('\0');

        Self {
            file_id,
            source,
            current,
            next,
            chars,
            interner,
            location: 0,
            identifier: 0,
        }
    }

    /// Returns a symbol corresponding to an identifier being processed early on.
    #[must_use]
    #[inline]
    pub const fn identifier(&self) -> Symbol {
        self.identifier
    }

    /// Returns `true` if current character is EOF (`\0`).
    #[inline]
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

        self.location += previous.len_utf8();
    }

    /// Advances the lexer state to the next 2 characters
    /// (calls [`Lexer::advance()`] twice).
    fn advance_twice(&mut self) {
        self.advance();
        self.advance();
    }

    /// Advances the lexer state to the next character, and returns the token
    /// with location being the current character location in the source text.
    fn advance_with(&mut self, raw: RawToken) -> Token {
        let r = Token::new(
            raw,
            Span::new(self.location, self.location + 1, self.file_id),
        );
        self.advance();
        r
    }

    /// Advances the lexer state to the next 2 characters, and returns the token
    /// with location being `self.location..self.location + 2`.
    fn advance_twice_with(&mut self, raw: RawToken) -> Token {
        let r = Token::new(
            raw,
            Span::new(self.location, self.location + 2, self.file_id),
        );
        self.advance_twice();
        r
    }

    /// Advances the lexer state to the next character while `f` returns `true`,
    /// where its arguments are the current and next characters.
    /// Returns the string slice of source text between `start_location`
    /// and `self.location` when `f` returns `false` OR `self.eof() == true`.
    fn advance_while<F>(&mut self, start_location: usize, mut f: F) -> &'a str
    where
        F: FnMut(char, char) -> bool,
    {
        while f(self.current, self.next) && !self.eof() {
            self.advance();
        }

        &self.source[start_location..self.location]
    }

    /// Parses an escape sequence.
    fn eat_escape(&mut self) -> Result<char, Spanned<LexError>> {
        self.advance(); // `\`
        let r =
            match self.current {
                'b' => Ok('\u{0008}'),
                'f' => Ok('\u{000C}'),
                'n' => Ok('\n'),
                'r' => Ok('\r'),
                't' => Ok('\t'),
                '\'' => Ok('\''),
                '"' => Ok('"'),
                '\\' => Ok('\\'),
                '\0' => Err(LexError::EmptyEscapeSequence.at(Span::new(
                    self.location,
                    self.location + 1,
                    self.file_id,
                ))),
                'u' => {
                    self.advance();

                    if self.current != '{' {
                        return Err(LexError::ExpectedOpenBracketInUnicodeEscapeSequence
                            .at(Span::new(self.location, self.location + 1, self.file_id)));
                    }

                    self.advance();

                    let mut buffer = String::new();

                    for _ in 0..4 {
                        if !self.current.is_ascii_hexdigit() {
                            return Err(LexError::ExpectedDigitInUnicodeEscapeSequence
                                .at(Span::new(self.location, self.location + 1, self.file_id)));
                        }

                        buffer.push(self.current);
                        self.advance();
                    }

                    if self.current != '}' {
                        return Err(LexError::ExpectedCloseBracketInUnicodeEscapeSequence
                            .at(Span::new(self.location, self.location + 1, self.file_id)));
                    }

                    match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                        Some(c) => Ok(c),
                        None => Err(LexError::InvalidUnicodeEscapeSequence.at(Span::new(
                            self.location,
                            self.location + 1,
                            self.file_id,
                        ))),
                    }
                }
                'U' => {
                    self.advance();

                    if self.current != '{' {
                        return Err(LexError::ExpectedOpenBracketInUnicodeEscapeSequence
                            .at(Span::new(self.location, self.location + 1, self.file_id)));
                    }

                    self.advance();

                    let mut buffer = String::new();

                    for _ in 0..8 {
                        if !self.current.is_ascii_hexdigit() {
                            return Err(LexError::ExpectedDigitInUnicodeEscapeSequence
                                .at(Span::new(self.location, self.location + 1, self.file_id)));
                        }

                        buffer.push(self.current);
                        self.advance();
                    }

                    if self.current != '}' {
                        return Err(LexError::ExpectedCloseBracketInUnicodeEscapeSequence
                            .at(Span::new(self.location, self.location + 1, self.file_id)));
                    }

                    match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                        Some(c) => Ok(c),
                        None => Err(LexError::InvalidUnicodeEscapeSequence.at(Span::new(
                            self.location,
                            self.location + 1,
                            self.file_id,
                        ))),
                    }
                }
                'x' => {
                    self.advance();

                    if self.current != '{' {
                        return Err(LexError::ExpectedOpenBracketInByteEscapeSequence
                            .at(Span::new(self.location, self.location + 1, self.file_id)));
                    }

                    self.advance();

                    let mut buffer = String::new();

                    for _ in 0..2 {
                        if !self.current.is_ascii_hexdigit() {
                            return Err(LexError::ExpectedDigitInByteEscapeSequence.at(Span::new(
                                self.location,
                                self.location + 1,
                                self.file_id,
                            )));
                        }

                        buffer.push(self.current);
                        self.advance();
                    }

                    if self.current != '}' {
                        return Err(LexError::ExpectedCloseBracketInByteEscapeSequence
                            .at(Span::new(self.location, self.location + 1, self.file_id)));
                    }

                    match char::from_u32(u32::from_str_radix(&buffer, 16).expect("Invalid hex")) {
                        Some(c) => Ok(c),
                        None => Err(LexError::InvalidByteEscapeSequence.at(Span::new(
                            self.location - 4,
                            self.location,
                            self.file_id,
                        ))),
                    }
                }
                _ => Err(LexError::UnknownEscapeSequence.at(Span::new(
                    self.location,
                    self.location + 1,
                    self.file_id,
                ))),
            };

        self.advance();

        r
    }

    /// Parses a char literal.
    fn eat_char(&mut self) -> Token {
        let start_location = self.location;

        self.advance();

        let mut size = 0;

        while self.current != '\'' {
            if self.current == '\n' || self.eof() {
                return RawToken::Error(LexError::UnterminatedCharLiteral).at(Span::new(
                    start_location,
                    self.location,
                    self.file_id,
                ));
            }

            if self.current == '\\' {
                let e = self.eat_escape();

                if let Err(e) = e {
                    return RawToken::from(*e.unwrap()).at(e.span());
                }
            } else {
                self.advance();
            }

            size += 1;
        }

        self.advance();

        match size {
            2..=usize::MAX => {
                return RawToken::Error(LexError::MoreThanOneCharInCharLiteral).at(Span::new(
                    start_location,
                    self.location,
                    self.file_id,
                ));
            }
            0 => {
                return RawToken::Error(LexError::EmptyCharLiteral).at(Span::new(
                    start_location,
                    self.location,
                    self.file_id,
                ));
            }
            _ => {}
        }

        RawToken::CharLiteral.at(Span::new(start_location, self.location, self.file_id))
    }

    /// Parses a string literal.
    fn eat_string(&mut self) -> Token {
        let start_location = self.location;

        self.advance();

        while !self.eof() && self.current != '\n' {
            let c = self.current;

            if c == '"' {
                break;
            }

            if c == '\\' {
                let e = self.eat_escape();

                if let Err(e) = e {
                    return RawToken::from(*e.unwrap()).at(e.span());
                }
            } else {
                self.advance();
            }
        }

        if self.eof() || self.current == '\n' {
            return RawToken::Error(LexError::UnterminatedStringLiteral).at(Span::new(
                start_location,
                self.location,
                self.file_id,
            ));
        }

        self.advance();

        RawToken::StringLiteral.at(Span::new(start_location, self.location, self.file_id))
    }

    /// Parses a wrapped identifier.
    fn eat_wrapped_id(&mut self) -> Token {
        let start_location = self.location;

        self.advance();

        let name = &self.advance_while(start_location, |current, _| {
            current.is_alphanumeric() || current == '_'
        })[1..];

        if self.current != '`' {
            return RawToken::Error(LexError::UnterminatedWrappedIdentifier).at(Span::new(
                start_location,
                self.location,
                self.file_id,
            ));
        }

        if name.is_empty() {
            return RawToken::Error(LexError::EmptyWrappedIdentifier).at(Span::new(
                start_location,
                self.location,
                self.file_id,
            ));
        }

        self.advance();

        self.identifier = self.interner.get_or_intern(name);
        RawToken::Identifier.at(Span::new(start_location, self.location, self.file_id))
    }

    /// Parses a usual comment (prefix is `//`).
    fn eat_comment(&mut self) -> Token {
        // first `/` character is already advanced
        let start_location = self.location - 1;
        self.advance();

        self.advance_while(start_location + 2, |current, _| (current != '\n'));

        RawToken::Comment.at(Span::new(start_location, self.location, self.file_id))
    }

    /// Parses a doc comment.
    ///
    /// When [`global`] is true,  doc comment is describing
    /// the whole module (3-rd character is `!`) and
    /// when not doc comment is corresponding to trait method, enum variant, etc.
    /// (everything else and the character is `/`).
    fn eat_doc_comment(&mut self, global: bool) -> Token {
        // first `/` character is already consumed
        let start_location = self.location - 1;
        self.advance_twice(); // `/` and (`!` or `/`)

        self.advance_while(start_location + 3, |current, _| (current != '\n'));

        if global {
            RawToken::GlobalDocComment
        } else {
            RawToken::LocalDocComment
        }
        .at(Span::new(start_location, self.location, self.file_id))
    }

    /// Parses weather an identifier or a keyword.
    fn eat_name(&mut self) -> Token {
        let start_location = self.location;
        let name = self.advance_while(start_location, |current, _| is_id_continue(current));

        if let Some(reserved) = RESERVED.get(name) {
            (*reserved).at(Span::new(start_location, self.location, self.file_id))
        } else {
            self.identifier = self.interner.get_or_intern(name);
            RawToken::Identifier.at(Span::new(start_location, self.location, self.file_id))
        }
    }

    /// Works the same as [`Lexer::next_token`], but skips comments ([`RawToken::Comment`]).
    pub fn next_no_comments(&mut self) -> Token {
        loop {
            let t = self.next_token();
            if t.unwrap() != &RawToken::Comment {
                return t;
            }
        }
    }

    /// Proceeds to the next token and returns it (see [top level documentation](../index.html) for more details).
    pub fn next_token(&mut self) -> Token {
        self.eat_whitespaces();

        match (self.current, self.next) {
            ('\0', _) => {
                RawToken::EndOfFile.at(Span::new(self.location, self.location + 1, self.file_id))
            }

            (':', _) => self.advance_with(Token![:]),
            ('@', _) => self.advance_with(Token![@]),

            ('"', _) => self.eat_string(),
            ('\'', _) => self.eat_char(),
            ('`', _) => self.eat_wrapped_id(),

            ('+', '+') => self.advance_twice_with(Token![++]),
            ('+', '=') => self.advance_twice_with(Token![+=]),
            ('+', _) => self.advance_with(Token![+]),
            ('-', '-') => self.advance_twice_with(Token![--]),
            ('-', '=') => self.advance_twice_with(Token![-=]),
            ('-', _) => self.advance_with(Token![-]),
            ('*', '*') => self.advance_twice_with(Token![**]),
            ('*', '=') => self.advance_twice_with(Token![*=]),
            ('*', _) => self.advance_with(Token![*]),

            ('#', _) => self.advance_with(Token![#]),

            ('/', '/') => {
                self.advance();

                match self.next {
                    '!' => self.eat_doc_comment(true),
                    '/' => self.eat_doc_comment(false),
                    _ => self.eat_comment(),
                }
            }

            ('/', '=') => self.advance_twice_with(Token![/=]),
            ('/', _) => self.advance_with(Token![/]),
            ('!', '=') => self.advance_twice_with(Token![!=]),
            ('!', _) => self.advance_with(Token![!]),
            ('>', '>') => self.advance_twice_with(Token![>>]),
            ('>', '=') => self.advance_twice_with(Token![>=]),
            ('>', _) => self.advance_with(Token![>]),
            ('<', '<') => self.advance_twice_with(Token![<<]),
            ('<', '=') => self.advance_twice_with(Token![<=]),
            ('<', _) => self.advance_with(Token![<]),
            ('=', '=') => self.advance_twice_with(Token![==]),
            ('=', '>') => self.advance_twice_with(Token![=>]),
            ('=', _) => self.advance_with(Token![=]),
            ('|', '=') => self.advance_twice_with(Token![|=]),
            ('|', '|') => self.advance_twice_with(Token![||]),
            ('|', _) => self.advance_with(Token![|]),
            ('?', _) => self.advance_with(Token![?]),
            ('&', '&') => self.advance_twice_with(Token![&&]),
            ('&', _) => self.advance_with(Token![&]),
            ('^', '=') => self.advance_twice_with(Token![^=]),
            ('^', _) => self.advance_with(Token![^]),
            ('~', _) => self.advance_with(Token![~]),
            ('(', _) => self.advance_with(Token!['(']),
            (')', _) => self.advance_with(Token![')']),
            ('[', _) => self.advance_with(Token!['[']),
            (']', _) => self.advance_with(Token![']']),
            ('{', _) => self.advance_with(Token!['{']),
            ('}', _) => self.advance_with(Token!['}']),
            (',', _) => self.advance_with(Token![,]),
            (';', _) => self.advance_with(Token![;]),
            ('%', '=') => self.advance_with(Token![%=]),
            ('%', _) => self.advance_with(Token![%]),

            ('.', '.') => self.advance_twice_with(Token![..]),

            (c, n) => {
                if number::decimal(c) || (c == '.' && number::decimal(n)) {
                    return self.eat_number();
                } else if is_id_start(c) {
                    return self.eat_name();
                } else if c == '.' {
                    return self.advance_with(Token![.]);
                }

                self.advance_with(RawToken::Error(LexError::UnexpectedChar))
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
