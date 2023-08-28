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

```
newline = /* the Unicode code point U+000A */ .
unicode_char = /* an arbitrary Unicode code point except newline */ .
unicode_letter = /* a Unicode code point categorized as "Letter" */ .
unicode_digit = /* a Unicode code point categorized as "Number, decimal digit" */ .
```

In The Unicode Standard 8.0, Section 4.5 "General Category" defines a set of character categories. Ry treats all characters in any of the Letter categories `Lu`, `Ll`, `Lt`, `Lm`, or `Lo` as Unicode letters, and those in the Number category `Nd` as Unicode digits.

# Letters and digits

> [!NOTE]
> The underscore character `_` (`U+005F`) is **not** considered a letter.

```
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

```
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

```
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

```
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

```
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

```
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
