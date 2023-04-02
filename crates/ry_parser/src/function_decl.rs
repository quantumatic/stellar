use crate::{error::ParserError, macros::*, Parser, ParserResult};
use num_traits::ToPrimitive;
use ry_ast::{
    declaration::{Function, FunctionArgument, FunctionDeclaration, FunctionDefinition, Item},
    precedence::Precedence,
    span::WithSpannable,
    token::RawToken::*,
    *,
};

impl<'c> Parser<'c> {
    pub(crate) fn parse_function_item(&mut self, visibility: Visibility) -> ParserResult<Item> {
        Ok(self.parse_function(visibility)?.into())
    }

    pub(crate) fn parse_function(&mut self, visibility: Visibility) -> ParserResult<Function> {
        let definition = self.parse_function_definition(visibility)?;

        if self.next.unwrap().is(Semicolon) {
            self.advance()?;

            Ok(definition.into())
        } else {
            Ok(FunctionDeclaration::new(definition, self.parse_statements_block(true)?).into())
        }
    }

    pub(crate) fn parse_function_declaration(
        &mut self,
        visibility: Visibility,
    ) -> ParserResult<FunctionDeclaration> {
        Ok(FunctionDeclaration::new(
            self.parse_function_definition(visibility)?,
            self.parse_statements_block(true)?,
        )
        .into())
    }

    fn parse_function_definition(
        &mut self,
        visibility: Visibility,
    ) -> ParserResult<FunctionDefinition> {
        self.advance()?;

        let name = consume_ident!(self, "function name in function declaration");

        let generics = self.parse_generics()?;

        consume!(self, OpenParent, "function declaration");

        let arguments = parse_list!(self, "function arguments", CloseParent, false, || self
            .parse_function_argument());

        self.advance()?;

        let mut return_type = None;

        if self.next.unwrap().is(Colon) {
            self.advance()?; // `:`

            return_type = Some(self.parse_type()?);
        }

        let r#where = self.parse_where_clause()?;

        Ok(FunctionDefinition::new(
            visibility,
            name,
            generics,
            arguments,
            return_type,
            r#where,
        ))
    }

    pub(crate) fn parse_function_argument(&mut self) -> ParserResult<FunctionArgument> {
        let name = consume_ident!(self, "function argument name");

        consume!(self, Colon, "function argument name");

        let r#type = self.parse_type()?;

        let mut default_value = None;

        if self.next.unwrap().is(Assign) {
            self.advance()?;

            default_value = Some(self.parse_expression(Precedence::Lowest.to_i8().unwrap())?);
        }

        Ok(FunctionArgument::new(name, r#type, default_value))
    }
}

#[cfg(test)]
mod function_decl_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(function1, "pub fun test() {}");
    parser_test!(function2, "pub fun test[A](a: A): A { a }");
    parser_test!(function3, "fun unwrap[T, B: T?](a: B): T { a.unwrap() }");
}
