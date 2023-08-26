mod macros;

tests_using! {
    parse_expression,

    int_literal -> "1",
    float_literal -> "1.2",
    string_literal -> "\"string\"",
    bool_literal -> "true",
    identifier -> "foo",
    call -> "f()",
    nested_call -> "f()()",
    method_call -> "a.f()",
    integer_method_call -> "1.to_string()",
    float_method_call -> "1.2.to_string()",
    binary1 -> "1 + 2",
    binary2 -> "(f()? + 2) / 3.2 + !a()",
    struct_ -> "Person { name: \"John\", age }",
    tuple -> "(1, (1, \"hello\"), true)",
    list -> "[1, 2, 3]",
    postfix -> "checked_div(1, 0)?",
    prefix -> "!++a",
    double_plus_hell -> "++a++",
    if_else -> "if true { 1 } else if f() { 3 } else { 2 }",
    loop_ -> "loop {}",
    while_ -> "while true { }",
    underscore -> "_",
    match_ -> "match true { true -> 1, _ -> 2 }",
    lambda -> "|a, b: usize| a + b",
    block -> "{ a++; a }"
}
