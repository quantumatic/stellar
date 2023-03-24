//! This crate provides a parser for Ry programming language
//!
//! It uses the lexer from the ry_lexer crate to tokenize the input source
//! code and produces an Abstract Syntax Tree (AST) that represents the parsed code.
use error::ParserError;
use ry_ast::{
    token::{RawToken::*, Token},
    *,
};
use ry_lexer::Lexer;
use string_interner::StringInterner;

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
    pub fn new(contents: &'a str, string_interner: &'a mut StringInterner) -> Self {
        let mut lexer = Lexer::new(contents, string_interner);

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
        if let Invalid(e) = self.current.value {
            Err(ParserError::ErrorToken(e.with_span(self.current.span)))
        } else {
            Ok(())
        }
    }

    /// Checks if the next token being parsed is invalid, and returns
    /// an error if so.
    fn check_scanning_error_for_next_token(&mut self) -> ParserResult<()> {
        if let Invalid(e) = self.next.value {
            Err(ParserError::ErrorToken(e.with_span(self.next.span)))
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
            if let DocstringComment(global, comment) = self.next.value {
                if global {
                    module_docstring.push(comment);
                } else {
                    local_docstring.push(comment);
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
            if let DocstringComment(global, comment) = self.next.value {
                if !global {
                    result.push(comment);
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
    /// use string_interner::StringInterner;
    ///
    /// let mut interner = StringInterner::new();
    /// let mut parser = Parser::new("fun test() {}", &mut interner);
    /// assert!(parser.parse().is_ok());
    /// ```
    pub fn parse(&mut self) -> ParserResult<ProgramUnit> {
        self.check_scanning_error_for_current_token()?;

        let (module_docstring, fst_docstring) = self.consume_module_and_first_item_docstrings()?;
        Ok(ProgramUnit {
            docstring: module_docstring,
            imports: self.parse_imports()?,
            items: self.parse_items(fst_docstring)?,
        })
    }

    fn parse_items(
        &mut self,
        mut local_docstring: Docstring,
    ) -> ParserResult<Vec<(Docstring, Item)>> {
        let mut top_level_statements = vec![];

        loop {
            top_level_statements.push((
                local_docstring,
                match self.next.value {
                    Fun => self.parse_function_declaration(None)?,
                    Struct => self.parse_struct_declaration(None)?,
                    Trait => self.parse_trait_declaration(None)?,
                    Enum => self.parse_enum_declaration(None)?,
                    Impl => self.parse_impl(None)?,
                    Pub => {
                        let pub_span = self.next.span;

                        self.check_scanning_error_for_next_token()?;
                        self.advance()?;

                        match self.next.value {
                            Fun => self.parse_function_declaration(Some(pub_span))?,
                            Struct => self.parse_struct_declaration(Some(pub_span))?,
                            Trait => self.parse_trait_declaration(Some(pub_span))?,
                            Enum => self.parse_enum_declaration(Some(pub_span))?,
                            Impl => self.parse_impl(Some(pub_span))?,
                            _ => {
                                return Err(ParserError::UnexpectedToken(
                                    self.current.clone(),
                                    "`fun`, `trait`, `enum`, `struct`".to_owned(),
                                    "item after `pub`".to_owned(),
                                ));
                            }
                        }
                    }
                    Import => {
                        let import = self.parse_import()?;

                        Item::Import(import)
                    }
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
                },
            ));

            local_docstring = self.consume_non_module_docstring()?;
        }

        Ok(top_level_statements)
    }
}
