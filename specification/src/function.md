# Function

> **<sup>Syntax</sup>**\
> _Function_ :\
> &nbsp;&nbsp; `pub`<sup>?</sup> `fun` > [IDENTIFIER][_generics_]<sup>?</sup> > `(` _FunctionArguments_ `)` > [_WhereClause_]<sup>?</sup>
> ( [_StatementsBlock_] | `;` )
>
> _FunctionArguments_ :\
> &nbsp;&nbsp; _FunctionArgument_ (`,` _FunctionArgument_)<sup>\*</sup> (`,` `...`)<sup>?</sup> `,`<sup>?</sup>
>
> _FunctionArgument_ :\
> &nbsp;&nbsp; [IDENTIFIER] `:` [_Type_] <br> &nbsp;&nbsp; | `self`

## Examples:

```ry
pub fun sum[T: Numeric](a: T, b: T): T {
    a + b
}

// defined in std lib
@[builtin(sprint)]
pub fun sprint(format: string, ...): string;

pub fun len(s: string): usize {
    s.len()
}
```

[identifier]: identifier.md
[_generics_]: generics.md
[_whereclause_]: where_clause.md
[_type_]: type.md
[_statementsblock_]: statements_block.md
