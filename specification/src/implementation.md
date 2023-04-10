# Implementation
> **<sup>Syntax</sup>**\
> _Implementation_ :\
> &nbsp;&nbsp; `pub`<sup>?</sup> `impl`
>   [_Generics_]<sup>?</sup>
>   [_Type_]
>   (`for` [_Type_])<sup>?</sup>
>   [_WhereClause_]<sup>?</sup>
>   `{` [_Function_]<sup>*</sup> `}`

## Examples

```ry
impl ToString for u64 {
    fun to_string(self: u64): string { sprint("%f", self) }
}
```

[IDENTIFIER]: ./identifier.md
[_Generics_]: ./generics.md
[_WhereClause_]: ./where_clause.md
[_Type_]: ./type.md
[_StatementsBlock_]: ./statements_block.md
[_Function_]: ./function.md
