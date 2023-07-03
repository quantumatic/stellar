<p align="center"><img width="70%" src="../additional/icon/banner.png" alt="rycon"></p>

An open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

# Structure of the source code

Ry source code is divided into several crates:

- `ry_ast` - Defines the AST nodes.
- `ry_lexer` - Implements the lexer.
- `ry_parser` - Implements the parser.
- `ry_workspace` - Implements some utility functions for easier work with the source code.
- `ry_interner` - Implements the identifier interner for the Ry programming language (see [crate documentation](/ry_interner/README.md) for more details).
- `ry_diagnostics` - Implements the diagnostics for the Ry programming language compiler.
- `ry_type_inference` - Implements the type inference for the Ry programming language.
