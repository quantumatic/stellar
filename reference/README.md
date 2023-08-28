<p align="center">
    <img width="40%" src="../additional/icon/banner.png">
</p>

# Official Ry 0.1.0 programming language reference

# Introduction

This is the reference manual for the Ry programming language.

Ry is a general-purpose language designed with systems programming in mind. It is strongly typed and garbage-collected.

# Notation

The syntax is specified using a [variant](https://en.wikipedia.org/wiki/Wirth_syntax_notation) of Extended Backus-Naur Form (EBNF):

```wsn
Syntax = { Production } .
Production = production_name "=" [ Expression ] "." .
Expression = Term { "|" Term } .
Term = Factor { Factor } .
Factor = production_name | token [ "…" token ] | Group | Option | Repetition .
Group = "(" Expression ")" .
Option = "[" Expression "]" .
Repetition = "{" Expression "}" .
```

Productions are expressions constructed from terms and the following operators, in increasing precedence:

```
| alternation
() grouping
[] option (0 or 1 times)
{} repetition (0 to n times)
```

Lowercase production names are used to identify lexical (terminal) tokens. Non-terminals are in CamelCase. Lexical tokens are enclosed in double quotes "" or back quotes \`\`.

The form `a … b` represents the set of characters from a through b as alternatives. The horizontal ellipsis `…` is also used elsewhere in the spec to informally denote various enumerations or code snippets that are not further specified. The character `…` (as opposed to the three characters `...`) is not a token of the Ry language.

# Source code representation

Source code is Unicode text encoded in UTF-8. The text is not canonicalized, so a single accented code point is distinct from the same character constructed from combining an accent and a letter; those are treated as two code points. For simplicity, this document will use the unqualified term character to refer to a Unicode code point in the source text.

Each code point is distinct; for instance, uppercase and lowercase letters are different characters.

# Characters

The following terms are used to denote specific Unicode character categories:

```wsn
newline = /* the Unicode code point U+000A */ .
unicode_char = /* an arbitrary Unicode code point except newline */ .
unicode_letter = /* a Unicode code point categorized as "Letter" */ .
unicode_digit = /* a Unicode code point categorized as "Number, decimal digit" */ .
```

In The Unicode Standard 8.0, Section 4.5 "General Category" defines a set of character categories. Ry treats all characters in any of the Letter categories `Lu`, `Ll`, `Lt`, `Lm`, or `Lo` as Unicode letters, and those in the Number category `Nd` as Unicode digits.

# Letters and digits

> [!NOTE]
> The underscore character `_` (`U+005F`) is **not** considered a letter.

```wsn
letter = unicode_letter | "*" .
decimal_digit = "0" … "9" .
binary_digit = "0" | "1" .
octal_digit = "0" … "7" .
hex_digit = "0" … "9" | "A" … "F" | "a" … "f" .
```

# Lexical elements

## Comments

Comments serve as program documentation and start with `//`.

> [!NOTE]  
> A comment cannot start inside a char or string literal, or inside another comment.

## Identifiers

Identifiers name program entities such as variables and types. An identifier is a sequence of one or more letters and digits. The first character in an identifier must be a letter.

```wsn
letter_or_underscore = letter | "_" .
identifier           = letter { letter_or_underscore | unicode_digit }
                     | "_" ( letter_or_underscore | unicode_digit )
                       { letter_or_underscore | unicode_digit } .
```

```
a
_x9
ThisVariableIsExported
αβ
```

> [!NOTE]
> single `_` character is not considered an identifier, but rather a [punctuation](#operators-and-punctuation).

Some identifiers are predeclared.

## Keywords

The following keywords are reserved and may not be used as identifiers.

```
as defer else enum for fun if pub return struct type let
where while match import break continue dyn loop
interface implements
```

## Operators and punctuation

The following character sequences represent operators (including assignment operators) and punctuation:

```
-> & &= && * ** *= @ ! } ] ) : , . .. = ==
> >= << < <= - -= -- ~ != { [ ( | |= || %
%= + += ++ ? >> ; / /= ^ ^= # _
```

## Integer literals

An integer literal is a sequence of digits representing an integer constant. An optional prefix sets a non-decimal base: `0b` or `0B` for binary, `0`, `0o`, or `0O` for octal, and `0x` or `0X` for hexadecimal. A single `0` is considered a decimal zero. In hexadecimal literals, letters `a` through `f` and `A` through `F` represent values `10` through `15`.

For readability, an underscore character `_` may appear after a base prefix or between successive digits; such underscores do not change the literal's value.

```wsn
int_lit        = decimal_lit | binary_lit | octal_lit | hex_lit .
decimal_lit    = "0" | ( "1" … "9" ) [ [ "_" ] decimal_digits ] .
binary_lit     = "0" ( "b" | "B" ) [ "_" ] binary_digits .
octal_lit      = "0" [ "o" | "O" ] [ "_" ] octal_digits .
hex_lit        = "0" ( "x" | "X" ) [ "_" ] hex_digits .

decimal_digits = decimal_digit { [ "_" ] decimal_digit } .
binary_digits  = binary_digit { [ "_" ] binary_digit } .
octal_digits   = octal_digit { [ "_" ] octal_digit } .
hex_digits     = hex_digit { [ "_" ] hex_digit } .
```

```
42
4_2
0600
0_600
0o600
0O600       // second character is capital letter 'O'
0xBadFace
0xBad_Face
0x_67_7a_2f_cc_40_c6
170141183460469231731687303715884105727
170_141183_460469_231731_687303_715884_105727

_42         // an identifier, not an integer literal
42_         // invalid: `_` must separate successive digits
4__2        // invalid: `_` must separate successive digits
0_xBadFace  // invalid: `_` must separate successive digits
```

## Floating-point literals

A floating-point literal consists of an integer part (decimal digits), a decimal point, a fractional part (decimal digits), and an exponent part (`e` or `E` followed by an optional sign and decimal digits). One of the integer part or the fractional part may be elided; one of the decimal point or the exponent part may be elided. An exponent value exp scales the mantissa (integer and fractional part) by `10exp`.

For readability, an underscore character `'_'` may appear after a base prefix or between successive digits; such underscores do not change the literal value.

```wsn
float_lit   = decimal_digits "." [ decimal_digits ] [ exponent ] |
              decimal_digits exponent |
              "." decimal_digits [ exponent ] .
exponent    = ( "e" | "E" ) [ "+" | "-" ] decimal_digits .
```

```
0.
72.40
072.40 // == 72.40
2.71828
1.e+0
6.67428e-11
1E6
.25
.12345E+5
1_5. // == 15.0
0.15e+0_2 // == 15.0

1_.5 // invalid: `_` must separate successive digits 1._5
1._5 // invalid: `_` must separate successive digits
1.5_e1 // invalid: `_` must separate successive digits
1.5e_1 // invalid: `_` must separate successive digits
1.5e1_ // invalid: `_` must separate successive digits
```

## Character literals

A character literal represents a character constant, an integer value identifying a Unicode code point. A character literal is expressed as one or more characters enclosed in single quotes, as in `'x'` or `'\n'`. Within the quotes, any character may appear except newline and unescaped single quote. A single quoted character represents the Unicode value of the character itself, while multi-character sequences beginning with a backslash encode values in various formats.

The simplest form represents the single character within the quotes; since Ry source text is Unicode characters encoded in UTF-8, multiple UTF-8-encoded bytes may represent a single integer value. For instance, the literal `'a'` holds a single byte representing a literal `a`, Unicode `U+0061`, value `0x61`, while `'ä'` holds two bytes (`0xc3` `0xa4`) representing a literal a-dieresis, `U+00E4`, value `0xe4`.

Several backslash escapes allow arbitrary values to be encoded as ASCII text. There are four ways to represent the integer value as a numeric constant: `\x` followed by exactly two hexadecimal digits; `\u` followed by exactly four hexadecimal digits; `\U` followed by exactly eight hexadecimal digits, and a plain backslash `\` followed by exactly three octal digits. In each case the value of the literal is the value represented by the digits in the corresponding base.

Although these representations all result in an integer, they have different valid ranges. Octal escapes must represent a value between `0` and `255` inclusive. Hexadecimal escapes satisfy this condition by construction. The escapes `\u` and `\U` represent Unicode code points so within them some values are illegal, in particular those above `0x10FFFF` and surrogate halves.

After a backslash, certain single-character escapes represent special values:

```wsn
char_lit         = "'" ( unicode_value | byte_value ) "'" .
unicode_value    = unicode_char | little_u_value | big_u_value | escaped_char .
byte_value       = `\` "x" hex_digit hex_digit .
little_u_value   = `\` "u" hex_digit hex_digit hex_digit hex_digit .
big_u_value      = `\` "U" hex_digit hex_digit hex_digit hex_digit
                           hex_digit hex_digit hex_digit hex_digit .
escaped_char     = `\` ( "a" | "b" | "f" | "n" | "r" | "t" | "v" | `\` | "'" | `"` ) .
```

```
'a'
'ä'
'本'
'\t'
'\000'
'\007'
'\377'
'\x07'
'\xff'
'\u12e4'
'\U00101234'
'\''
'aa'         // illegal: too many characters
'\k'         // illegal: k is not recognized after a backslash
'\xa'        // illegal: too few hexadecimal digits
'\0'         // illegal: too few octal digits
'\400'       // illegal: octal value over 255
'\uDFFF'     // illegal: surrogate half
'\U00110000' // illegal: invalid Unicode code point
```

# Module items

## Type alias

A type alias defines a new name for an existing type. Type aliases are declared with the keyword `type`:

```wsn
TypeAlias = "type" identifier [ GenericParameters ] "=" Type .
```

For example, the following defines the type `Point` as a synonym for the type `(uint8, uint8)`, the type of pairs of unsigned 8 bit integers:

```ry
type Point = (uint8, uint8);

fun main() {
    let point: Point = (1, 2);
}
```

> [!NOTE]
> Type aliases **cannot** be used to qualify type's constructor:
>
> ```ry
> struct A(uint32);
> type B = A;
>
> fun main() {
>   let a = A(42);
>   let a = B(42); // invalid
> }
> ```

> [!NOTE]
> Type aliases **cannot** be used to qualify interfaces:
>
> ```ry
> type MyToString = ToString; // invalid
>
> fun foo[T](s: S) where S: MyToString {}
> ```

> [!NOTE]
> Type aliases **cannot** be used to call static methods on:
>
> ```ry
> type MyToString = String;
>
> fun main() {
>   let s = "hello";
>   println(MyToString.len(s)); // invalid
>   println(String.len(s)); // ok
> }
> ```

## Function

A function consists of a block, along with a name, a set of parameters, and an output type. Other than a name, all these are optional. Functions are declared with the keyword `fun`. Functions may declare a set of input variables as parameters, through which the caller passes arguments into the function, and the output type of the value the function will return to its caller on completion. If the output type is not explicitly stated, it is the unit type.

```wsn
Function = [ "pub" ] "fun" identifier "[" GenericParameters "]" "(" FunctionParameters ")"
           [ ":" Type ] [ WhereClause ] StatementsBlock
         | [ "pub" ] "fun" identifier "[" GenericParameters "]" "(" FunctionParameters ")"
           [ ":" Type ] [ WhereClause ] ";" .

FunctionParameters = [ FunctionParameter { "," FunctionParameter } [ "," ] ] .
FunctionParameter  = Pattern [ ":" ] Type
                   | "self" [ ":" Type ] .
```

Example:

```ry
fun answer_to_life_the_universe_and_everything(): uint32 {
    42
}
```

### Function parameters

Function parameters are irrefutable patterns, so any pattern that is valid in an else-less let binding is also valid as a parameter:

```ry
fn first((value, _): (i32, i32)): i32 { value }
```

If the first parameter is a `self`, this indicates that the function is a method.

```wsn
Method = Function .
```

### Generic functions

A generic function allows one or more parameterized types to appear in its signature. Each type parameter must be explicitly declared in an bracket-enclosed and comma-separated list, following the function name.

```ry
fun foo[A, B](a: A, b: B) where A: ToString { ... }
```

> [!NOTE]
> Function overloading is not supported in Ry.
>
> ```ry
> fun foo(A { a }: A) {}
> fun foo() {} // invalid
> ```

> [!NOTE]
> Functions with names `_` cannot exist, because `_` is not a valid identifier.
>
> ```ry
> fun _() { println("test") }
> ```

## Struct

```wsn
Struct        = StructStruct | TupleStruct .

StructStruct  = [ "pub" ] "struct" identifier "[" GenericParameters "]"
                [ Implements ] [ WhereClause ] "{" StructFieldList { Method } "}" .
Implements    = "implements" TypeConstructor { "," TypeConstructor } [ "," ] .
StructFields  = [ StructField { "," StructField } [ "," ] ] .
StructField   = [ "pub" ] identifier ":" type .

TupleStruct   = [ "pub" ] "struct" identifier "[" GenericParameters "]"
                "(" TupleFields ")" [ Implements ] [ WhereClause ]
                "{" { Method } "}" .
TupleFields   = [ TupleField { "," TupleField } [ "," ] ] .
TupleField    = [ "pub" ] Type .
```

A _struct_ is a nominal struct type defined with the keyword `struct`.

An example of a _struct_ module item and its use:

```ry
struct Point {
    x: int32,
    y: int32

    fun new(x: int32, y: int32): Point {
        Point { x, y }
    }
}

fun main() {
    let point = Point.new(1, 2);
}
```

A tuple struct is a nominal tuple type, also defined with the keyword `struct`. For example:

```ry
struct Point(int32, int32);

fun main() {
    let point = Point(1, 2);
    let x = point.0;
}
```

## Enumerations

```wsn
Enum         = [ "pub" ] "enum" identifier "[" GenericParameters "]"
               [ WhereClause ] "{" EnumItems { Method } "}" .
EnumItems    = [ EnumItem { "," Enumitem } [ "," ] ] .
EnumItem     = identifier
             | identifier "{" StructFields "}"
             | identifier "(" TupleFields ")" .
```

An enumeration, also referred to as an enum, is a simultaneous definition of a nominal enumerated type as well as a set of constructors, that can be used to create or pattern-match values of the corresponding enumerated type.

Enumerations are declared with the keyword `enum`.

An example of an enum item and its use:

```ry
enum Animal {
    Dog,
    Cat
}

fun main() {
    let a: Animal = Animal.Dog;
    a = Animal.Cat;
}
```

Enum constructors can have either named or unnamed fields:

```ry
enum Animal {
    Dog(String, float64),
    Cat { name: String, weight: float64 },
}

fun main() {
    let a: Animal = Animal.Dog("Cocoa", 37.2);
    a = Animal.Cat { name: "Spotty", weight: 2.7 };
}
```

In this example, `Cat` is a struct-like enum variant, whereas `Dog` is simply called an enum variant.

An enum where no constructors contain fields are called a field-less enum. For example, this is a fieldless enum:

```
enum Fieldless {
    Tuple(),
    Struct{},
    Unit,
}
```

> [!NOTE]
> Enum items don't have visibilities!
>
> ```ry
> pub enum Option[T] {
>   pub None, // invalid
>   Some(T),
> }
> ```

## Interfaces

```wsn
Interface = [ "pub" ] "interface" identifier "[" GenericParameters "]"
            [ ":" Bounds ] [ WhereClause ] "{" { Method } "}" .
```

Interfaces are declared with the keyword `interface`.

Interface methods may omit the function body by replacing it with a semicolon. This indicates that the implementation must define the method's body.
If the interface method defines a body, this definition acts as a default for any implementation which does not override it.

```ry
interface ToString {
    fun to_string(self): String;
}
```

### Generic interfaces

Type parameters can be specified for a interface to make it generic. These appear after the interface name, using the same syntax used in generic functions:

```ry
interface Iterator[T] {
    fun next(self): Option[T]
}
```

### Super interfaces

Super interfaces are interfaces that are required to be implemented for a type to implement a specific interface. Furthermore, anywhere a generic or trait object is bounded by a trait, it has access to the associated items of its super interfaces.

Super interfaces are declared by bounds on the `Self` type of an interface and transitively the super interfaces of the interfaces declared in those bounds. It is an error for an interface to to be its own super interface.

The interface with a super interface is called a sub interface of its super interface.

The following is an example of declaring `Shape` to be a super interface of `Circle`.

```ry
interface Shape { fun area(self): float64; }
interface Circle : Shape { fun radius(self): float64; }
```

And the following is the same example, except using where clauses.

```ry
interface Shape { fun area(self): float64; }
interface Circle where Self: Shape { fun radius(self): float64; }
```

This next example gives radius a default implementation using the `area` function from `Shape`.

```ry
interface Circle where Self: Shape {
    fun radius(self): float64 {
        // A = pi * r^2
        // so algebraically,
        // r = sqrt(A / pi)
        (self.area() /std.float64.consts.PI).sqrt()
    }
}
```

This next example calls a super interface method on a generic parameter.

```ry
fun print_area_and_radius[C: Circle](c: C) {
    println(c.area());
    println(c.radius());
}
```

Similarly, here is an example of calling super interface methods on interface objects.

```
let circle = circle as dyn Circle;
let nonsense = circle.radius() * circle.area();
```

### Parameter patterns

Function or method declarations without a body only allow identifier or `_` wild card patterns. All irrefutable patterns are allowed as long as there is a body:

```ry
interface T {
    fun f1((a, b): (int32, int32)) {}
    fun f2(_: (int32, int32));
    fun f3((a, b): (int32, int32)) {} // invalid
}
```

### Method visibility

All methods in public interface are public. All method in private interface are private! So `pub` in methods is invalid!

```ry
pub interface Foo {
    pub fun foo(); // invalid
}
```

## Imports

```wsn
Import     = "import" ImportPath .
ImportPath = Path [ "as" identifier ] .
```

Imports are used to qualify long names from other modules, packages, etc.

Examples:

```
import std.io;
import std.fs as stdfs;
```
