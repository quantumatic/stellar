//! This crate provides a lexer for Ry programming language.
//!
//! Lexer is a first stage of compilation, state machine
//! that converts Ry source text into [`type@Token`]s.
//!
//! Whitespaces are ignored during scanning process.
//!
//! Lexer is fairly standart. It implements [`Iterator<Item = Token>`] on each step,
//! and stops at eof (always returns [`EndOfFile`] when it's already eof and so iterator
//! never returns [`None`]).
//! ```
//! use ry_lexer::Lexer;
//! use ry_ast::token::RawToken::EndOfFile;
//! use ry_interner::Interner;
//!
//! let mut interner = Interner::default();
//! let mut lexer = Lexer::new(0, "", &mut interner);
//!
//! assert_eq!(lexer.next(), None);
//! assert_eq!(lexer.next(), None); // ok
//! ```
//!
//! Note: the Ry lexer makes use of the `ry_interner` crate to perform string interning,
//! a process of deduplicating strings, which can be highly beneficial when dealing with
//! identifiers.
//!
//! If error appeared in the process, [`Error`] will be returned:
//!
//! ```
//! use ry_lexer::Lexer;
//! use ry_ast::token::{LexError, RawToken::Error};
//! use ry_interner::Interner;
//!
//! let mut interner = Interner::default();
//! let mut lexer = Lexer::new(0, "١", &mut interner);
//!
//! assert_eq!(lexer.next().unwrap().unwrap(), &Error(LexError::UnexpectedChar));
//! ```
use ry_ast::{
    token::{RawToken::*, *},
    Token,
};
use ry_interner::Interner;
use ry_span::{At, Span, Spanned};
use std::{char::from_u32, str::Chars, string::String};

mod number;
mod tests;

#[derive(Debug)]
pub struct Lexer<'a> {
    file_id: usize,
    contents: &'a str,
    current: char,
    next: char,
    chars: Chars<'a>,
    location: usize,
    interner: &'a mut Interner,
}

type IterElem = Option<Token>;

impl<'a> Lexer<'a> {
    pub fn new(file_id: usize, contents: &'a str, interner: &'a mut Interner) -> Self {
        let mut chars = contents.chars();

        let current = chars.next().unwrap_or('\0');
        let next = chars.next().unwrap_or('\0');

        Self {
            file_id,
            contents,
            current,
            next,
            chars,
            interner,
            location: 0,
        }
    }

    fn eof(&self) -> bool {
        self.current == '\0'
    }

    fn eat_whitespaces(&mut self) {
        while is_whitespace(self.current) {
            self.advance();
        }
    }

    fn advance(&mut self) {
        let previous = self.current;

        self.current = self.next;
        self.next = self.chars.next().unwrap_or('\0');

        self.location += previous.len_utf8();
    }

    fn advance_twice(&mut self) {
        self.advance();
        self.advance();
    }

    fn advance_with(&mut self, raw: RawToken) -> IterElem {
        let r = Some(Token::new(
            raw,
            Span::new(self.location, self.location + 1, self.file_id),
        ));
        self.advance();
        r
    }

    fn advance_twice_with(&mut self, raw: RawToken) -> IterElem {
        let r = Some(Token::new(
            raw,
            Span::new(self.location, self.location + 2, self.file_id),
        ));
        self.advance_twice();
        r
    }

    fn advance_while<F>(&mut self, start_location: usize, mut f: F) -> &'a str
    where
        F: FnMut(char, char) -> bool,
    {
        while f(self.current, self.next) && !self.eof() {
            self.advance();
        }

        &self.contents[start_location..self.location]
    }

