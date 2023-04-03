//! This crate provides a parser for Ry programming language
//!
//! It uses the lexer from the ry_lexer crate to tokenize the input source
//! code and produces an Abstract Syntax Tree (AST) that represents the parsed code.
use error::ParserError;
use ry_ast::{
    declaration::{Docstring, WithDocstringable},
    span::WithSpan,
    token::{RawToken::*, Token},
    *,
};
use ry_lexer::Lexer;
use ry_interner::Interner;

pub mod error;

mod r#enum;
mod expression;
mod function_decl;
mod r#impl;
mod imports;
mod statement;
mod struct_decl;
mod trait_decl;
mod r#type;

#[macro_use]
mod macros;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    next: Token,
}

pub type ParserResult<T> = Result<T, ParserError>;

impl<'a> Parser<'a> {
    pub fn new(contents: &'a str, interner: &'a mut Interner) -> Self {
        let mut lexer = Lexer::new(contents, interner);

        let current = lexer.next().unwrap();
        let next = current.clone();

        Self {
            lexer,
            current,
            next,
        }
    }

    /// Checks if the current token being parsed is invalid, and returns
    /// an error if so.
    fn check_scanning_error_for_current_token(&mut self) -> ParserResult<()> {
        if let Invalid(e) = self.current.unwrap() {
            Err(ParserError::ErrorToken((*e).with_span(self.current.span())))
        } else {
            Ok(())
        }
    }

    /// Checks if the next token being parsed is invalid, and returns
    /// an error if so.
    fn check_scanning_error_for_next_token(&mut self) -> ParserResult<()> {
        if let Invalid(e) = self.next.unwrap() {
            Err(ParserError::ErrorToken((*e).with_span(self.next.span())))
        } else {
            Ok(())
        }
    }

    /// Advances the parser to the next token while it is not [`DocstringComment`].
    fn advance(&mut self) -> ParserResult<()> {
        self.check_scanning_error_for_next_token()?;

        self.current = self.next.clone();

        self.next = self.lexer.next_no_docstrings_and_comments().unwrap();

        Ok(())
    }

    /// Advances the parser to the next token.
    fn advance_with_docstring(&mut self) -> ParserResult<()> {
        self.check_scanning_error_for_next_token()?;

        self.current = self.next.clone();

        self.next = self.lexer.next_no_comments().unwrap();

        Ok(())
    }

    /// Consumes the docstrings for the module and the first item in the module, if present.
    pub(crate) fn consume_module_and_first_item_docstrings(
        &mut self,
    ) -> ParserResult<(Docstring, Docstring)> {
        let (mut module_docstring, mut local_docstring) = (vec![], vec![]);

        loop {
            if let DocstringComment { global, content } = self.next.unwrap() {
                if *global {
                    module_docstring.push(content.clone());
                } else {
                    local_docstring.push(content.clone());
                }
            } else {
                return Ok((module_docstring, local_docstring));
            }

            self.advance_with_docstring()?;
        }
    }

    /// Consumes the docstring for a local item (i.e., anything that is not the module docstring
    /// or the first item in the module (because it will be already consumed in
    /// [`Parser::consume_module_and_first_item_docstrings()`])).
    pub(crate) fn consume_non_module_docstring(&mut self) -> ParserResult<Docstring> {
        let mut result = vec![];

        loop {
            if let DocstringComment { global, content } = self.next.unwrap() {
                if !global {
                    result.push(content.clone());
                }
            } else {
                return Ok(result);
            }

            self.advance_with_docstring()?;
        }
    }

    /// Returns [`ParserResult<ProgramUnit>`] where [`ProgramUnit`] represents
    /// AST for a Ry module.
    /// ```
    /// use ry_parser::Parser;
    /// use ry_interner::Interner;
    ///
    /// let mut interner = Interner::default();
    /// let mut parser = Parser::new("fun test() {}", &mut interner);
    /// assert!(parser.parse().is_ok());
    /// ```
    pub fn parse(&mut self) -> ParserResult<ProgramUnit> {
        self.check_scanning_error_for_current_token()?;

        let (module_docstring, fst_docstring) = self.consume_module_and_first_item_docstrings()?;
        Ok(ProgramUnit::new(
            module_docstring,
            self.parse_items(fst_docstring)?,
        ))
    }

    fn parse_items(&mut self, mut local_docstring: Docstring) -> ParserResult<Items> {
        let mut items = vec![];

        loop {
            items.push(
                match self.next.unwrap() {
                    Fun => self.parse_function_item(None)?,
                    Struct => self.parse_struct_declaration(None)?,
                    Trait => self.parse_trait_declaration(None)?,
                    Enum => self.parse_enum_declaration(None)?,
                    Impl => self.parse_impl(None)?,
                    Pub => {
                        let visiblity = self.next.span();

                        self.check_scanning_error_for_next_token()?;
                        self.advance()?;

                        match self.next.unwrap() {
                            Fun => self.parse_function_item(Some(visiblity))?,
                            Struct => self.parse_struct_declaration(Some(visiblity))?,
                            Trait => self.parse_trait_declaration(Some(visiblity))?,
                            Enum => self.parse_enum_declaration(Some(visiblity))?,
                            Impl => self.parse_impl(Some(visiblity))?,
                            _ => {
                                return Err(ParserError::UnexpectedToken(
                                    self.current.clone(),
                                    "`fun`, `trait`, `enum`, `struct`".to_owned(),
                                    "item after `pub`".to_owned(),
                                ));
                            }
                        }
                    }
                    Import => self.parse_import()?,
                    EndOfFile => break,
                    _ => {
                        let err = Err(ParserError::UnexpectedToken(
                            self.next.clone(),
                            "`fun`, `trait`, `enum`, `struct`".to_owned(),
                            "item".to_owned(),
                        ));
                        self.advance()?;
                        return err;
                    }
                }
                .with_docstring(local_docstring),
            );

            local_docstring = self.consume_non_module_docstring()?;
        }

        Ok(items)
    }
}
