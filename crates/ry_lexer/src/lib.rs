//! This crate provides a lexer for Ry programming language.
//!
//! Lexer is a first stage of compilation, state machine
//! that converts Ry source text into [`Token`]s.
//!
//! Whitespaces are ignored during scanning process.
//!
//! Inherited multiline comments are not supported:
//! This is not valid:
//! ```ry
//!  /* /* test */ */
//! ```
//! While this is:
//! ```ry
//!  /* /* test */
//! ```
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
//! let mut lexer = Lexer::new("", &mut interner);
//!
//! assert_eq!(lexer.next().unwrap().unwrap(), &EndOfFile);
//! assert_eq!(lexer.next().unwrap().unwrap(), &EndOfFile); // ok
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
//! let mut lexer = Lexer::new("#", &mut interner);
//!
//! assert_eq!(lexer.next().unwrap().unwrap(), &Error(LexError::UnexpectedChar));
//! ```

use ry_ast::{
    span::*,
    token::{Punctuator::*, RawToken::*, *},
};
use ry_interner::Interner;
use std::{char::from_u32, str::Chars, string::String};

mod number;
mod tests;

#[derive(Debug)]
pub struct Lexer<'a> {
    pub interner: &'a mut Interner,
    current: char,
    next: char,
    contents: &'a str,
    chars: Chars<'a>,
    location: usize,
}

type IterElem = Option<Token>;

