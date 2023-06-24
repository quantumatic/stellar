mod r#macro;

test!(r#return: "fun test(): int32 { return a()?.b.unwrap_or(0); }");
test!(let_and_defer: "fun test() {
    let file = File.open(\"hello.txt\");
    defer file.close();
}");
test!(let1: "fun test(): int32 {
    let a = 1;
    let b = 2;
    a + b
}");
test!(let2: "fun test(): int32 {
    let Some(a) = Some(2);
}");
// grouped pattern
test!(let3: "fun test(): int32 {
    let (Some(a)) = Some(2);
}");
test!(let4: "fun test(): int32 {
    let #(Some(a), None) = #(Some(2), None);
}");
// or
test!(let5: "fun test(): int32 {
    let A(a) | B(a) = A(2);
}");
test!(let6: "fun test(): int32 {
    let a: Option[int32] = Some(2);
}");
test!(let7: "fun test(): int32 {
    let a: Iterator[Item = uint32].Item = 3;
    a
}");
test!(let8: "fun test(): uint32 {
    let a = [1, 2, 3].into_iter() as dyn Iterator[Item = uint32];
    a.next()
}");

// parenthesized type
test!(let9: "fun main() { let a: (((uint32))) = 2; }");

test!(let10: "fun main() { let a: [List[uint32] as IntoIterator].Item = 3; }");

test!(r#break: "fun test() {
    while true {
        break;
    }
}");
test!(r#continue: "fun test() {
    while true {
        continue;
    }
}");
