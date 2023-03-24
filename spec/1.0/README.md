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