impl<'a> Lexer<'a> {
    pub fn new(contents: &'a str, interner: &'a mut Interner) -> Self {
        let mut chars = contents.chars();

        let current = chars.next().unwrap_or('\0');
        let next = chars.next().unwrap_or('\0');

        Self {
            current,
            next,
            contents,
            chars,
            location: 0,
            interner,
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

    fn char_location(&self, character_len: usize) -> Span {
        (self.location..self.location + character_len).into()
    }

    fn advance_with(&mut self, raw: RawToken) -> IterElem {
        let r = Some(Token::new(raw, self.char_location(1)));
        self.advance();
        r
    }

    fn advance_twice_with(&mut self, raw: RawToken) -> IterElem {
        let r = Some(Token::new(raw, self.char_location(2)));
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

    fn eat_escape(&mut self) -> Result<char, (LexError, Span)> {
        let r = match self.current {
            'b' => Ok('\u{0008}'),
            'f' => Ok('\u{000C}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '\'' => Ok('\''),
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '\0' => Err((LexError::EmptyEscapeSequence, self.char_location(1))),
            'u' => {
                self.advance(); // u

                if self.current != '{' {
                    return Err((
                        LexError::ExpectedOpenBracketInUnicodeEscapeSequence,
                        self.char_location(1),
                    ));
                }

                self.advance(); // '{'

                let mut buffer = String::from("");

                for _ in 0..4 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err((
                            LexError::ExpectedDigitInUnicodeEscapeSequence,
                            self.char_location(1),
                        ));
                    }

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err((
                        LexError::ExpectedCloseBracketInUnicodeEscapeSequence,
                        self.char_location(1),
                    ));
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).unwrap()) {
                    Some(c) => Ok(c),
                    None => Err((
                        LexError::InvalidUnicodeEscapeSequence,
                        (self.location - 4..self.location).into(),
                    )),
                }
            }
            'x' => {
                self.advance(); // x

                if self.current != '{' {
                    return Err((
                        LexError::ExpectedOpenBracketInByteEscapeSequence,
                        self.char_location(1),
                    ));
                }

                self.advance(); // '{'

                let mut buffer = String::from("");

                for _ in 0..2 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err((
                            LexError::ExpectedDigitInByteEscapeSequence,
                            self.char_location(1),
                        ));
                    }

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err((
                        LexError::ExpectedCloseBracketInByteEscapeSequence,
                        self.char_location(1),
                    ));
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).unwrap()) {
                    Some(c) => Ok(c),
                    None => Err((
                        LexError::InvalidByteEscapeSequence,
                        (self.location - 4..self.location).into(),
                    )),
                }
            }
            _ => Err((LexError::UnknownEscapeSequence, self.char_location(1))),
        };

        self.advance();

        r
    }

    fn eat_char(&mut self) -> IterElem {
        let start_location = self.location;

        self.advance(); // `'`

        let mut size = 0;

        let mut result = '\0';

        while self.current != '\'' {
            if self.current == '\\' {
                let e = self.eat_escape();

                if let Err(e) = e {
                    return Some((Error(e.0), e.1).into());
                } else if let Ok(c) = e {
                    result = c;
                }
            } else {
                result = self.current;
            }

            if self.current == '\n' || self.eof() {
                return Some(
                    Error(LexError::UnterminatedCharLiteral)
                        .with_span(start_location..self.location),
                );
            }

            size += 1;

            self.advance(); // c
        }

        self.advance(); // `'`

        match size {
            2..=i32::MAX => {
                return Some(
                    Error(LexError::MoreThanOneCharInCharLiteral)
                        .with_span(start_location..self.location),
                );
            }
            0 => {
                return Some(
                    Error(LexError::EmptyCharLiteral).with_span(start_location..self.location),
                );
            }
            _ => {}
        }

        Some(CharLiteral(result).with_span(start_location..self.location))
    }

    fn eat_string(&mut self) -> IterElem {
        let start_location = self.location;

        self.advance(); // '"'

        let mut buffer = String::from("");

        while !self.eof() && self.current != '\n' {
            let c = self.current;

            if c == '"' {
                break;
            }

            self.advance();

            if c == '\\' {
                let e = self.eat_escape();

                if let Err(e) = e {
                    return Some((Error(e.0), e.1).into());
                } else if let Ok(c) = e {
                    buffer.push(c);
                }
            } else {
                buffer.push(c);
            }
        }

        if self.eof() || self.current == '\n' {
            return Some(
                Error(LexError::UnterminatedStringLiteral).with_span(start_location..self.location),
            );
        }

        self.advance(); // '"'

        Some(
            StringLiteral(self.contents[start_location + 1..self.location - 1].into())
                .with_span(start_location..self.location),
        )
    }

    fn eat_wrapped_id(&mut self) -> IterElem {
        let start_location = self.location;

        self.advance(); // '`'

        let name = &self.advance_while(start_location, |current, _| {
            current.is_alphanumeric() || current == '_'
        })[1..];

        if self.current != '`' {
            return Some(
                Error(LexError::UnterminatedWrappedIdentifier)
                    .with_span(start_location..self.location),
            );
        }

        if name.is_empty() {
            return Some(
                Error(LexError::EmptyWrappedIdentifier).with_span(start_location..self.location),
            );
        }

        self.advance(); // '`'

        Some(Identifier(self.interner.get_or_intern(name)).with_span(start_location..self.location))
    }

    fn eat_comment(&mut self) -> IterElem {
        // first `/` character is already advanced
        let start_location = self.location - 1;
        self.advance(); // `/`

        self.advance_while(start_location + 2, |current, _| (current != '\n'));

        Some(Comment.with_span(start_location..self.location))
    }

    /// In this case [`bool`] is true when docstring is describing
    /// the whole module (3-rd character is `!`) and not when
    /// docstring is corresponding to trait method, enum variant, etc.
    /// (everything else and the character is `/`).
    fn eat_docstring(&mut self, global: bool) -> IterElem {
        // first `/` character is already advanced
        let start_location = self.location - 1;
        self.advance_twice(); // `/` and (`!` or `/`)

        let content = self.advance_while(start_location + 3, |current, _| (current != '\n'));

        Some(
            DocstringComment {
                global,
                content: content.into(),
            }
            .with_span(start_location..self.location),
        )
    }

    fn eat_name(&mut self) -> IterElem {
        let start_location = self.location;
        let name = self.advance_while(start_location, |current, _| is_id_continue(current));

        match RESERVED.get(name) {
            Some(reserved) => Some(reserved.clone().with_span(start_location..self.location)),
            None => Some(
                Identifier(self.interner.get_or_intern(name))
                    .with_span(start_location..self.location),
            ),
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

    pub fn next_no_docstrings_and_comments(&mut self) -> IterElem {
        loop {
            let t = self.next();
            match t.as_ref().unwrap().unwrap() {
                DocstringComment { .. } => {}
                Comment => {}
                _ => {
                    return t;
                }
            }
        }
    }

    pub fn next_no_comments(&mut self) -> IterElem {
        loop {
            let t = self.next();
            match t.as_ref().unwrap().unwrap() {
                Comment => {}
                _ => {
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
            ('\0', _) => Some(Token::new(EndOfFile, self.char_location(1))),

            (':', _) => self.advance_with(Punctuator(Colon)),

            ('@', _) => self.advance_with(Punctuator(AtSign)),

            ('"', _) => self.eat_string(),
            ('\'', _) => self.eat_char(),
            ('`', _) => self.eat_wrapped_id(),

            ('+', '+') => self.advance_twice_with(Punctuator(PlusPlus)),
            ('+', '=') => self.advance_twice_with(Punctuator(PlusEq)),
            ('+', _) => self.advance_with(Punctuator(Plus)),

            ('-', '-') => self.advance_twice_with(Punctuator(MinusMinus)),
            ('-', '=') => self.advance_twice_with(Punctuator(MinusEq)),
            ('-', _) => self.advance_with(Punctuator(Minus)),

            ('*', '*') => self.advance_twice_with(Punctuator(AsteriskAsterisk)),
            ('*', '=') => self.advance_twice_with(Punctuator(AsteriskEq)),
            ('*', _) => self.advance_with(Punctuator(Asterisk)),

            ('/', '/') => {
                self.advance(); // first `/` character

                match self.next {
                    '!' => self.eat_docstring(true),
                    '/' => self.eat_docstring(false),
                    _ => self.eat_comment(),
                }
            }
            ('/', '=') => self.advance_twice_with(Punctuator(SlashEq)),
            ('/', _) => self.advance_with(Punctuator(Slash)),

            ('!', '=') => self.advance_twice_with(Punctuator(NotEq)),
            ('!', _) => self.advance_with(Punctuator(Bang)),

            ('>', '>') => self.advance_twice_with(Punctuator(RightShift)),
            ('>', '=') => self.advance_twice_with(Punctuator(GreaterThanOrEq)),
            ('>', _) => self.advance_with(Punctuator(GreaterThan)),

            ('<', '<') => self.advance_twice_with(Punctuator(LeftShift)),
            ('<', '=') => self.advance_twice_with(Punctuator(LessThanOrEq)),
            ('<', _) => self.advance_with(Punctuator(LessThan)),

            ('=', '=') => self.advance_twice_with(Punctuator(Eq)),
            ('=', _) => self.advance_with(Punctuator(Assign)),

            ('|', '=') => self.advance_twice_with(Punctuator(OrEq)),
            ('|', '|') => self.advance_twice_with(Punctuator(OrOr)),
            ('|', _) => self.advance_with(Punctuator(Or)),

            ('?', ':') => self.advance_twice_with(Punctuator(Elvis)),
            ('?', _) => self.advance_with(Punctuator(QuestionMark)),

            ('&', '&') => self.advance_twice_with(Punctuator(AndAnd)),
            ('&', _) => self.advance_with(Punctuator(And)),

            ('^', '=') => self.advance_twice_with(Punctuator(XorEq)),
            ('^', _) => self.advance_with(Punctuator(Xor)),

            ('~', '=') => self.advance_twice_with(Punctuator(NotEq)),
            ('~', _) => self.advance_with(Punctuator(Not)),

            ('(', _) => self.advance_with(Punctuator(OpenParent)),
            (')', _) => self.advance_with(Punctuator(CloseParent)),

            ('[', _) => self.advance_with(Punctuator(OpenBracket)),
            (']', _) => self.advance_with(Punctuator(CloseBracket)),

            ('{', _) => self.advance_with(Punctuator(OpenBrace)),
            ('}', _) => self.advance_with(Punctuator(CloseBrace)),

            (',', _) => self.advance_with(Punctuator(Comma)),
            (';', _) => self.advance_with(Punctuator(Semicolon)),

            ('%', _) => self.advance_with(Punctuator(Percent)),

            (c, n) => {
                if number::decimal(c) || (c == '.' && number::decimal(n)) {
                    return self.eat_number();
                } else if is_id_start(c) {
                    return self.eat_name();
                } else if c == '.' {
                    return self.advance_with(Punctuator(Dot));
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
