
# Ry 1.0 - Syntax

# Item
<pre>
item = <a href="#enum-declaration">enum_declaration</a> | <a href="#trait-declaration">trait_declaration</a> | <a href="#impl">impl</a> | <a href="#struct-declaration">struct_declaration</a> | <a href="#function-declaration">function_declaration</a> .
</pre>

## Enum declaration

<pre>
enum_declaration = ["pub"] "enum" "{" enum_variants "}" .
enum_variants = enum_variant { "," enum_variant } [","] .
enum_variant = <a href="#identifier">identifier</a> .
</pre>

## Trait declaration

<pre>
trait_declaration =  ["pub"] "trait" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] "{" { trait_method ";" } "}" .
trait_method = "fun" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] <a href="#function-arguments">function_arguments</a> ( ";" | <a href="#block">block</a> ).
</pre>

## Struct declaration

<pre>
struct_declaration = ["pub"] "struct" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] "{" [ "pub" ] [ "mut" ] <a href="#identifier">identifier</a> <a href="#type">type</a> "}" .
</pre>

## Impl

<pre>
impl = "impl" <a href="#type">type</a> [ "for" <a href="#type">type</a> ] "{" trait_method_implementation "}" .
trait_method_implementation = ["pub"] "fun" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] <a href="#function-arguments">function_arguments</a> <a href="#block">block</a> .
</pre>

## Function declaration

<pre>
function_declaration = ["pub"] "fun" <a href="#identifier">identifier</a> [ <a href="#generics">generics</a> ] [ <a href="#where-clause">where_clause</a> ] <a href="#function-arguments">function_arguments</a> <a href="#block">block</a> .
</pre>

# Type

<pre>
type = option_type | array_type | reference_type | primary_type .
</pre>

## Option type
<pre>
option_type = <a href="#type">type</a> "?" .
</pre>

## Array type
<pre>
array_type = "[" <a href="#type">type</a> "]" .
</pre>

## Reference type
<pre>
reference_type = "&" ["mut"] <a href="#type">type</a> .
</pre>

## Primary type
<pre>
primary_type = <a href="#name">name</a> [ <a href="#type_annotations">type_annotations</a> ] .
</pre>

## Type annotations
<pre>
type_annotations = "<" <a href="#type">type</a> { "," <a href="#type">type</a> } [","] ">" .
</pre>

## Generics
<pre>
generics = "<" <a href="#identifier">identifier</a> [ "of" <a href="#type">type</a> ] ">" .
</pre>
