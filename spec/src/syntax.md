
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

### Identifier

> <pre>
> IDENTIFIER = (XID_Start | `_`) XID_Continue<sup>*</sup>
> </pre>

Identifiers follow the specification in [Unicode Standart Annex #31](https://www.unicode.org/reports/tr31/tr31-37.html) for Unicode 15.0, with the additions described bellow. Some examples of identifiers:

- `i`
- `foo`
- `тест`
- `東京`
- `_test`

### Comment
 
> `//` ~<sup>*</sup>

Comments serve as program documentation and start with character sequence `//` and stop at the end of line.

### Docstring

> MODULE_DOC:
> 
> `//!` ~`\n`<sup>*</sup>
> 
> LOCAL_DOC:
> 
> `///` ~`\n`<sup>*</sup>

## Syntax

### Item
<pre>
Item = <a href="#enum-declaration">EnumDeclaration</a> | <a href="#trait-declaration">TraitDeclaration</a> | <a href="#impl">Impl</a> 
     | <a href="#struct-declaration">StructDeclaration</a> | <a href="#function-declaration">FunctionDeclaration</a> | <a href="#function-definition">FunctionDefinition</a> .
</pre>

#### Enum declaration

<pre>
<a href="#enum-declaration">EnumDeclaration</a> = ["pub"] "enum" "{" <a href="#enum-declaration">EnumVariants</a> "}" .
<a href="#enum-declaration">EnumVariants</a>   = <a href="#enum-declaration">EnumVariant</a> { "," <a href="#enum-declaration">EnumVariant</a> } [","] .
<a href="#enum-declaration">EnumVariant</a>    = <a href="#identifier">identifier</a> .
</pre>

#### Trait declaration

<pre>
trait_declaration =  ["pub"] "trait" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] 
                  "{" { trait_method ";" } "}" .
trait_method      = "fun" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] 
                  <a href="#function-arguments">function_arguments</a> ( ";" | <a href="#block">block</a> ).
</pre>

#### Struct declaration

<pre>
struct_declaration = ["pub"] "struct" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] 
                  "{" { struct_member ";" } "}" .
struct_member      = ["pub"] ["mut"] <a href="#identifier">identifier</a> <a href="#type">type</a> .
</pre>

#### Impl

<pre>
impl                        = "impl" <a href="#type">type</a> [ "for" <a href="#type">type</a> ] "{" trait_method_implementation "}" .
trait_method_implementation = ["pub"] "fun" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] <a href="#function-arguments">function_arguments</a> <a href="#block">block</a> .
</pre>

#### Function declaration

<pre>
function_declaration = ["pub"] "fun" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] <a href="#function-arguments">function_arguments</a> <a href="#block">block</a> .
</pre>

#### Function definition

<pre>
function_definition = ["pub"] "fun" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] <a href="#function-arguments">function_arguments</a> ";" .
</pre>

#### Function arguments

<pre>
function_arguments = "(" function_argument { "," function_argument } [","] ")"
function_argument  = <a href="#identifier">identifier</a> ":" <a href="#type">type</a> ( "=" <a href="#expression">expression</a> ) 
</pre>

### Type

<pre>
type = <a href="#option-type">option_type</a> | <a href="#array-type">array_type</a> | <a href="#reference-type">reference_type</a> | primary_type .
</pre>

#### Option type
<pre>
option_type = <a href="#type">type</a> "?" .
</pre>

#### Array type
<pre>
array_type = "[" <a href="#type">type</a> "]" .
</pre>

#### Reference type
<pre>
reference_type = "&" ["mut"] <a href="#type">type</a> .
</pre>

#### Primary type
<pre>
primary_type = <a href="#name">name</a> [ <a href="#type-annotations">type_annotations</a> ] .
</pre>

#### Type annotations
<pre>
type_annotations = "<" <a href="#type">type</a> { "," <a href="#type">type</a> } [","] ">" .
</pre>

#### Generics
<pre>
generics = "<" <a href="#identifier">identifier</a> [ "of" <a href="#type">type</a> ] ">" .
</pre>

### Statements

#### Block

<pre>
block = "{" { <a href="#statement">statement</a> } "}" .
</pre>

#### Statement

<pre>
statement = <a href="#defer-statement">defer_statement</a> | <a href="#return-statement">return_statement</a> | <a href="#expression-statement">expression_statement</a> .
</pre>

#### Defer statement

<pre>
defer_statement = "defer" <a href="#expression">expression</a> ";" .
</pre>

#### Return statement

<pre>
return_statement = "return" <a href="#expression">expression</a> ";" .
</pre>


#### Expression statement

<pre>
expression_statement = <a href="#expression">expression</a> (";") .
</pre>

### Expression

<pre>
expression = primary_expression | unary_expression | binary_expression | call_expression |  .
</pre>
