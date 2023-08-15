use std::io::{self, Write};

use termion::color;

use crate::visit::Visitor;

struct Printer<W>
where
    W: Write,
{
    ident: usize,
    writer: W,
}

impl<W> Printer<W>
where
    W: Write,
{
    #[inline]
    #[must_use]
    pub const fn new(writer: W) -> Self {
        Self { ident: 0, writer }
    }
}

impl<W> Printer<W>
where
    W: Write,
{
    const NUMBER_COLOR: color::Cyan = color::Cyan;
    const TEXT_COLOR: color::LightWhite = color::LightWhite;
    const KEYWORD_COLOR: color::Magenta = color::Magenta;
    const BOOLEAN_COLOR: color::Yellow = color::Yellow;

    #[inline]
    fn add_whitespace(&mut self) -> Result<(), io::Error> {
        self.writer.write_fmt(format_args!(" "))
    }

    #[inline]
    fn add_newline(&mut self) -> Result<(), io::Error> {
        self.writer.write_fmt(format_args!("\n"))
    }

    #[inline]
    fn add_keyword(&mut self, keyword: &str) -> Result<(), io::Error> {
        self.writer
            .write_fmt(format_args!("{}{} ", Self::KEYWORD_COLOR, keyword))
    }

    #[inline]
    fn add_text(&mut self, text: &str) -> Result<(), io::Error> {
        self.writer
            .write_fmt(format_args!("{}{} ", Self::TEXT_COLOR, text))
    }

    #[inline]
    fn add_boolean(&mut self, boolean: bool) -> Result<(), io::Error> {
        self.writer.write_fmt(format_args!(
            "{}{}",
            Self::BOOLEAN_COLOR,
            boolean.to_string()
        ))
    }

    #[inline]
    fn add_number(&mut self, number: f64) -> Result<(), io::Error> {
        self.writer
            .write_fmt(format_args!("{}{}", Self::NUMBER_COLOR, number))
    }
}

impl<W> Visitor for Printer<W> where W: Write {
    // TODO
}
