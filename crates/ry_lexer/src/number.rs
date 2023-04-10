use ry_ast::{span::*, token::RawToken::*, token::*};
use std::string::String;

use crate::{IterElem, Lexer};

#[inline]
pub(crate) fn decimal(c: char) -> bool {
    c.is_ascii_digit()
}

#[inline]
pub(crate) fn hexadecimal(c: char) -> bool {
    c.is_ascii_digit() || ('a'..='f').contains(&c.to_ascii_lowercase())
}

fn invalid_separator(buffer: String) -> i32 {
    let mut x1 = ' ';
    let mut d = '.';
    let mut i = 0;

    let bytes = buffer.as_bytes();

    if buffer.len() >= 2 && bytes[0] as char == '0' {
        x1 = bytes[1] as char;
        if x1 == 'x' || x1 == 'o' || x1 == 'b' {
            d = '0';
            i = 2;
        }
    }

    while i < buffer.len() {
        let p = d;
        d = bytes[i] as char;
        if d == '_' {
            if p != '0' {
                return i as i32;
            }
        } else if decimal(d) || x1 == 'x' && hexadecimal(d) {
            d = '0';
        } else {
            if p == '_' {
                return i as i32 - 1;
            }

            d = '.';
        }

        i += 1;
    }

    if d == '_' {
        return bytes.len() as i32 - 1;
    }

    -1
}

impl Lexer<'_> {
    pub(crate) fn eat_number(&mut self) -> IterElem {
        let start_location = self.location;

        let mut number_kind = NumberKind::Invalid;

        let mut base: i8 = 10;
        let mut prefix = '0';
        let mut digit_separator = 0;

        let mut invalid_digit_location: Option<usize> = None;

        if self.current != '.' {
            number_kind = NumberKind::Int;

            if self.current == '0' {
                self.advance();

                match self.current.to_ascii_lowercase() {
                    'x' => {
                        self.advance();
                        base = 16;
                        prefix = 'x';
                    }
                    'o' => {
                        self.advance();
                        base = 8;
                        prefix = 'o';
                    }
                    'b' => {
                        self.advance();
                        base = 2;
                        prefix = 'b';
                    }
                    _ => {
                        base = 10;
                        prefix = '0';
                        digit_separator = 1;
                    }
                }
            }

            self.eat_digits(base, &mut invalid_digit_location, &mut digit_separator);
        }

        // fractional part
        if self.current == '.' {
            number_kind = NumberKind::Float;

            if prefix == 'o' || prefix == 'b' || prefix == 'x' {
                return Some(Error(LexError::InvalidRadixPoint).at(start_location..self.location));
            }

            self.advance();

            self.eat_digits(base, &mut invalid_digit_location, &mut digit_separator);
        }

        if digit_separator & 1 == 0 {
            return Some(Error(LexError::HasNoDigits).at(start_location..self.location));
        }

        let l = self.current.to_ascii_lowercase();
        if l == 'e' {
            if prefix != '\0' && prefix != '0' {
                return Some(
                    Error(LexError::ExponentRequiresDecimalMantissa)
                        .at(start_location..self.location),
                );
            }

            self.advance();

            number_kind = NumberKind::Float;

            if self.current == '+' || self.current == '-' {
                self.advance();
            }

            let mut ds = 0;
            self.eat_digits(10, &mut None, &mut ds);
            digit_separator |= ds;

            if ds & 1 == 0 {
                return Some(
                    Error(LexError::ExponentHasNoDigits).at(start_location..self.location),
                );
            }
        }

        let buffer = &self.contents[start_location..self.location];

        if let Some(location) = invalid_digit_location {
            if number_kind == NumberKind::Int {
                return Some(Token::new(
                    Error(LexError::InvalidDigit),
                    Span::from_location(location, 1),
                ));
            }
        }

        let s = invalid_separator(buffer.to_owned());

        if digit_separator & 2 != 0 && s >= 0 {
            return Some(Token::new(
                Error(LexError::UnderscoreMustSeparateSuccessiveDigits),
                Span::from_location(s as usize + start_location, 1),
            ));
        }

        match number_kind {
            NumberKind::Int => Some(IntegerLiteral.at(start_location..self.location)),
            NumberKind::Float => Some(FloatLiteral.at(start_location..self.location)),
            NumberKind::Invalid => unreachable!(),
        }
    }
}
