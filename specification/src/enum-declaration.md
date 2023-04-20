# Enum declaration

> **<sup>Syntax</sup>**\
> _EnumDeclaration_ :\
> &nbsp;&nbsp; `pub`<sup>?</sup> `enum`
>   [IDENTIFIER]
>   `{` _EnumVariants_<sup>?</sup> `}`
> 
> _EnumVariants_ :\
> &nbsp;&nbsp; [IDENTIFIER] (`,` [IDENTIFIER])<sup>\*</sup> `,`<sup>?</sup>

[IDENTIFIER]: ./identifier.md

## Examples:

```ry
enum BaseColor {
    Red,
    Green,
    Blue
}

enum EmptyEnum {}
```
