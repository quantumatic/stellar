# Identifier

> **<sup>Lexer:<sup>**\
> IDENTIFIER_OR_KEYWORD :\
> &nbsp;&nbsp; &nbsp;&nbsp; XID_Start XID_Continue<sup>\*</sup>\
> &nbsp;&nbsp; | `_` XID_Continue<sup>\*</sup>
>
> RAW_IDENTIFIER : `` ` `` IDENTIFIER_OR_KEYWORD `` ` ``
>
> NON_KEYWORD_IDENTIFIER : IDENTIFIER_OR_KEYWORD <sub>*Except keyword*</sub>

> Normalization is planned to be here
