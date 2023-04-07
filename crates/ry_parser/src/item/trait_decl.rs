use crate::{
    r#type::{generics::GenericsParser, where_clause::WhereClauseParser},
    OptionalParser, ParseResult, Parser, ParserState,
};
use ry_ast::{
    declaration::TraitDeclarationItem,
    token::{
        Punctuator::{CloseBrace, OpenBrace},
        RawToken::Punctuator,
    },
    Visibility,
};

pub(crate) struct TraitDeclarationParser {
    pub(crate) visibility: Visibility,
}

impl Parser for TraitDeclarationParser {
    type Output = TraitDeclarationItem;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let name = state.consume_identifier("trait name in trait declaration")?;

        let generics = GenericsParser.optionally_parse_with(state)?;

        let r#where = WhereClauseParser.optionally_parse_with(state)?;

        state.consume(Punctuator(OpenBrace), "trait declaration")?;

        let methods = vec![];

        state.consume(Punctuator(CloseBrace), "trait declaration")?;

        Ok(TraitDeclarationItem {
            visibility: self.visibility,
            name,
            generics,
            r#where,
            methods,
        })
    }
}

// impl TraitDeclarationParser {
//     fn parse_trait_associated_functions(&mut self) -> ParseResult<Vec<Documented<Function>>> {
//         let mut associated_functions = vec![];

//         loop {
//             if self.next.inner == Punctuator(CloseBrace) {
//                 break;
//             }

//             let docstring = self.consume_non_module_docstring()?;

//             let mut visibility = Visibility::private();

//             if let Keyword(Pub) = self.next.inner {
//                 visibility = Visibility::public(self.next.span);
//                 self.advance();
//             }

//             associated_functions.push(self.parse_function(visibility)?.with_docstring(docstring));
//         }

//         Ok(associated_functions)
//     }
// }

// #[cfg(test)]
// mod trait_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(empty_trait, "trait test {}");
//     parser_test!(r#trait, "trait test { fun f(); }");
//     parser_test!(
//         r#trait_with_generics,
//         "trait Into[T] { fun into(self: &Self): T; }"
//     );
//     parser_test!(
//         unnecessary_visibility_qualifier,
//         "trait Into[T] { pub fun into(self: &Self): T; }"
//     );
// }
