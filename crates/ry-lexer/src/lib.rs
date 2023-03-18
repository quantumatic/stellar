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
//! and stops at eof (always returns [`RawToken::EndOfFile`] when it's already eof).
//! ```
//! use ry_lexer::Lexer;
//! use ry_ast::token::RawToken;
//!
//! let mut lexer = Lexer::new("");
//!
//! assert_eq!(lexer.next().unwrap().value, RawToken::EndOfFile);
//! assert_eq!(lexer.next().unwrap().value, RawToken::EndOfFile); // ok
//! ```
//!
//! If error appeared in the process, [`RawToken::Invalid`] will be returned:
//!
//! ```
//! use ry_lexer::Lexer;
//! use ry_ast::token::{RawToken, LexerError};
//!
//! let mut lexer = Lexer::new("#");
//!
//! assert_eq!(lexer.next().unwrap().value, RawToken::Invalid(LexerError::UnexpectedChar('#')));
//! ```
//!
//! Lexer doesn't emit diagnostics in the process.

use ry_ast::location::*;
use ry_ast::token::*;

use std::char::from_u32;
use std::str::Chars;

mod number;
mod tests;

pub struct Lexer<'c> {
    current: char,
    next: char,
    contents: &'c str,
    chars: Chars<'c>,
    location: usize,
    start_location: usize,
}

type IterElem = Option<Token>;

