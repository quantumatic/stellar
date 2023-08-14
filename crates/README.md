An open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

# Structure of the source code

Ry source code is divided into several crates:

- `ry` - CLI.
- `ry_ast` - Defines AST nodes, token struct, implements AST serialization.
- `ry_ast_lowering` - Implements lowering AST to HIR.
- `ry_diagnostics` - Implements the diagnostics wrapper over `codespan_reporting` crate.
- `ry_filesystem` - Implements some utility functions for easier work with the OS and in-memory filesystem.
- `ry_fx_hash` - Implements fx hash algorithm and defines `FxHashMap` and `FxHashSet`.
- `ry_hir` - Defines HIR nodes.
- `ry_interner` - Implements different strings interners.
- `ry_lexer` - Implements the lexer.
- `ry_llvm_codegen` - Implements the code generation.
- `ry_manifest` - Implements the toml manifest parser.
- `ry_name_resolution` - Implements name resolution.
- `ry_parser` - Implements the parser.
- `ry_stable_likely` - Brings likely and unlikely intrinsics to stable Rust.
- `ry_thir` - Defines typed HIR nodes.
- `ry_typechecker` - Implements type checking.
