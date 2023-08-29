An open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

# Structure of the source code

Stellar source code is divided into several crates:

- `stellar` - CLI.
- `stellar_ast` - Defines AST nodes, token struct, implements AST serialization.
- `stellar_ast_lowering` - Implements lowering AST to HIR.
- `stellar_diagnostics` - Implements beautiful diagnostics emittion.
- `stellar_filesystem` - Implements some utility functions for easier work with the OS and in-memory filesystem.
- `stellar_fx_hash` - Implements fx hash algorithm and defines `FxHashMap` and `FxHashSet`.
- `stellar_hir` - Defines HIR nodes.
- `stellar_interner` - Implements different strings interners.
- `stellar_lexer` - Implements the lexer.
- `stellar_llvm_codegen` - Implements the code generation.
- `stellar_manifest` - Implements the toml manifest parser.
- `stellar_name_resolution` - Implements name resolution.
- `stellar_parser` - Implements the parser.
- `stellar_stable_likely` - Brings likely and unlikely intrinsics to stable Rust.
- `stellar_thir` - Defines typed HIR nodes.
- `stellar_typechecker` - Implements type checking.
