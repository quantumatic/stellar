//! `lib.rs` - implements lexer.
//!
//! Lexer is a part of parser (first stage of compilation), state machine
//! that converts Ry source text into [`Token`]s.
//!
//! Whitespaces are ignored during scanning process.
//!
//! Inherited multiline comments are not supported:
//! This is not valid:
//! ```ry
//!  /* /* test */ */
//! ```
//!
//! Lexer is fairly standart. It implements `Iterator<Item = Token>` on each step,
//! and stops at eof (always returns [`EndOfFile`] when it's already eof).
//! ```
//! use ry_lexer::Lexer;
//! use ry_ast::token::RawToken;
//! use string_interner::StringInterner;
//!
//! let mut string_interner = StringInterner::default();
//! let mut lexer = Lexer::new("", &mut string_interner);
//!
//! assert_eq!(lexer.next().unwrap().value, EndOfFile);
//! assert_eq!(lexer.next().unwrap().value, EndOfFile); // ok
//! ```
//!
//! Note: the Ry lexer makes use of the `string_interner` crate to perform string interning,
//! a process of deduplicating strings, which can be highly beneficial when dealing with
//! identifiers.
//!
//! If error appeared in the process, [`Invalid`] will be returned:
//!
//! ```
//! use ry_lexer::Lexer;
//! use ry_ast::token::{RawToken, LexerError};
//! use string_interner::StringInterner;
//!
//! let mut string_interner = StringInterner::default();
//! let mut lexer = Lexer::new("#", &mut string_interner);
//!
//! assert_eq!(lexer.next().unwrap().value, Invalid(LexerError::UnexpectedChar('#')));
//! ```

use std::{string::String, str::Chars, char::from_u32};
use ry_ast::{location::*, token::RawToken::*, token::*};

use string_interner::StringInterner;

mod number;
mod tests;

pub struct Lexer<'a> {
    pub identifier_interner: &'a mut StringInterner,
    current: char,
    next: char,
    contents: &'a str,
    chars: Chars<'a>,
    location: usize,
    start_location: usize,
}

type IterElem = Option<Token>;

impl<'a> Lexer<'a> {
    pub fn new(contents: &'a str, identifier_interner: &'a mut StringInterner) -> Self {
        let mut chars = contents.chars();

        let current = chars.next().unwrap_or('\0');
        let next = chars.next().unwrap_or('\0');

        Self {
            current,
            next,
            contents,
            chars,
            location: 0,
            start_location: 0,
            identifier_interner,
        }
    }

    fn eof(&self) -> bool {
        self.current == '\0'
    }

    fn skip_over_whitespaces(&mut self) {
        while self.current.is_whitespace() {
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

    fn advance_while<F>(&mut self, mut f: F) -> &'a str
    where
        F: FnMut(char, char) -> bool,
    {
        while f(self.current, self.next) && !self.eof() {
            self.advance();
        }

        &self.contents[self.start_location..self.location]
    }

    fn span_from_start(&self) -> Span {
        (self.start_location..self.location).into()
    }

    fn scan_escape(&mut self) -> Result<char, (LexerError, Span)> {
        let r = match self.current {
            'b' => Ok('\u{0008}'),
            'f' => Ok('\u{000C}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '\'' => Ok('\''),
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '\0' => Err((LexerError::EmptyEscapeSequence, self.char_location(1))),
            'u' => {
                self.advance(); // u

                if self.current != '{' {
                    return Err((
                        LexerError::ExpectedOpenBracketInUnicodeEscapeSequence,
                        self.char_location(1),
                    ));
                }

                self.advance(); // '{'

                let mut buffer = String::from("");

                for _ in 0..4 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err((
                            LexerError::ExpectedDigitInUnicodeEscapeSequence,
                            self.char_location(1),
                        ));
                    }

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err((
                        LexerError::ExpectedCloseBracketInUnicodeEscapeSequence,
                        self.char_location(1),
                    ));
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).unwrap()) {
                    Some(c) => Ok(c),
                    None => Err((
                        LexerError::InvalidUnicodeEscapeSequence,
                        (self.location - 4..self.location).into(),
                    )),
                }
            }
            'x' => {
                self.advance(); // x

                if self.current != '{' {
                    return Err((
                        LexerError::ExpectedOpenBracketInByteEscapeSequence,
                        self.char_location(1),
                    ));
                }

                self.advance(); // '{'

                let mut buffer = String::from("");

                for _ in 0..2 {
                    if !self.current.is_ascii_hexdigit() {
                        return Err((
                            LexerError::ExpectedDigitInByteEscapeSequence,
                            self.char_location(1),
                        ));
                    }

                    buffer.push(self.current);
                    self.advance();
                }

                if self.current != '}' {
                    return Err((
                        LexerError::ExpectedCloseBracketInByteEscapeSequence,
                        self.char_location(1),
                    ));
                }

                match char::from_u32(u32::from_str_radix(&buffer, 16).unwrap()) {
                    Some(c) => Ok(c),
                    None => Err((
                        LexerError::InvalidByteEscapeSequence,
                        (self.location - 4..self.location).into(),
                    )),
                }
            }
            _ => Err((LexerError::UnknownEscapeSequence, self.char_location(1))),
        };

        self.advance();

        r
    }

