#[cfg(test)]
macro_rules! parse_test {
    ($parser: expr, $name:ident, $source:literal) => {
        #[test]
        #[allow(unused_qualifications)]
        fn $name() {
            let mut diagnostics = vec![];
            let mut string_interner = ry_interner::Interner::default();
            let mut cursor = crate::Cursor::new(0, $source, &mut string_interner, &mut diagnostics);
            let node = crate::Parse::parse_with($parser, &mut cursor);
            assert!(node.is_some());
        }
    };
}

macro_rules! parse_list {
    ($p:ident, $name:literal, $closing_token:pat, $fn:expr) => {
        parse_list!($p, $name, $closing_token, $fn, )
    };
    ($p:ident, $name:literal, $closing_token:pat, $fn:expr, $($fn_arg:expr)*) => {
        {
            let mut result = vec![];

            if !matches!($p.next.unwrap(), $closing_token) {
                loop {
                    result.push($fn($($fn_arg)*)?);

                    #[allow(unused_qualifications)]
                    if !matches!($p.next.unwrap(), $closing_token) {
                        $p.consume(ry_ast::Token![,], $name)?;

                        if matches!($p.next.unwrap(), $closing_token) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }

            result
        }
    };
}

macro_rules! binop_pattern {
    () => {
        ry_ast::Token![+=]
        | ry_ast::Token![+]
        | ry_ast::Token![-=]
        | ry_ast::Token![-]
        | ry_ast::Token![**]
        | ry_ast::Token![*=]
        | ry_ast::Token![*]
        | ry_ast::Token![/=]
        | ry_ast::Token![/]
        | ry_ast::Token![!=]
        | ry_ast::Token![!]
        | ry_ast::Token![>>]
        | ry_ast::Token![>=]
        | ry_ast::Token![>]
        | ry_ast::Token![<<]
        | ry_ast::Token![<=]
        | ry_ast::Token![<]
        | ry_ast::Token![==]
        | ry_ast::Token![=]
        | ry_ast::Token![|=]
        | ry_ast::Token![||]
        | ry_ast::Token![|]
        | ry_ast::Token![&&]
        | ry_ast::Token![~=]
        | ry_ast::Token![%]
    };
}

macro_rules! postfixop_pattern {
    () => {
        ry_ast::Token![?] | ry_ast::Token![++] | ry_ast::Token![--]
    };
}

macro_rules! prefixop_pattern {
    () => {
        ry_ast::Token![!] | ry_ast::Token![~] | ry_ast::Token![++] | ry_ast::Token![--] | ry_ast::Token![-] | ry_ast::Token![+]
    };
}

pub(crate) use {binop_pattern, parse_list, postfixop_pattern, prefixop_pattern};

#[cfg(test)]
pub(crate) use parse_test;
