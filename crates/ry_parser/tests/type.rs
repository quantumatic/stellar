mod macros;

tests_using! {
    parse_type,
    underscore -> "_",
    primitive -> "int32",
    type_constructor -> "List[uint32]",
    unit_type -> "()",
    parenthesized_type -> "(A)",
    single_tuple_type -> "(A,)",
    tuple_type -> "(A, B)",
    function_type1 -> "fun (A, B)",
    function_type2 -> "fun (A, B): C"
}
