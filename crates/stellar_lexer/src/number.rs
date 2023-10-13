//! Implements tokenizing for number literals.

use std::char::from_u32;

use stellar_ast::token::{NumberKind, RawLexError, RawToken, Token};
use stellar_filesystem::location::{ByteOffset, Location};

use crate::{is_id_start, IsAsciiExt, Lexer};

impl Lexer<'_> {
    /// Tokenizes a number literal token.
    pub(crate) fn tokenize_number(&mut self) -> Token {
        let start_offset = self.offset;

        // If the number is an integer or a float.
        let mut number_kind = NumberKind::Invalid;

        // Base of the number.
        let mut base = 10;

        // 0b010
        //  ^ prefix = 'b'
        //
        // 0x9f
        //  ^ prefix = 'x'
        //
        // 3822
        // ^^^^ prefix = '0' for decimals
        let mut prefix = '0';

        // bit 0: digit present, bit 1: `_` present.
        let mut digit_separator = 0;

        // Location of the first invalid digit.
        let mut invalid_digit_location = None;

        if self.current != Some('.') {
            number_kind = NumberKind::Int;

            if self.current == Some('0') {
                self.advance();

                match self.current.map(|c| c.to_ascii_lowercase()) {
                    Some('x') => {
                        self.advance();
                        base = 16;
                        prefix = 'x';
                    }
                    Some('o') => {
                        self.advance();
                        base = 8;
                        prefix = 'o';
                    }
                    Some('b') => {
                        self.advance();
                        base = 2;
                        prefix = 'b';
                    }
                    _ => {
                        base = 8;
                        prefix = '0';
                        digit_separator = 1; // leading 0
                    }
                }
            }

            self.eat_digits(base, &mut invalid_digit_location, &mut digit_separator);
        }

        'processing_float: {
            if self.current == Some('.') {
                // 1.to_string() is parsed as:
                // Int(1) Punct(Dot) Ident Punct(Lparen) ...
                if is_id_start(self.next) {
                    break 'processing_float;
                }

                number_kind = NumberKind::Float;

                self.advance();

                if prefix == 'o' || prefix == 'b' || prefix == 'x' {
                    return Token {
                        raw: RawToken::Error(RawLexError::InvalidRadixPoint),
                        location: self.location_from(start_offset),
                    };
                }

                self.eat_digits(base, &mut invalid_digit_location, &mut digit_separator);
            }

            if digit_separator & 1 == 0 {
                return Token {
                    raw: RawToken::Error(RawLexError::NumberContainsNoDigits),
                    location: self.location_from(start_offset),
                };
            }
        }

        if self.current.map(|c| c.to_ascii_lowercase()) == Some('e') {
            if prefix != '\0' && prefix != '0' {
                return Token {
                    raw: RawToken::Error(RawLexError::ExponentRequiresDecimalMantissa),
                    location: self.location_from(start_offset),
                };
            }

            self.advance();

            number_kind = NumberKind::Float;

            if self.current == Some('+') || self.current == Some('-') {
                self.advance();
            }

            let mut ds = 0;
            self.eat_digits(10, &mut None, &mut ds);
            digit_separator |= ds;

            if ds & 1 == 0 {
                return Token {
                    raw: RawToken::Error(RawLexError::ExponentHasNoDigits),
                    location: self.location_from(start_offset),
                };
            }
        }

        let number_string = &self.source[start_offset.0..self.offset.0];

        if let Some(invalid_digit_offset) = invalid_digit_location {
            if number_kind == NumberKind::Int {
                return Token {
                    raw: RawToken::Error(RawLexError::InvalidDigit),
                    location: invalid_digit_offset.next_byte_location_at(self.filepath),
                };
            }
        }

        if let Some(location) = self.check_for_invalid_separator(start_offset, number_string) {
            if digit_separator & 2 != 0 {
                return Token {
                    raw: RawToken::Error(RawLexError::UnderscoreMustSeparateSuccessiveDigits),
                    location,
                };
            }
        }

        match number_kind {
            NumberKind::Int => Token {
                raw: RawToken::IntegerLiteral,
                location: self.location_from(start_offset),
            },
            NumberKind::Float => Token {
                raw: RawToken::FloatLiteral,
                location: self.location_from(start_offset),
            },
            NumberKind::Invalid => {
                unreachable!()
            }
        }
    }

    /// Checks if the number has an invalid `_` separator in it (if it doesn't separate
    /// successive digits, the function returns the location of the separator).
    fn check_for_invalid_separator(
        &self,
        start_offset: ByteOffset,
        number_string: &str,
    ) -> Option<Location> {
        let mut idx = 1;
        let chars_slice = &number_string.chars().collect::<Vec<char>>();

        for window in chars_slice.windows(3) {
            let (previous, current, next) = (window[0], window[1], window[2]);

            if current != '_' || (next.is_ascii_hexdigit() && previous.is_ascii_hexdigit()) {
                idx += 1;
                continue;
            }

            return Some(self.make_location(start_offset + idx, start_offset + idx + 1));
        }

        if *chars_slice.last().unwrap() == '_' {
            return Some(self.make_location(start_offset + idx, start_offset + idx + 1));
        }

        None
    }

    /// Processes a group of digits.
    fn eat_digits(
        &mut self,
        base: u8,
        invalid_digit_offset: &mut Option<ByteOffset>,
        digit_separator: &mut i32,
    ) {
        #[inline]
        fn set_if_none<T>(option: &mut Option<T>, value: T) {
            if option.is_none() {
                *option = Some(value);
            }
        }

        if base <= 10 {
            let max = from_u32('0' as u32 + u32::from(base)).unwrap();

            while self.current.is_ascii_digit() || self.current == Some('_') {
                let mut digit_separator_ = 1;

                if self.current == Some('_') {
                    digit_separator_ = 2;
                } else if self.current >= Some(max) && invalid_digit_offset.is_none() {
                    set_if_none(invalid_digit_offset, self.offset);
                }

                *digit_separator |= digit_separator_;
                self.advance();
            }
        } else {
            while self.current.is_ascii_hexdigit() || self.current == Some('_') {
                *digit_separator |= if self.current == Some('_') { 2 } else { 1 };
                self.advance();
            }
        }
    }
}