    fn scan_char(&mut self) -> IterElem {
        self.start_location = self.location;

        self.advance(); // `'`

        let mut size = 0;

        let mut result = '\0';

        while self.current != '\'' {
            if self.current == '\\' {
                self.start_location = self.location;

                let e = self.scan_escape();

                if let Err(e) = e {
                    return Some((Invalid(e.0), e.1).into());
                } else if let Ok(c) = e {
                    result = c;
                }
            } else {
                result = self.current;
            }

            if self.current == '\n' || self.eof() {
                return Some(
                    (
                        Invalid(LexerError::UnterminatedCharLiteral),
                        self.span_from_start(),
                    )
                        .into(),
                );
            }

            size += 1;

            self.advance(); // c
        }

        self.advance(); // `'`

        match size {
            2..=i32::MAX => {
                return Some(
                    (
                        Invalid(LexerError::MoreThanOneCharInCharLiteral),
                        self.span_from_start(),
                    )
                        .into(),
                );
            }
            0 => {
                return Some(
                    (
                        Invalid(LexerError::EmptyCharLiteral),
                        self.span_from_start(),
                    )
                        .into(),
                );
            }
            _ => {}
        }

        Some((Char(result), self.span_from_start()).into())
    }

    fn scan_string(&mut self) -> IterElem {
        self.start_location = self.location;

        self.advance(); // '"'

        let mut buffer = String::from("");

        while !self.eof() && self.current != '\n' {
            let c = self.current;

            if c == '"' {
                break;
            }

            self.advance();

            if c == '\\' {
                self.start_location = self.location;

                let e = self.scan_escape();

                if let Err(e) = e {
                    return Some((Invalid(e.0), e.1).into());
                } else if let Ok(c) = e {
                    buffer.push(c);
                }
            } else {
                buffer.push(c);
            }
        }

        if self.eof() || self.current == '\n' {
            return Some(Token::new(
                Invalid(LexerError::UnterminatedStringLiteral),
                self.span_from_start(),
            ));
        }

        self.advance(); // '"'

        Some(Token::new(String(buffer), self.span_from_start()))
    }

    fn scan_wrapped_id(&mut self) -> IterElem {
        self.start_location = self.location;

        self.advance(); // '`'

        let name =
            &self.advance_while(|current, _| current.is_alphanumeric() || current == '_')[1..];

        if self.current != '`' {
            return Some(Token::new(
                Invalid(LexerError::UnterminatedWrappedIdentifierLiteral),
                self.span_from_start(),
            ));
        }

        if name.is_empty() {
            return Some(Token::new(
                Invalid(LexerError::EmptyWrappedIdentifierLiteral),
                self.span_from_start(),
            ));
        }

        self.advance(); // '`'

        Some(Token::new(
            Identifier(self.identifier_interner.get_or_intern(name)),
            self.span_from_start(),
        ))
    }

    fn scan_single_line_comment(&mut self) -> IterElem {
        self.start_location = self.location;

        self.advance_twice(); // '//'

        let content = self.advance_while(|current, _| (current != '\n'));

        Some(Token::new(
            Comment(content[2..].replace('\r', "")),
            self.span_from_start(),
        ))
    }

