use crate::{
    error::*,
    expression::ExpressionParser,
    macros::*,
    r#type::{GenericsParser, TypeParser, WhereClauseParser},
    statement::StatementsBlockParser,
    OptionalParser, Parser, ParserState,
};
use ry_ast::{
    declaration::{Function, FunctionArgument, FunctionDeclaration, FunctionTypeSignature},
    token::{Punctuator::*, RawToken::*},
    Visibility,
};

pub(crate) struct FunctionArgumentParser;

impl Parser for FunctionArgumentParser {
    type Output = FunctionArgument;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let name = state.consume_identifier("function argument name")?;

        state.consume(Punctuator(Colon), "function argument name")?;

        let r#type = TypeParser.parse_with(state)?;

        let mut default_value = None;

        if state.next.inner == Punctuator(Assign) {
            state.advance();
            default_value = Some(ExpressionParser::default().parse_with(state)?);
        }

        Ok(FunctionArgument {
            name,
            r#type,
            default_value,
        })
    }
}

pub(crate) struct FunctionTypeSignatureParser {
    pub(crate) visibility: Visibility,
}

impl Parser for FunctionTypeSignatureParser {
    type Output = FunctionTypeSignature;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let name = state.consume_identifier("function name in function declaration")?;

        let generics = GenericsParser.optionally_parse_with(state)?;

        state.consume(Punctuator(OpenParent), "function declaration")?;

        let arguments = parse_list!(state, "function arguments", Punctuator(CloseParent), || {
            FunctionArgumentParser.parse_with(state)
        });

        state.advance();

        let mut return_type = None;

        if state.next.inner == Punctuator(Colon) {
            state.advance();
            return_type = Some(TypeParser.parse_with(state)?);
        }

        let r#where = WhereClauseParser.optionally_parse_with(state)?;

        Ok(FunctionTypeSignature {
            visibility: self.visibility,
            name,
            generics,
            arguments,
            return_type,
            r#where,
        })
    }
}

pub(crate) struct FunctionParser {
    pub(crate) visibility: Visibility,
}

impl Parser for FunctionParser {
    type Output = Function;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let signature = FunctionTypeSignatureParser {
            visibility: self.visibility,
        }
        .parse_with(state)?;

        if state.next.inner == Punctuator(Semicolon) {
            state.advance();
            Ok(signature.into())
        } else {
            Ok(FunctionDeclaration {
                signature,
                body: StatementsBlockParser.parse_with(state)?,
            }
            .into())
        }
    }
}

// #[cfg(test)]
// mod function_decl_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(function1, "pub fun test() {}");
//     parser_test!(function2, "pub fun test[A](a: A): A { a }");
//     parser_test!(
//         function3,
//         "fun unwrap[T, B: Option[T]](a: B): T { a.unwrap() }"
//     );
// }
