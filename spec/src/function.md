# Function

> **<sup>Syntax</sup>**\
> _Function_ :\
> &nbsp;&nbsp; `pub`<sup>?</sup> `fun`
>   [IDENTIFIER]
>   [_Generics_]<sup>?</sup>
>   `(` _FunctionArguments_ `)`
>   [_WhereClause_]<sup>?</sup>
>   ( [_StatementsBlock_] | `;` )
> 
> _FunctionArguments_ :\
> &nbsp;&nbsp; _FunctionArgument_ (`,` _FunctionArgument_)<sup>\*</sup> `,`<sup>`?`</sup>
> 
> _FunctionArgument_ :\
> &nbsp;&nbsp; [IDENTIFIER] `:` [_Type_]


[IDENTIFIER]: identifier.md
[_Generics_]: generics.md
[_WhereClause_]: where_clause.md
[_Type_]: type.md
[_StatementsBlock_]: statements_block.md
