mod r#macro;

test!(function1: "fun test();");
test!(function2: "fun test[A](a: A): A { a }");
test!(function3: "fun unwrap[T, B = Option[T]](a: B): T { a.unwrap() }");
test!(function4: "fun foo(a: uint32, b: uint32, fn: (uint32, uint32): uint32): uint32 { fn(a, b) }");
test!(impl1: "impl[T] NotOption for T {}");
test!(impl2: "impl[T] Into[Option[M]] for Tuple[T, M] where M: Into[T] {}");
test!(import1: "import test;");
test!(import2: "import test2.test;");
test!(import4: "import std.io as myio;");
test!(empty_struct: "struct test {}");
test!(struct1: "struct Point[T: Numeric] { pub x: T, pub y: T }");
test!(struct2: "struct Lexer[S] where S: Iterator[char] + Default { contents: S }");
test!(struct3: "struct StringWrapper(String);");
test!(empty_trait: "trait test {}");
test!(trait1: "trait test { fun f(); }");
test!(trait2: "trait Into[T] { fun into(self: Self): T; }");
test!(trait3: "trait Into[T] { fun into(self): T; }");
test!(empty_type_alias: "type A;");
test!(type_alias1: "type B = Option[i32];");
test!(type_alias2: "type B[T] = Option[T];");
test!(empty_enum: "enum test {}");
test!(enum1: "enum Result[T, E] { Ok(T), Err(E) }");
test!(enum2: "enum Option[T] { Some(T), None }");
test!(enum3: "pub enum UserPrincipal {
        Full {
            email: string,
            phone_number: PhoneNumber,
        },
        EmailOnly { email: string },
        PhoneNumberOnly { phone_number: PhoneNumber },
}");
