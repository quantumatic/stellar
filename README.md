# Ry programming language

<p align="center">
<details>
  <summary>We're on Discord and Telegram!</summary>
  We have a <a href="https://discord.gg/re29xvSV2">Discord server</a> and a <a href="https://t.me/ry_lang">Telegram group</a>.
</details>
</p>

## Table of contents

- [Introduction](#Introduction)
- [Installation](#Installation)
- [Overview](#Overview)

# Introduction

Ry is an open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

Example of hello world program:

```
pub fun main() {
    println("hello world");
}
```

# Installation

## Compiling from source code

You need to have Rust installed on your system. Then run:

```
cargo install --path crates/ry
```

# Overview

# Pattern matching

Ry supports matching patterns by having a `match` expression:

```
match tuple {
    (1, ..) => {
        println("First element is 1");
    }
    (.., 'b', true) | (.., 'a', true) => {
        println("Second element is 'b' or 'a', and third element is true");
    }
    (.., false) => {
        println("Third element is false");
    }
    .. => {
        println("Default case");
    }
}
```

Pattern matching can also be used in `let` statement for destructuring:

```
let Person {
    name,
    age,
} = get_person();
```

# Everything is expression

Ry follows "everything is expression" philosophy. So `if`, `match`, `while`, etc. are expressions:

```
fun factorial(n: uint32): uint32 {
    if n < 2 {
        1
    } else {
        factorial(n - 1) * n
    }
}
```

Ry supports function types:

```
fun do_stuff_with(a: uint32, b: uint32, fn: (uint32, uint32): ()) {
    fn(a, b)
}
```

The language also has an analog of sum types: _enums_:

```
enum Result[T, E] implements ToString {
    Ok(T)
    Err(E)

    fun to_string(self): String where T: ToString, E: ToString {
        match self {
            Self.Ok(t) => t.to_string(),
            Self.Err(e) => e.to_string(),
        }
    }
}
```

and error propagation:

```
fun safe_div[T](a: T, b: T): Option[T] where T: Numeric {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

fun main() {
    let a = safe_div(1, 1)?;
    assert(a == 1);

    safe_div(1, 0)?;
}
```

If you want to have to deal with dynamic dispatch, you can use `dyn` type:

```
fun main() {
    let iter = [1, 2, 3].into_iter() as dyn Iterator[uint32];

    assert(iter.next() == Some(1));
}
```

Ry also supports tuple-like struct types and enum items:

```
pub struct MyStringWrapper(pub String);
```

You can access their inner values using pattern matching:

```
let MyStringWrapper(str) = wrapper;
println(str);
```