    fn eat_escape(&mut self) -> Result<char, Spanned<LexError>> {
        self.advance();
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

                    let mut buffer = String::from("");

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

                    match char::from_u32(u32::from_str_radix(&buffer, 16).unwrap()) {
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

                    let mut buffer = String::from("");

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

                    match char::from_u32(u32::from_str_radix(&buffer, 16).unwrap()) {
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

                    let mut buffer = String::from("");

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

                    match char::from_u32(u32::from_str_radix(&buffer, 16).unwrap()) {
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

    fn eat_char(&mut self) -> IterElem {
        let start_location = self.location;

        self.advance();

        let mut size = 0;

        while self.current != '\'' {
            if self.current == '\n' || self.eof() {
                return Some(Error(LexError::UnterminatedCharLiteral).at(Span::new(
                    start_location,
                    self.location,
                    self.file_id,
                )));
            }

            if self.current == '\\' {
                let e = self.eat_escape();

                if let Err(e) = e {
                    return Some(RawToken::from(*e.unwrap()).at(e.span()));
                }
            } else {
                self.advance();
            }

            size += 1;
        }

        self.advance();

        match size {
            2..=usize::MAX => {
                return Some(Error(LexError::MoreThanOneCharInCharLiteral).at(Span::new(
                    start_location,
                    self.location,
                    self.file_id,
                )));
            }
            0 => {
                return Some(Error(LexError::EmptyCharLiteral).at(Span::new(
                    start_location,
                    self.location,
                    self.file_id,
                )));
            }
            _ => {}
        }

        Some(CharLiteral.at(Span::new(start_location, self.location, self.file_id)))
    }

    fn eat_string(&mut self) -> IterElem {
        let start_location = self.location;

        self.advance();

        let mut buffer = String::from("");

        while !self.eof() && self.current != '\n' {
            let c = self.current;

            if c == '"' {
                break;
            }

            if c == '\\' {
                let e = self.eat_escape();

                if let Err(e) = e {
                    return Some(RawToken::from(*e.unwrap()).at(e.span()));
                } else if let Ok(c) = e {
                    buffer.push(c);
                }
            } else {
                buffer.push(c);
                self.advance();
            }
        }

        if self.eof() || self.current == '\n' {
            return Some(Error(LexError::UnterminatedStringLiteral).at(Span::new(
                start_location,
                self.location,
                self.file_id,
            )));
        }

        self.advance();

        Some(StringLiteral.at(Span::new(start_location, self.location, self.file_id)))
    }

    fn eat_wrapped_id(&mut self) -> IterElem {
        let start_location = self.location;

        self.advance();

        let name = &self.advance_while(start_location, |current, _| {
            current.is_alphanumeric() || current == '_'
        })[1..];

        if self.current != '`' {
            return Some(Error(LexError::UnterminatedWrappedIdentifier).at(Span::new(
                start_location,
                self.location,
                self.file_id,
            )));
        }

        if name.is_empty() {
            return Some(Error(LexError::EmptyWrappedIdentifier).at(Span::new(
                start_location,
                self.location,
                self.file_id,
            )));
        }

        self.advance();

        Some(Identifier(self.interner.get_or_intern(name)).at(Span::new(
            start_location,
            self.location,
            self.file_id,
        )))
    }

    fn eat_comment(&mut self) -> IterElem {
        // first `/` character is already advanced
        let start_location = self.location - 1;
        self.advance();

        self.advance_while(start_location + 2, |current, _| (current != '\n'));

        Some(Comment.at(Span::new(start_location, self.location, self.file_id)))
    }

    /// In this case [`bool`] is true when doc comment is describing
    /// the whole module (3-rd character is `!`) and not when
    /// doc comment is corresponding to trait method, enum variant, etc.
    /// (everything else and the character is `/`).
    fn eat_doc_comment(&mut self, global: bool) -> IterElem {
        // first `/` character is already advanced
        let start_location = self.location - 1;
        self.advance_twice(); // `/` and (`!` or `/`)

        self.advance_while(start_location + 3, |current, _| (current != '\n'));

        Some(
            if global {
                GlobalDocComment
            } else {
                LocalDocComment
            }
            .at(Span::new(start_location, self.location, self.file_id)),
        )
    }

    fn eat_name(&mut self) -> IterElem {
        let start_location = self.location;
        let name = self.advance_while(start_location, |current, _| is_id_continue(current));

        match RESERVED.get(name) {
            Some(reserved) => {
                Some((*reserved).at(Span::new(start_location, self.location, self.file_id)))
            }
            None => Some(Identifier(self.interner.get_or_intern(name)).at(Span::new(
                start_location,
                self.location,
                self.file_id,
            ))),
        }
    }

    fn eat_digits(
        &mut self,
        base: i8,
        invalid_digit_location: &mut Option<usize>,
        digit_separator: &mut i32,
    ) {
        if base <= 10 {
            let max = from_u32('0' as u32 + base as u32).unwrap();

            while number::decimal(self.current) || self.current == '_' {
                let mut ds = 1;

                if self.current == '_' {
                    ds = 2;
                } else if self.current >= max && invalid_digit_location.is_none() {
                    *invalid_digit_location = Some(self.location);
                }

                *digit_separator |= ds;
                self.advance();
            }
        } else {
            while number::hexadecimal(self.current) || self.current == '_' {
                let mut ds = 1;

                if self.current == '_' {
                    ds = 2;
                }

                *digit_separator |= ds;
                self.advance();
            }
        }
    }

    pub fn next_no_comments(&mut self) -> IterElem {
        loop {
            let t = self.next();
            match t {
                Some(ref c) => {
                    if *c.unwrap() != Comment {
                        return t;
                    }
                }
                None => {
                    return t;
                }
            }
        }
    }
}

impl<'c> Iterator for Lexer<'c> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.eat_whitespaces();

        match (self.current, self.next) {
            ('\0', _) => None,

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

                self.advance_with(Error(LexError::UnexpectedChar))
            }
        }
    }
}

/// True if `c` is a whitespace.
fn is_whitespace(c: char) -> bool {
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
