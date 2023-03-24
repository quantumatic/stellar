
# Ry 1.0 - Syntax - Item

## Item
```
item = enum_declaration | trait_declaration | impl | struct_declaration | function_declaration .
```

## Enum declaration

```
enum_declaration = "enum" "{" enum_variants "}" .
enum_variants = enum_variant { "," enum_variant } [","] .
```