impl<'c> Lexer<'c> {
    pub fn new(contents: &'c str) -> Self {
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

    fn advance_while<F>(&mut self, mut f: F) -> &'c str
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
                    return Some((RawToken::Invalid(e.0), e.1).into());
                } else if let Ok(c) = e {
                    result = c;
                }
            } else {
                result = self.current;
            }

            if self.current == '\n' || self.eof() {
                return Some(
                    (
                        RawToken::Invalid(LexerError::UnterminatedCharLiteral),
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
                        RawToken::Invalid(LexerError::MoreThanOneCharInCharLiteral),
                        self.span_from_start(),
                    )
                        .into(),
                );
            }
            0 => {
                return Some(
                    (
                        RawToken::Invalid(LexerError::EmptyCharLiteral),
                        self.span_from_start(),
                    )
                        .into(),
                );
            }
            _ => {}
        }

        Some((RawToken::Char(result), self.span_from_start()).into())
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
                    return Some((RawToken::Invalid(e.0), e.1).into());
                } else if let Ok(c) = e {
                    buffer.push(c);
                }
            } else {
                buffer.push(c);
            }
        }

        if self.eof() || self.current == '\n' {
            return Some(Token::new(
                RawToken::Invalid(LexerError::UnterminatedStringLiteral),
                self.span_from_start(),
            ));
        }

        self.advance(); // '"'

        Some(Token::new(RawToken::String(buffer), self.span_from_start()))
    }

    fn scan_wrapped_id(&mut self) -> IterElem {
        self.start_location = self.location;

        self.advance(); // '`'

        let name =
            &self.advance_while(|current, _| current.is_alphanumeric() || current == '_')[1..];

        if self.current != '`' {
            return Some(Token::new(
                RawToken::Invalid(LexerError::UnterminatedWrappedIdentifierLiteral),
                self.span_from_start(),
            ));
        }

        if name.is_empty() {
            return Some(Token::new(
                RawToken::Invalid(LexerError::EmptyWrappedIdentifierLiteral),
                self.span_from_start(),
            ));
        }

        self.advance(); // '`'

        Some(Token::new(
            RawToken::Identifier(name.to_owned()),
            self.span_from_start(),
        ))
    }

    fn scan_single_line_comment(&mut self) -> IterElem {
        self.start_location = self.location;

        self.advance_twice(); // '//'

        let content = self.advance_while(|current, _| (current != '\n'));

        Some(Token::new(
            RawToken::Comment(content[2..].replace('\r', "")),
            self.span_from_start(),
        ))
    }

    fn scan_name(&mut self) -> IterElem {
        self.start_location = self.location;
        let name = self.advance_while(|current, _| current.is_alphanumeric() || current == '_');

        match RESERVED.get(name) {
            Some(reserved) => Some(Token::new(reserved.clone(), self.span_from_start())),
            None => Some(Token::new(
                RawToken::Identifier(name.to_owned()),
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
                RawToken::Comment(_) => {}
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
            ('\0', _) => Some(Token::new(RawToken::EndOfFile, self.char_location(1))),

            (':', ':') => self.advance_twice_with(RawToken::DoubleColon),
            (':', _) => self.advance_with(RawToken::Colon),

            ('@', _) => self.advance_with(RawToken::AtSign),

            ('"', _) => self.scan_string(),
            ('\'', _) => self.scan_char(),
            ('`', _) => self.scan_wrapped_id(),

            ('+', '+') => self.advance_twice_with(RawToken::PlusPlus),
            ('+', '=') => self.advance_twice_with(RawToken::PlusEq),
            ('+', _) => self.advance_with(RawToken::Plus),

            ('-', '-') => self.advance_twice_with(RawToken::MinusMinus),
            ('-', '=') => self.advance_twice_with(RawToken::MinusEq),
            ('-', _) => self.advance_with(RawToken::Minus),

            ('*', '*') => self.advance_twice_with(RawToken::AsteriskAsterisk),
            ('*', '=') => self.advance_twice_with(RawToken::AsteriskEq),
            ('*', _) => self.advance_with(RawToken::Asterisk),

            ('/', '/') => self.scan_single_line_comment(),
            ('/', '=') => self.advance_twice_with(RawToken::SlashEq),
            ('/', _) => self.advance_with(RawToken::Slash),

            ('!', '=') => self.advance_twice_with(RawToken::NotEq),
            ('!', '!') => self.advance_twice_with(RawToken::BangBang),
            ('!', _) => self.advance_with(RawToken::Bang),

            ('>', '>') => self.advance_twice_with(RawToken::RightShift),
            ('>', '=') => self.advance_twice_with(RawToken::GreaterThanOrEq),
            ('>', _) => self.advance_with(RawToken::GreaterThan),

            ('<', '<') => self.advance_twice_with(RawToken::LeftShift),
            ('<', '=') => self.advance_twice_with(RawToken::LessThanOrEq),
            ('<', _) => self.advance_with(RawToken::LessThan),

            ('=', '=') => self.advance_twice_with(RawToken::Eq),
            ('=', _) => self.advance_with(RawToken::Assign),

            ('|', '=') => self.advance_twice_with(RawToken::OrEq),
            ('|', '|') => self.advance_twice_with(RawToken::OrOr),
            ('|', _) => self.advance_with(RawToken::Or),

            ('?', ':') => self.advance_twice_with(RawToken::Elvis),
            ('?', _) => self.advance_with(RawToken::QuestionMark),

            ('&', '&') => self.advance_twice_with(RawToken::AndAnd),
            ('&', _) => self.advance_with(RawToken::And),

            ('^', '=') => self.advance_twice_with(RawToken::XorEq),
            ('^', _) => self.advance_with(RawToken::Xor),

            ('~', '=') => self.advance_twice_with(RawToken::NotEq),
            ('~', _) => self.advance_with(RawToken::Not),

            ('(', _) => self.advance_with(RawToken::OpenParent),
            (')', _) => self.advance_with(RawToken::CloseParent),

            ('[', _) => self.advance_with(RawToken::OpenBracket),
            (']', _) => self.advance_with(RawToken::CloseBracket),

            ('$', _) => self.advance_with(RawToken::Dollar),

            ('{', _) => self.advance_with(RawToken::OpenBrace),
            ('}', _) => self.advance_with(RawToken::CloseBrace),

            (',', _) => self.advance_with(RawToken::Comma),
            (';', _) => self.advance_with(RawToken::Semicolon),

            ('%', _) => self.advance_with(RawToken::Percent),

            (c, n) => {
                if number::decimal(c) || (c == '.' && number::decimal(n)) {
                    return self.scan_number();
                } else if c.is_alphanumeric() || c == '_' {
                    return self.scan_name();
                } else if c == '.' {
                    return self.advance_with(RawToken::Dot);
                }

                self.advance_with(RawToken::Invalid(LexerError::UnexpectedChar(c)))
            }
        }
    }
}
