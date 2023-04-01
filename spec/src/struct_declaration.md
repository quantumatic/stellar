# Struct declaration

> **<sup>Syntax</sup>**\
> _StructDeclaration_ :\
> &nbsp;&nbsp; `pub`<sup>?</sup> `struct`
>   [IDENTIFIER]
>   [_Generics_]<sup>?</sup>
>   [_WhereClause_]<sup>?</sup>
>   `{` _StructField_<sup>*</sup> `}`
> 
> _StructField_ :\
> &nbsp;&nbsp; `pub`<sup>?</sup> `mut`<sup>?</sup> [IDENTIFIER] `:` [_Type_] `;`


[IDENTIFIER]: identifier.md
[_Generics_]: generics.md
[_WhereClause_]: where_clause.md
[_Type_]: type.md
