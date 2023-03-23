use crate::{error::ParserError, macros::*, Parser, ParserResult};
use num_traits::ToPrimitive;
use ry_ast::{location::Span, precedence::Precedence, token::RawToken::*, *};

impl<'c> Parser<'c> {
    pub(crate) fn parse_function_declaration(
        &mut self,
        public: Option<Span>,
    ) -> ParserResult<Item> {
        self.advance()?;

        let name = consume_ident!(self, "function name in function declaration");

        let generic_annotations = self.parse_generic_annotations()?;

        consume!(self, OpenParent, "function declaration");

        let arguments = parse_list!(self, "function arguments", CloseParent, false, || self
            .parse_function_argument());

        self.advance()?;

        let mut return_type = None;

        if !self.next.value.is(OpenBrace) && !self.next.value.is(Where) {
            return_type = Some(self.parse_type()?);
        }

        let r#where = self.parse_where_clause()?;

        let stmts = self.parse_statements_block(true)?;

        Ok(Item::FunctionDecl(FunctionDecl {
            def: FunctionDef {
                name,
                generic_annotations,
                params: arguments,
                public,
                return_type,
                r#where,
            },
            stmts,
        }))
    }

    pub(crate) fn parse_function_argument(&mut self) -> ParserResult<FunctionParam> {
        let name = consume_ident!(self, "function argument name");

        let r#type = self.parse_type()?;

        let mut default_value = None;

        if self.next.value.is(Assign) {
            self.advance()?;

            default_value = Some(self.parse_expression(Precedence::Lowest.to_i8().unwrap())?);
        }

        Ok(FunctionParam {
            name,
            r#type,
            default_value,
        })
    }
}

#[cfg(test)]
mod function_decl_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(function1, "pub fun test() {}");
    parser_test!(function2, "pub fun test[A](a A) A { a }");
    parser_test!(function3, "fun unwrap[T, B of T?](a B) T { a.unwrap() }");
}
