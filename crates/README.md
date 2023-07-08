<p align="center"><img width="70%" src="../additional/icon/banner.png" alt="rycon"></p>

An open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

# Structure of the source code

Ry source code is divided into several crates:

- `ry` - CLI.
- `ry_analyze` - Implements the type inference.
- `ry_ast` - Defines AST nodes, token struct, implements AST serialization.
- `ry_diagnostics` - Implements the diagnostics.
- `ry_filesystem` - Implements some utility functions for easier work with the filesystem.
- `ry_interner` - Implements the identifier interner.
- `ry_lexer` - Implements the lexer.
- `ry_llvm_codegen` - Implements the code generation.
- `ry_manifest` - Implements the toml manifest parser.
- `ry_name_resolution` - Defines the name resolution graph.
- `ry_parser` - Implements the parser.
- `ry_stable_likely` - Implements the stable version of likely and unlikely intrinsics.
- `ry_typed_ast` - Defines typed AST nodes.
