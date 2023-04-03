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

A _struct_ is a nominal struct type defined with the keyword `struct`.

An example of a `struct` item and its use:

```ry
struct RGB {
    pub red u8;
    pub green u8;
    pub blue u8;
}

pub fun main() {
    var green = RGB {
        red: 0,
        green: 255,
        blue: 0
    };
    println("%d", green.red);
}
```

Example of more complex _struct item_ with generics and where clause:
```ry
struct JSONParser[I] where I: Iterator[char] {
    source: I;
    mut cursor: usize;
}
```


[IDENTIFIER]: identifier.md
[_Generics_]: generics.md
[_WhereClause_]: where_clause.md
[_Type_]: type.md
