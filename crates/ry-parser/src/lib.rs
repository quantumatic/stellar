//! `lib.rs` - implements parser for Ry source files.
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

pub(crate) type ParserResult<T> = Result<T, ParserError>;

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

    fn check_scanning_error_for_current_token(&mut self) -> ParserResult<()> {
        if let Invalid(e) = self.current.value {
            Err(ParserError::ErrorToken(e.with_span(self.current.span)))
        } else {
            Ok(())
        }
    }

    fn check_scanning_error_for_next_token(&mut self) -> ParserResult<()> {
        if let Invalid(e) = self.next.value {
            Err(ParserError::ErrorToken(e.with_span(self.next.span)))
        } else {
            Ok(())
        }
    }

    fn advance(&mut self) -> ParserResult<()> {
        self.check_scanning_error_for_next_token()?;

        self.current = self.next.clone();

        self.next = self.lexer.next_no_comments().unwrap();

        Ok(())
    }

    fn advance_with_comments(&mut self) -> ParserResult<()> {
        self.check_scanning_error_for_next_token()?;

        self.current = self.next.clone();

        self.next = self.lexer.next().unwrap();

        Ok(())
    }

    pub(crate) fn consume_fst_docstring(&mut self) -> ParserResult<(Docstring, Docstring)> {
        let (mut module_docstring, mut local_docstring) = (vec![], vec![]);

        loop {
            if let Comment(s) = self.next.value {
                let str = self.lexer.string_interner.resolve(s).unwrap();

                if str.starts_with('!') {
                    module_docstring.push(s);
                } else if str.starts_with('/') {
                    local_docstring.push(s);
                }
            } else {
                return Ok((module_docstring, local_docstring));
            }

            self.advance_with_comments()?;
        }
    }

    pub(crate) fn consume_local_docstring(&mut self) -> ParserResult<Docstring> {
        let mut result = vec![];

        loop {
            if let Comment(s) = self.next.value {
                let str = self.lexer.string_interner.resolve(s).unwrap();

                if str.starts_with('/') {
                    result.push(s);
                }
            } else {
                return Ok(result);
            }

            self.advance_with_comments()?;
        }
    }

    pub fn parse(&mut self) -> ParserResult<ProgramUnit> {
        self.check_scanning_error_for_current_token()?;

        let (module_docstring, fst_docstring) = self.consume_fst_docstring()?;
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

            local_docstring = self.consume_local_docstring()?;
        }

        Ok(top_level_statements)
    }
}