    fn scan_name(&mut self) -> IterElem {
        self.start_location = self.location;
        let name = self.advance_while(|current, _| current.is_alphanumeric() || current == '_');

        match RESERVED.get(name) {
            Some(reserved) => Some(Token::new(reserved.clone(), self.span_from_start())),
            None => Some(Token::new(
                Identifier(self.identifier_interner.get_or_intern(name)),
                self.span_from_start(),
            )),
        }
    }

    fn scan_digits(
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
            match t.as_ref().unwrap().value {
                Comment(_) => {}
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
        if self.current.is_whitespace() {
            self.skip_over_whitespaces();
        }

        match (self.current, self.next) {
            ('\0', _) => Some(Token::new(EndOfFile, self.char_location(1))),

            (':', ':') => self.advance_twice_with(DoubleColon),
            (':', _) => self.advance_with(Colon),

            ('@', _) => self.advance_with(AtSign),

            ('"', _) => self.scan_string(),
            ('\'', _) => self.scan_char(),
            ('`', _) => self.scan_wrapped_id(),

            ('+', '+') => self.advance_twice_with(PlusPlus),
            ('+', '=') => self.advance_twice_with(PlusEq),
            ('+', _) => self.advance_with(Plus),

            ('-', '-') => self.advance_twice_with(MinusMinus),
            ('-', '=') => self.advance_twice_with(MinusEq),
            ('-', _) => self.advance_with(Minus),

            ('*', '*') => self.advance_twice_with(AsteriskAsterisk),
            ('*', '=') => self.advance_twice_with(AsteriskEq),
            ('*', _) => self.advance_with(Asterisk),

            ('/', '/') => self.scan_single_line_comment(),
            ('/', '=') => self.advance_twice_with(SlashEq),
            ('/', _) => self.advance_with(Slash),

            ('!', '=') => self.advance_twice_with(NotEq),
            ('!', '!') => self.advance_twice_with(BangBang),
            ('!', _) => self.advance_with(Bang),

            ('>', '>') => self.advance_twice_with(RightShift),
            ('>', '=') => self.advance_twice_with(GreaterThanOrEq),
            ('>', _) => self.advance_with(GreaterThan),

            ('<', '<') => self.advance_twice_with(LeftShift),
            ('<', '=') => self.advance_twice_with(LessThanOrEq),
            ('<', _) => self.advance_with(LessThan),

            ('=', '=') => self.advance_twice_with(Eq),
            ('=', _) => self.advance_with(Assign),

            ('|', '=') => self.advance_twice_with(OrEq),
            ('|', '|') => self.advance_twice_with(OrOr),
            ('|', _) => self.advance_with(Or),

            ('?', ':') => self.advance_twice_with(Elvis),
            ('?', _) => self.advance_with(QuestionMark),

            ('&', '&') => self.advance_twice_with(AndAnd),
            ('&', _) => self.advance_with(And),

            ('^', '=') => self.advance_twice_with(XorEq),
            ('^', _) => self.advance_with(Xor),

            ('~', '=') => self.advance_twice_with(NotEq),
            ('~', _) => self.advance_with(Not),

            ('(', _) => self.advance_with(OpenParent),
            (')', _) => self.advance_with(CloseParent),

            ('[', _) => self.advance_with(OpenBracket),
            (']', _) => self.advance_with(CloseBracket),

            ('$', _) => self.advance_with(Dollar),

            ('{', _) => self.advance_with(OpenBrace),
            ('}', _) => self.advance_with(CloseBrace),

            (',', _) => self.advance_with(Comma),
            (';', _) => self.advance_with(Semicolon),

            ('%', _) => self.advance_with(Percent),

            (c, n) => {
                if number::decimal(c) || (c == '.' && number::decimal(n)) {
                    return self.scan_number();
                } else if c.is_alphanumeric() || c == '_' {
                    return self.scan_name();
                } else if c == '.' {
                    return self.advance_with(Dot);
                }

                self.advance_with(Invalid(LexerError::UnexpectedChar(c)))
            }
        }
    }
}
