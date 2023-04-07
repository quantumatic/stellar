use crate::{
    error::ParseResult,
    r#type::{generics::GenericsParser, where_clause::WhereClauseParser, TypeParser},
    OptionalParser, Parser, ParserState,
};
use ry_ast::{
    declaration::ImplItem,
    token::{
        Keyword::For,
        Punctuator::CloseBrace,
        RawToken::{Keyword, Punctuator},
    },
    Visibility,
};

pub(crate) struct ImplItemParser {
    pub(crate) visibility: Visibility,
}

impl Parser for ImplItemParser {
    type Output = ImplItem;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let generics = GenericsParser.optionally_parse_with(state)?;

        let mut r#type = TypeParser.parse_with(state)?;
        let mut r#trait = None;

        if state.next.inner == Keyword(For) {
            state.advance();

            r#trait = Some(r#type);
            r#type = TypeParser.parse_with(state)?;
        }

        let r#where = WhereClauseParser.optionally_parse_with(state)?;

        state.advance();

        let implementations = vec![];

        state.consume(Punctuator(CloseBrace), "type implementation")?;

        Ok(ImplItem {
            visibility: self.visibility,
            generics,
            r#type,
            r#trait,
            r#where,
            implementations,
        }
        .into())
    }
}

// #[cfg(test)]
// mod impl_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(impl1, "impl[T] NotOption for T {}");
//     parser_test!(
//         impl2,
//         "impl[T] Into[Option[M]] for Tuple[T, M] where M: Into[T] {}"
//     );
// }
