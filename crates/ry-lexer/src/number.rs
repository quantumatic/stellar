use ry_ast::{location::*, token::RawToken::*, token::*};
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

#[inline]
pub(crate) fn to_digit2(c: char) -> i8 {
    let l = c.to_ascii_lowercase();
    match l {
        '0'..='9' => l as i8 - '0' as i8,
        'a'..='f' => l as i8 - 'a' as i8 + 10,
        '_' => 0,
        _ => 16,
    }
}

fn parse_integer(buffer: &[u8], base: i8) -> Option<u64> {
    let mut n = 0;
    let mut pow = 1;

    for i in 0..buffer.len() {
        n += (to_digit2(buffer[buffer.len() - 1 - i] as char) as u64).checked_mul(pow)?;
        pow = pow.checked_mul(base as u64)?;
    }

    Some(n)
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
    pub(crate) fn scan_number(&mut self) -> IterElem {
        self.start_location = self.location;

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

            self.scan_digits(base, &mut invalid_digit_location, &mut digit_separator);
        }

        // fractional part
        if self.current == '.' {
            number_kind = NumberKind::Float;

            if prefix == 'o' || prefix == 'b' || prefix == 'x' {
                return Some(Token::new(
                    Invalid(LexerError::InvalidRadixPoint),
                    self.span_from_start(),
                ));
            }

            self.advance();

            self.scan_digits(base, &mut invalid_digit_location, &mut digit_separator);
        }

        if digit_separator & 1 == 0 {
            return Some(Token::new(
                Invalid(LexerError::HasNoDigits),
                self.span_from_start(),
            ));
        }

        let l = self.current.to_ascii_lowercase();
        if l == 'e' {
            if prefix != '\0' && prefix != '0' {
                return Some(Token::new(
                    Invalid(LexerError::ExponentRequiresDecimalMantissa),
                    self.span_from_start(),
                ));
            }

            self.advance();

            number_kind = NumberKind::Float;

            if self.current == '+' || self.current == '-' {
                self.advance();
            }

            let mut ds = 0;
            self.scan_digits(10, &mut None, &mut ds);
            digit_separator |= ds;

            if ds & 1 == 0 {
                return Some(Token::new(
                    Invalid(LexerError::ExponentHasNoDigits),
                    self.span_from_start(),
                ));
            }
        }

        if self.current == 'i' {
            number_kind = NumberKind::Imag;
            self.advance();
        }

        let buffer = &self.contents[self.start_location..self.location];

        if let Some(location) = invalid_digit_location {
            if number_kind == NumberKind::Int {
                return Some(Token::new(
                    Invalid(LexerError::InvalidDigit),
                    Span::from_location(location, 1),
                ));
            }
        }

        let s = invalid_separator(buffer.to_owned());

        if digit_separator & 2 != 0 && s >= 0 {
            return Some(Token::new(
                Invalid(LexerError::UnderscoreMustSeperateSuccessiveDigits),
                Span::from_location(s as usize + self.start_location, 1),
            ));
        }

        match number_kind {
            NumberKind::Int => {
                match parse_integer(
                    (if base == 10 { buffer } else { &buffer[2..] }).as_bytes(),
                    base,
                ) {
                    Some(n) => Some(Token::new(Int(n), self.span_from_start())),
                    None => Some(Token::new(
                        Invalid(LexerError::NumberParserError),
                        self.span_from_start(),
                    )),
                }
            }
            NumberKind::Float => Some(Token::new(
                Float(match buffer.parse::<f64>() {
                    Ok(n) => n,
                    Err(_) => {
                        return Some(Token::new(
                            Invalid(LexerError::NumberParserError),
                            self.span_from_start(),
                        ));
                    }
                }),
                self.span_from_start(),
            )),
            NumberKind::Imag => Some(Token::new(
                Imag(match buffer[..buffer.len() - 1].parse::<f64>() {
                    Ok(n) => n,
                    Err(_) => {
                        return Some(Token::new(
                            Invalid(LexerError::NumberParserError),
                            self.span_from_start(),
                        ));
                    }
                }),
                self.span_from_start(),
            )),
            NumberKind::Invalid => unimplemented!(),
        }
    }
}
