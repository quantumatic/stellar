# Official Stellar 0.1.0 programming language reference

# Table of contents

- [Introduction](#introduction)
- [Notation](#notation)
- [Source code representation](#source-code-representation)
- [Characters](#characters)
- [Letters and digits](#letters-and-digits)
- [Lexical elements](#lexical-elements)
  - [Comments](#comments)
  - [Identifiers](#identifiers)
  - [Keywords](#keywords)
  - [Operators and punctuation](#operators-and-punctuation)
  - [Integer literals](#integer-literals)
  - [Floating-point literals](#floating-point-literals)
  - [Character literals](#character-literals)
- [Module items](#module-items)
  - [Type alias](#type-alias)
  - [Function](#function)
    - [Function parameters](#function-parameters)
    - [Generic functions](#generic-functions)
  - [Struct](#struct)
  - [Enumerations](#enumerations)
  - [Interfaces](#interfaces)
    - [Generic interfaces](#generic-interfaces)
    - [Super interfaces](#super-interfaces)
    - [Parameter patterns](#parameter-patterns)
    - [Method visibility](#method-visibility)
  - [Imports](#imports)
- [Expressions and statements](#statements-and-expressions)
  - [Statements](#statements)
    - [Let statements](#let-statements)
    - [Expression statements](#expression-statements)
    - [Defer statements](#defer-statements)
    - [Return statements](#return-statements)
    - [Break statements](#break-statements)
    - [Continue statements](#continue-statements)
  - [Expressions](#expressions)
    - [Literal expressions](#literal-expressions)
    - Identifier expressions
    - [Block expressions](#block-expressions)
    - [Binary expressions](#binary-expressions)
    - [Prefix expressions](#prefix-expressions)
    - [Postfix expressions](#postfix-expressions)
    - [Parenthesized expressions](#parenthesized-expressions)
    - [List expressions](#list-expressions)
    - Tuple expressions
    - Type argument qualification expressions
    - [Cast expressions](#cast-expressions)
    - [Loop expressions](#loop-expressions)
    - [While expressions](#while-expressions)
    - [If expressions](#if-expressions)
    - [Match expressions](#match-expressions)
    - [Struct expressions](#struct-expressions)
    - Field access expressions
    - [Call expressions](#call-expressions)
    - [Underscore expressions](#underscore-expressions)
    - Lambda expressions
- [Patterns](#patterns)
  - [Literal patterns](#literal-patterns)
  - [Identifier patterns](#identifier-patterns)
  - [Wildcard patterns](#wildcard-patterns)
  - [Rest patterns](#rest-patterns)
  - [Struct patterns](#struct-patterns)
  - Tuple patterns
  - List patterns
  - Grouped patterns
  - Path patterns
- Type system
  - Types
    - Boolean type
    - Numeric types
    - String type
    - Never type
    - Tuple types
    - List types
    - Struct types
    - Enumerated types
    - Function types
    - Interface object types
    - Underscore type
  - Type layout
  - Predicates
- Names
  - Namespaces
  - Scopes
  - Path
  - Visibility

# Introduction

This is the reference manual for the Stellar programming language.

Stellar is a general-purpose language designed with systems programming in mind. It is strongly typed and garbage-collected.

# Notation

The syntax is specified using a [variant](https://en.wikipedia.org/wiki/Wirth_syntax_notation) of Extended Backus-Naur Form (EBNF):

```ebnf
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

The form `a … b` represents the set of characters from a through b as alternatives. The horizontal ellipsis `…` is also used elsewhere in the spec to informally denote various enumerations or code snippets that are not further specified. The character `…` (as opposed to the three characters `...`) is not a token of the Stellar language.

# Source code representation

Source code is Unicode text encoded in UTF-8. The text is not canonicalized, so a single accented code point is distinct from the same character constructed from combining an accent and a letter; those are treated as two code points. For simplicity, this document will use the unqualified term character to refer to a Unicode code point in the source text.

Each code point is distinct; for instance, uppercase and lowercase letters are different characters.

# Characters

The following terms are used to denote specific Unicode character categories:

```ebnf
newline = /* the Unicode code point U+000A */ .
unicode_char = /* an arbitrary Unicode code point except newline */ .
unicode_letter = /* a Unicode code point categorized as "Letter" */ .
unicode_digit = /* a Unicode code point categorized as "Number, decimal digit" */ .
```

In The Unicode Standard 8.0, Section 4.5 "General Category" defines a set of character categories. Stellar treats all characters in any of the Letter categories `Lu`, `Ll`, `Lt`, `Lm`, or `Lo` as Unicode letters, and those in the Number category `Nd` as Unicode digits.

# Letters and digits

> **NOTE**:
> The underscore character `_` (`U+005F`) is **not** considered a letter.

```ebnf
letter = unicode_letter | "*" .
decimal_digit = "0" … "9" .
binary_digit = "0" | "1" .
octal_digit = "0" … "7" .
hex_digit = "0" … "9" | "A" … "F" | "a" … "f" .
```

# Lexical elements

## Comments

Comments serve as program documentation and start with `//`.

> **NOTE**:  
> A comment cannot start inside a char or string literal, or inside another comment.

## Identifiers

Identifiers name program entities such as variables and types. An identifier is a sequence of one or more letters and digits. The first character in an identifier must be a letter.

```ebnf
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

> **NOTE**:
> single `_` character is not considered an identifier, but rather a [punctuation](#operators-and-punctuation).

Some identifiers are predeclared.

## Keywords

The following keywords are reserved and may not be used as identifiers.

```
as defer else enum for false fun if pub return struct
true type let where while match import break continue
dyn loop interface implements
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

```ebnf
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

```ebnf
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

The simplest form represents the single character within the quotes; since Stellar source text is Unicode characters encoded in UTF-8, multiple UTF-8-encoded bytes may represent a single integer value. For instance, the literal `'a'` holds a single byte representing a literal `a`, Unicode `U+0061`, value `0x61`, while `'ä'` holds two bytes (`0xc3` `0xa4`) representing a literal a-dieresis, `U+00E4`, value `0xe4`.

Several backslash escapes allow arbitrary values to be encoded as ASCII text. There are four ways to represent the integer value as a numeric constant: `\x` followed by exactly two hexadecimal digits; `\u` followed by exactly four hexadecimal digits; `\U` followed by exactly eight hexadecimal digits, and a plain backslash `\` followed by exactly three octal digits. In each case the value of the literal is the value represented by the digits in the corresponding base.

Although these representations all result in an integer, they have different valid ranges. Octal escapes must represent a value between `0` and `255` inclusive. Hexadecimal escapes satisfy this condition by construction. The escapes `\u` and `\U` represent Unicode code points so within them some values are illegal, in particular those above `0x10FFFF` and surrogate halves.

After a backslash, certain single-character escapes represent special values:

```ebnf
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

```ebnf
TypeAlias = "type" identifier [ GenericParameters ] "=" Type .
```

For example, the following defines the type `Point` as a synonym for the type `(uint8, uint8)`, the type of pairs of unsigned 8 bit integers:

```stellar
type Point = (uint8, uint8);

fun main() {
    let point: Point = (1, 2);
}
```

> **NOTE**:
> Type aliases **cannot** be used to qualify type's constructor:
>
> ```stellar
> struct A(uint32);
> type B = A;
>
> fun main() {
>   let a = A(42);
>   let a = B(42); // invalid
> }
> ```

> **NOTE**:
> Type aliases **cannot** be used to qualify interfaces:
>
> ```stellar
> type MyToString = ToString; // invalid
>
> fun foo[T](s: S) where S: MyToString {}
> ```

> **NOTE**:
> Type aliases **cannot** be used to call static methods on:
>
> ```stellar
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

```ebnf
Function = [ "pub" ] "fun" identifier "[" GenericParameters "]" "(" FunctionParameters ")"
           [ ":" Type ] [ WhereClause ] StatementsBlock
         | [ "pub" ] "fun" identifier "[" GenericParameters "]" "(" FunctionParameters ")"
           [ ":" Type ] [ WhereClause ] ";" .

FunctionParameters = [ FunctionParameter { "," FunctionParameter } [ "," ] ] .
FunctionParameter  = Pattern ":" Type
                   | "self" [ ":" Type ] .
```

Example:

```stellar
fun answer_to_life_the_universe_and_everything(): uint32 {
    42
}
```

### Function parameters

Function parameters are irrefutable patterns, so any pattern that is valid in an else-less let binding is also valid as a parameter:

```stellar
fun first((value, _): (int32, int32)): int32 { value }
```

If the first parameter is a `self`, this indicates that the function is a method.

```ebnf
Method = Function .
```

### Generic functions

A generic function allows one or more parameterized types to appear in its signature. Each type parameter must be explicitly declared in an bracket-enclosed and comma-separated list, following the function name.

```stellar
fun foo[A, B](a: A, b: B) where A: ToString { ... }
```

> **NOTE**:
> Function overloading is not supported in Stellar.
>
> ```stellar
> fun foo(A { a }: A) {}
> fun foo() {} // invalid
> ```

> **NOTE**:
> Functions with names `_` cannot exist, because `_` is not a valid identifier.
>
> ```stellar
> fun _() { println("test") }
> ```

## Struct

```ebnf
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

```stellar
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

```stellar
struct Point(int32, int32) {
  fun new(x: int32, y: int32): Self {
    Self(x, y)
  }
}

fun main() {
    let point = Point.new(1, 2);
    let x = point.0;
}
```

## Enumerations

```ebnf
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

```stellar
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

```stellar
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

> **NOTE**:
> Enum items don't have visibilities!
>
> ```stellar
> pub enum Option[T] {
>   pub None, // invalid
>   Some(T),
> }
> ```

## Interfaces

```ebnf
Interface = [ "pub" ] "interface" identifier "[" GenericParameters "]"
            [ ":" Bounds ] [ WhereClause ] "{" { Method } "}" .
```

Interfaces are declared with the keyword `interface`.

Interface methods may omit the function body by replacing it with a semicolon. This indicates that the implementation must define the method's body.
If the interface method defines a body, this definition acts as a default for any implementation which does not override it.

```stellar
interface ToString {
    fun to_string(self): String;
}
```

### Generic interfaces

Type parameters can be specified for a interface to make it generic. These appear after the interface name, using the same syntax used in generic functions:

```stellar
interface Iterator[T] {
    fun next(self): Option[T]
}
```

### Super interfaces

Super interfaces are interfaces that are required to be implemented for a type to implement a specific interface. Furthermore, anywhere a generic or interface object is bounded by a interface, it has access to the associated items of its super interfaces.

Super interfaces are declared by bounds on the `Self` type of an interface and transitively the super interfaces of the interfaces declared in those bounds. It is an error for an interface to to be its own super interface.

The interface with a super interface is called a sub interface of its super interface.

The following is an example of declaring `Shape` to be a super interface of `Circle`.

```stellar
interface Shape { fun area(self): float64; }
interface Circle : Shape { fun radius(self): float64; }
```

And the following is the same example, except using where clauses.

```stellar
interface Shape { fun area(self): float64; }
interface Circle where Self: Shape { fun radius(self): float64; }
```

This next example gives radius a default implementation using the `area` function from `Shape`.

```stellar
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

```stellar
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

```stellar
interface T {
    fun f1((a, b): (int32, int32)) {}
    fun f2(_: (int32, int32));
    fun f3((a, b): (int32, int32)) {} // invalid
}
```

### Method visibility

All methods in public interface are public. All method in private interface are private! So `pub` in methods is invalid!

```stellar
pub interface Foo {
    pub fun foo(); // invalid
}
```

## Imports

```ebnf
Import     = "import" ImportPath .
ImportPath = Path [ "as" identifier ] .
```

Imports are used to qualify long names from other modules, packages, etc.

Examples:

```stellar
import std.io;
import std.fs as stdfs;
```

# Statements and expressions

## Statements

```ebnf
Statement = `;` /* empty statement */
          | LetStatement
          | ExpressionStatement
          | DeferStatement
          | ContinueStatement
          | BreakStatement .
```

### Let statements

```ebnf
LetStatement = "let" Pattern [ ":" Type ] "=" Expression ";" .
```

A let statement introduces a new set of variables, given by a pattern. The pattern is followed optionally by a type annotation and then either ends, or is followed by an initializer expression plus an optional else block. When no type annotation is given, the compiler will infer the type, or signal an error if insufficient type information is available for definite inference.

Any variables introduced by a variable declaration are visible from the point of declaration until the end of the enclosing block scope, except when they are shadowed by another variable declaration.

The pattern inside the let statement must be **irrefutable**.

```stellar
let [a, b] = [1, 2];
let (a, b, _) = (1, 2, 3);
let a = 3;
let Some(a) = foo(); // invalid
```

### Expression statements

```ebnf
ExpressionStatement = ExpressionWithoutBlock ";"
                    | ExpressionWithBlock [ ";" ] .
```

An expression statement is one that evaluates an expression and ignores its result. As a rule, an expression statement's purpose is to trigger the effects of evaluating its expression.

An expression that consists of only a block expression or control flow expression, if used in a context where a statement is permitted, can omit the trailing semicolon. This can cause an ambiguity between it being parsed as a standalone statement and as a part of another expression; in this case, it is parsed as a statement.

```stellar
v.pop();

if v.is_empty() {
    v.push(5);
} else {
    v.remove(0);
}

3 + 2; // separate expression statement
```

When the trailing semicolon is omitted, the result must be type `()`.

```stellar
fun foo(): int32 {
    if true {
        1
    } else {
        2
    }; // invalid
}
```

### Defer statements

```ebnf
DeferStatement = "defer" Expression ";" .
```

Defer statements are used to defer the execution of a function until the end of the enclosing block scope and are denoted with the keyword `defer`:

```stellar
defer file.close();
defer { println("deferred") };
```

The expression in the defer statement must be either a call or a block.

### Return statements

```ebnf
ReturnStatement = "return" Expression ";" .
```

Return statements are denoted with the keyword `return`. Evaluating a return expression moves its argument into the designated output location for the current function call, destroys the current function activation frame, and transfers control to the caller frame:

```stellar
fun factorial(n: uint32): uint32 {
    if n < 2 {
        1
    } else {
        n * factorial(n - 1)
    }
}
```

### Break statements

```ebnf
BreakStatement = "break" ";" .
```

Break statements are denoted with the keyword `break`. When break is encountered, execution of the associated loop body is immediately terminated, for example:

```stellar
fun main() {
    let a = 3;

    loop {
        a++;

        if a > 10 {
            break;
        }
    }

    println(a);
}
```

### Continue statements

```ebnf
ContinueStatement = "continue" ";" .
```

Continue statements are denoted with the keyword `continue`. When continue is encountered, the current iteration of the associated loop body is immediately terminated, returning control to the loop head. In the case of a while loop, the head is the conditional expression controlling the loop. In the case of a for loop, the head is the call-expression controlling the loop.

```stellar
fun main() {
    let a = 0;

    loop {
        a++;

        if a > 4 {
            continue;
        }

        println(a); // prints 1\n2\n3\n4\n
    }
}
```

## Expressions

### Literal expressions

```ebnf
LiteralExpression = "true" | "false" | int_lit | float_lit
                  | string_lit | char_lit .
```

A literal expression is an expression consisting of a single token, rather than a sequence of tokens, that immediately and directly denotes the value it evaluates to, rather than referring to it by name or some other evaluation rule.

A literal is a form of constant expression, so is evaluated (primarily) at compile time.

Each of the lexical literal forms described earlier can make up a literal expression, as can the keywords `true` and `false`.

### Block expressions

```ebnf
BlockExpression = StatementsBlock .
StatementsBlock = "{" Statements "}" .
Statements      = [ Statement { "," Statement } [ "," ] ] .
```

A block expression, or block, is a control flow expression and anonymous namespace scope for items and variable declarations. As a control flow expression, a block sequentially executes its component non-item declaration statements and then its final optional expression. As an anonymous namespace scope, variables declared by let statements are in scope from the next statement until the end of the block.

```stellar
let a = {
  let b = 3;
  b++;
  b
};
```

### Binary expressions

```ebnf
BinaryExpression = Expression BinaryOperator Expression ";" .
BinaryOperator   = "+=" | "+" | "-=" | "-" | "**" | "*" | "*="
                 | "/=" | "/" | "!=" | ">>" | "<<" | "<="
                 | "<" | ">=" | ">" | "==" | "=" | "|" | "&"
                 | "||" | "&&" | "|=" | "&=" | "%" | "%=" .
```

### Prefix expressions

```ebnf
PrefixExpression = PrefixOperator Expression .
PrefixOperator   = "++" | "--" .
```

### Postfix expressions

```ebnf
PostfixExpression = Expression PostfixOperator .
PostfixOperator   = "++" | "--" .
```

### Parenthesized expressions

```ebnf
ParenthesizedExpression = "(" Expression ")" .
```

A parenthesized expression wraps a single expression, evaluating to that expression. The syntax for a parenthesized expression is a `(`, then an expression, called the enclosed operand, and then a `)`.

Parenthesized expressions evaluate to the value of the enclosed operand. Parentheses can be used to explicitly modify the precedence order of subexpressions within an expression.

An example of a parenthesized expression:

```
fun main() {
    let x = 2 + 3 * 4;
    let y = (2 + 3) * 4;
}
```

### List expressions

```ebnf
ListExpression = "[" [ Expression { "," Expression } [ "," ] ] "]" .
```

List expressions construct lists. The syntax is a comma-separated list of expressions of uniform type enclosed in square brackets. This produces an list containing each of these values in the order they are written.

```stellar
let x = [1, 2, 3];
let y = ["a", "b", "c"];
let empty = [];
```

### Tuple expressions

```ebnf
TupleExpression = "(" ExpressionsInTuple ")" .
ExpressionsInTuple = /* empty */
                   | Expression ","
                   | Expression "," Expression { "," Expression } [ "," ] .
```

A tuple expression constructs tuple values.

The syntax for tuple expressions is a parenthesized, comma separated list of expressions, called the tuple initializer operands. 1-ary tuple expressions require a comma after their tuple initializer operand to be disambiguated with a parenthetical expression.

Tuple expressions are a value expression that evaluate into a newly constructed value of a tuple type. The number of tuple initializer operands is the **arity** of the constructed tuple. Tuple expressions without any tuple initializer operands produce the **unit tuple**. For other tuple expressions, the first written tuple initializer operand initializes the field `0`, and subsequent operands initializes the next highest field. For example, in the tuple expression `('a', 'b', 'c')`, `'a'` initializes the value of the field `0`, `'b'` field `1`, and `'c'` field `2`.

Examples of tuple expressions and their types:

| Expression          | Type                       |
| ------------------- | -------------------------- |
| `()`                | `()` (unit type)           |
| `(0.0, 4.5)`        | `(float64, float64)`       |
| `("x",)`            | `(String,)`                |
| `("a", (1,), true)` | `(String, (int32,), bool)` |

### Cast expressions

```ebnf
CastExpression = Expression "as" Type .
```

A type cast expression is denoted with the binary operator `as`.

Executing an `as` expression casts the value on the left-hand side to the type on the right-hand side.

An example of an `as` expression:

```
let x = 1 as bool;
```

A table of all possible type casts:

| From             | To                   | Cast                      | Example              |
| ---------------- | -------------------- | ------------------------- | -------------------- |
| Numeric type     | Numeric type         | Numeric cast              | `1 as uint64`        |
| Enumeration      | Integer type         | Enum cast                 | `Color.Red as int32` |
| `bool` or `char` | Integer type         | Primitive to integer cast | `true as int32`      |
| Integer type     | `bool` or `char`     | Integer to primitive cast | `0 as bool`          |
| `A`              | `A`                  | Type to itself cast       | `"hello" as String`  |
| `A`              | `dyn T` where `A: T` | Cast to interface object  | `1 as dyn ToString`  |

### Loop expressions

```ebnf
LoopExpression = "loop" StatementsBlock .
```

A `loop` expression repeats execution of its body continuously: `loop { println("hi!"); }`.

### While expressions

```ebnf
WhileExpression = "while" ExpressionExceptStruct StatementsBlock .
```

A while loop begins by evaluating the boolean loop conditional operand. If the loop conditional operand evaluates to true, the loop body block executes, then control returns to the loop conditional operand. If the loop conditional expression evaluates to false, the while expression completes.

An example:

```stellar
let i = 0;

while i < 10 {
    println("hello");
    i++;
}
```

### If expressions

```ebnf
IfExpression = "if" ExpressionExceptStruct StatementsBlock
               [ "else" (StatementsBlock | IfExpression) ] .
```

An `if` expression is a conditional branch in program control. The syntax of an `if` expression is a condition operand, followed by a consequent block, any number of `else if` conditions and blocks, and an optional trailing `else` block. The condition operands must have the boolean type. If a condition operand evaluates to `true`, the consequent block is executed and any subsequent else `if` or `else` block is skipped. If a condition operand evaluates to `false`, the consequent block is skipped and any subsequent `else if` condition is evaluated. If all `if` and `else if` conditions evaluate to `false` then any `else` block is executed. An if expression evaluates to the same value as the executed block, or `()` if no block is evaluated. An `if` expression must have the same type in all situations.

```stellar
if x == 4 {
    println("x is 4");
} else if x == 5 {
    println("x is 5");
} else {
    println("x is neither 4 nor 5");
}

let y = if 12 * 15 > 150 {
    "Bigger"
} else {
    "Smaller"
};
```

### Match expressions

```ebnf
MatchExpression = "match" ExpressionExceptStruct "{" [ MatchArm { "," MatchArm } [ "," ] ] "}" .
MatchArm        = Pattern "->" Expression .
```

A `match` expression branches on a pattern. The exact form of matching that occurs depends on the pattern. A `match` expression has a scrutinee expression, which is the value to compare to the patterns. The scrutinee expression and the patterns must have the same type.

An example of a match expression:

```stellar
match (1, 2, 4) {
    (2, ..) -> {
        println("First element is 2");
    }
    (_, 2, _) -> {
        println("Second element is 2");
    }
    (.., 2) -> {
        println("Third element is 2");
    }
    _ -> {
        println("I don't know where is 2");
    }
}
```

### Struct expressions

```ebnf
StructExpression           = Path "[" GenericArguments "]"
                             "{" [ StructExpressionField { "," StructExpressionField } [ "," ] ] "}" .
StructExpressionField      = identifier [ ":" Expression ] .
```

A struct expression creates a struct, enum, or union value. It consists of a path to a struct, enum variant, or union item followed by the values for the fields of the item. There are three forms of struct expressions: struct, tuple, and unit.

The following are examples of struct expressions:

```
let y = 0.0;
let a = Point { x: 10.0, y };
let u = game.User { name: "Joe", age: 35, score: 100_000 };
```

### Call expressions

```ebnf
CallExpression = Expression "(" [ Expression { "," Expression } [ "," ] ] ")" .
```

A call expression calls a function. The syntax of a call expression is an expression, called the function operand, followed by a parenthesized comma-separated list of expression, called the argument operands:

```stellar
let a = (|| "Stellar")();
let b = add(1, 2);
```

### Underscore expressions

```ebnf
UnderscoreExpression = "_" .
```

Underscore expressions, denoted with the symbol `_`, are used to signify a placeholder in a destructuring assignment. They may only appear in the left-hand side of an assignment.

```stellar
let p = (1, 2);
let a = 0;
(_, a) = p;
```

## Patterns

### Literal patterns

```ebnf
LiteralPattern = Literal | "-" float_lit | "-" int_lit .
```

Literal patterns match exactly the same value as what is created by the literal. Since negative numbers are not literals, literal patterns also accept an optional minus sign before the literal, which acts like the negation operator.

```stellar
let a = 5;

match a {
    5 => {
        println("a is 5");
    }
    2 | 4 => {
        println("a is 2 or 4");
    }
    _ => {
        println("a is neither 2 nor 4");
    }
}
```

### Identifier patterns

```ebnf
IdentifierPattern = identifier [ "@" Pattern ] .
```

Identifier patterns bind the value they match to a variable. The identifier must be unique within the pattern. The variable will shadow any variables of the same name in scope. The scope of the new binding depends on the context of where the pattern is used (such as match arm).

Patterns that consist of only an identifier and optionally a pattern that identifier is bound to.

```
let x = [2];

match x {
    a @ [_, _] => { println(a); }
    a @ [_] => { println(a); }
    _ => { println("Not matched"); }
}
```

### Wildcard patterns

```ebnf
WildcardPattern = "_" .
```

The wildcard pattern (an underscore symbol) matches any value. It is used to ignore values when they don't matter. Inside other patterns it matches a single data field (as opposed to the `..` which matches the remaining fields).

```stellar
let (a, _) = (1, x); // the x is always matched by `_`

// ignore a function/closure param
let real_part = |r: f64, _: f64| { r };

// ignore a field from a struct
let RGBA { r: red, g: green, b: blue, a: _ } = color;
```

### Rest patterns

```ebnf
RestPattern = ".." .
```

The rest pattern (the `..` token) acts as a variable-length pattern which matches zero or more elements that haven't been matched already before and after. It may only be used in tuple, tuple struct, and list patterns, and may only appear once as one of the elements in those patterns. It is also allowed in an identifier pattern for list patterns only. The rest pattern is always irrefutable.

```stellar
match list {
    [] -> println("list is empty"),
    [one] -> println("list has one element: " + one),
    [head, tail @ ..] => println("head: " + head + " tail: " + tail),
}
```

### Struct patterns

```ebnf
StructPattern      = Path "[" GenericArguments "]"
                     "{" [ StructPatternField { "," StructPatternField } [ "," ] ] "}" .
StructPatternField = identifier [ ":" Pattern ] .
```

Struct patterns match struct values that match all criteria defined by its subpatterns. They are also used to destructure a struct.

```stellar
let Person { name, age, .. } = get_person();

match s {
    Point { x: 10, y: 20 } -> (),
    Point { y: 10, x: 20 } -> (), // order doesn't matter
    Point { x: 10, .. } -> (),
    Point { .. } -> (),
}
```
