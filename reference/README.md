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

The underscore character `_` (`U+005F`) is **not** considered a letter.

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

# Identifiers

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

Some identifiers are predeclared.

# Keywords

The following keywords are reserved and may not be used as identifiers.

```
as defer else enum for fun if pub return struct type let
where while match import break continue dyn loop
interface implements
```

# Operators and punctuation

The following character sequences represent operators (including assignment operators) and punctuation:

```
-> & &= && * ** *= @ ! } ] ) : , . .. = ==
> >= << < <= - -= -- ~ != { [ ( | |= || %
%= + += ++ ? >> ; / /= ^ ^= # _
```
