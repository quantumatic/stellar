mod macros;

tests_using! {
    parse_pattern,
    wildcard -> "_",
    rest -> "..",
    literal -> "3",
    identifier1 -> "foo",
    identifier2 -> "foo @ [1, ..]",
    tuple -> "(1, 2, _)",
    tuple2 -> "(1,)",
    tuple3 -> "(..)",
    list -> "[1, .., 3]",
    struct_ -> "Person { name: \"John\", age, .. }",
    or -> "Some(_) | None"
}
