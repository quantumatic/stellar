# Ry 1.0 programming language specification

## Notation
The syntax is specified using Wirth syntax notation (WSN), alternative to BNF:
```
Syntax      = { Production } .
Production  = production_name "=" [ Expression ] "." .
Expression  = Term { "|" Term } .
Term        = Factor { Factor } .
Factor      = production_name | token [ "…" token ] | Group | Option | Repetition .
Group       = "(" Expression ")" .
Option      = "[" Expression "]" .
Repetition  = "{" Expression "}" .
```
Productions are expressions constructed from terms and the following operators, in increasing precedence:
```
|   alternation
()  grouping
[]  option (0 or 1 times)
{}  repetition (0 to n times)
```

## Tokens

### Identifiers

> <pre>
> IDENTIFIER = (XID_Start | `_`) XID_Continue<sup>*</sup>
> </pre>

Identifiers follow the specification in [Unicode Standart Annex #31](https://www.unicode.org/reports/tr31/tr31-37.html) for Unicode 15.0, with the additions described bellow. Some examples of identifiers:

- `i`
- `foo`
- `тест`
- `東京`
- `_test`

### Comments

> MODULE_DOC:
> 
> `//!` ~`\n`<sup>*</sup>
> 
> LOCAL_DOC:
> 
> `///` ~`\n`<sup>*</sup>

Comments serve as program documentation and start with character sequence `//` and stop at the end of line.



## Syntax

### Item
<pre>
item = <a href="#?id=enum-declaration">enum_declaration</a> | <a href="#?id=trait-declaration">trait_declaration</a> | <a href="#?id=impl">impl</a> 
     | <a href="#?id=struct-declaration">struct_declaration</a> | <a href="#?id=function-declaration">function_declaration</a> .
</pre>

#### Enum declaration

<pre>
enum_declaration = ["pub"] "enum" "{" enum_variants "}" .
enum_variants    = enum_variant { "," enum_variant } [","] .
enum_variant     = <a href="#?id=identifier">identifier</a> .
</pre>

#### Trait declaration

<pre>
trait_declaration =  ["pub"] "trait" <a href="#?id=identifier">identifier</a> [ <a href="#?id=generics">generics</a> ] [ <a href="#?id=where-clause">where_clause</a> ] 
                  "{" { trait_method ";" } "}" .
trait_method      = "fun" <a href="#?id=identifier">identifier</a> [ <a href="#?id=generics">generics</a> ] [ <a href="#?id=where-clause">where_clause</a> ] 
                  <a href="#?id=function-arguments">function_arguments</a> ( ";" | <a href="#?id=block">block</a> ).
</pre>

#### Struct declaration

<pre>
struct_declaration = ["pub"] "struct" <a href="#?id=identifier">identifier</a> [ <a href="#?id=generics">generics</a> ] [ <a href="#?id=where-clause">where_clause</a> ] 
                  "{" { struct_member ";" } "}" .
struct_member      = ["pub"] ["mut"] <a href="#?id=identifier">identifier</a> <a href="#?id=type">type</a> .
</pre>

#### Impl

<pre>
impl                        = "impl" <a href="#?id=type">type</a> [ "for" <a href="#?id=type">type</a> ] "{" trait_method_implementation "}" .
trait_method_implementation = ["pub"] "fun" <a href="#?id=identifier">identifier</a> [ <a href="#?id=generics">generics</a> ] [ <a href="#?id=where-clause">where_clause</a> ] <a href="#?id=function-arguments">function_arguments</a> <a href="#?id=block">block</a> .
</pre>

#### Function declaration

<pre>
function_declaration = ["pub"] "fun" <a href="#?id=identifier">identifier</a> [ <a href="#?id=generics">generics</a> ] [ <a href="#?id=where-clause">where_clause</a> ] <a href="#?id=function-arguments">function_arguments</a> <a href="#?id=block">block</a> .
</pre>

### Type

<pre>
type = <a href="#?id=option-type">option_type</a> | <a href="#?id=array-type">array_type</a> | <a href="#?id=reference-type">reference_type</a> | primary_type .
</pre>

#### Option type
<pre>
option_type = <a href="#?id=type">type</a> "?" .
</pre>

#### Array type
<pre>
array_type = "[" <a href="#?id=type">type</a> "]" .
</pre>

#### Reference type
<pre>
reference_type = "&" ["mut"] <a href="#?id=type">type</a> .
</pre>

#### Primary type
<pre>
primary_type = <a href="#?id=name">name</a> [ <a href="#?id=type_annotations">type_annotations</a> ] .
</pre>

#### Type annotations
<pre>
type_annotations = "<" <a href="#?id=type">type</a> { "," <a href="#?id=type">type</a> } [","] ">" .
</pre>

#### Generics
<pre>
generics = "<" <a href="#?id=identifier">identifier</a> [ "of" <a href="#?id=type">type</a> ] ">" .
</pre>

### Statements

#### Block

<pre>
block = "{" { statement ";" } "}" .
</pre>

#### Statement

<pre>
statement = {  }
</pre>
