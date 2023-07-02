mod r#macro;

test!(literal1: "fun test(): int32 { 3 }");
test!(literal2: "fun test(): string { \"hello\" }");
test!(literal3: "fun test(): bool { true }");
test!(literal4: "fun test(): bool { false }");
test!(literal5: "fun test(): float32 { 3.2 }");
test!(literal6: "fun test(): char { 'a' }");
test!(list: "fun test(): List[int32] { [1, 2, \"3\".into()] }");
test!(tuple: "fun test(): (int32, float32, string) { (1, 3.2, \"hello\") }");
test!(binary: "fun test(): int32 { 1 + 2 }");
test!(primitive_cast: "fun test(): float32 { 1 as float32 }");
test!(call: "fun test(): int32 { 2 * b() + 2 }");
test!(call_with_generic_arguments: "fun test(): usize { sizeof[int32]() }");
test!(generic_arguments: "fun test() { let a = [1, 2, 3].into_iter() as Dispatcher[Iterator[Item = uint32]]; }");
test!(ifelse: "fun test(): bool { if false { 2.3 } else if false { 5 as float32 } else { 2.0 } }");
test!(postfix: "fun test(): int32 { Some(a().unwrap_or(0) + b()?) }");
test!(r#struct: "fun test(): Person { Person { age: 3, name } }");
test!(match1: "fun test(): int32 { match Some(3) { Some(a) => println(a), .. => {} } }");
// identifier pattern + rest pattern
test!(match2: "fun test(): int32 {
    match Some([1, 2, 3]) {
        Some([1, a @ ..]) => println(a),
        .. => {}
    }
}");
test!(function: "fun test(): int32 { |a: int32, b: int32|: int32 { a + b } }");
test!(r#while: "fun test() {
    let x = 0;

    while x <= 100 {
        x++;

        if x % 15 == 0 {
            println(\"FizzBuzz\");
        } else if x % 3 == 0 {
            println(\"Fizz\");
        } else if x % 5 == 0 {
            println(\"Buzz\");
        } else {
            println(x.to_string());
        }
    }
}");
