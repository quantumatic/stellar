use crate::{error::ParserError, macros::*, Parser, ParserResult};
use num_traits::ToPrimitive;
use ry_ast::{location::Span, precedence::Precedence, token::RawToken::*, *};

impl<'c> Parser<'c> {
    pub(crate) fn parse_function_declaration(
        &mut self,
        public: Option<Span>,
    ) -> ParserResult<Item> {
        self.advance(false)?; // `fun`

        check_token0!(
            self,
            "identifier for function name",
            Identifier(_),
            "function declaration"
        )?;

        let name = self.current_ident_with_span();

        self.advance(false)?; // name

        let generic_annotations = self.parse_generic_annotations()?;

        check_token!(self, OpenParent, "function declaration")?;

        self.advance(false)?; // `(`

        let arguments = parse_list!(self, "function arguments", CloseParent, false, || self
            .parse_function_argument());

        let mut return_type = None;

        if !self.current.value.is(OpenBrace) {
            return_type = Some(self.parse_type()?);
        }

        let stmts = self.parse_statements_block(true)?;

        Ok(Item::FunctionDecl(FunctionDecl {
            def: FunctionDef {
                name,
                generic_annotations,
                params: arguments,
                public,
                return_type,
            },
            stmts,
        }))
    }

    pub(crate) fn parse_function_argument(&mut self) -> ParserResult<FunctionParam> {
        check_token0!(self, "identifier for argument name", Identifier(_))?;

        let name = self.current_ident_with_span();

        self.advance(false)?; // ident

        let r#type = self.parse_type()?;

        let mut default_value = None;

        if self.current.value.is(Assign) {
            self.advance(false)?;

            default_value = Some(self.parse_expression(Precedence::Lowest.to_i8().unwrap())?);
        }

        Ok(FunctionParam {
            name,
            r#type,
            default_value,
        })
    }
}
