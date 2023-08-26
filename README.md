<p align="center">
<img width="80%" src="./additional/icon/banner.png">
</p>

Ry is an attempt of a [teenager](https://github.com/abs0luty) to create his own programming language and write its compiler in Rust. You can view it as a toy project. I view it as a showcase of what can a single young developer achieve if he is passionate about it!

<p align="center">
<details>
  <summary>We're on Discord and Telegram!</summary>
  We have a <a href="https://discord.gg/re29xvSV2">Discord server</a> and a <a href="https://t.me/ry_lang">Telegram group</a>.
</details>
</p>

Table of contents
=================

<!--ts-->
- [Introduction](#introduction)
- [Installation](#installation)
- [Overview](#overview)
- [Roadmap](#roadmap)
<!--te-->


Introduction
=================

Ry is an open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

Example of hello world program:

```
pub fun main() {
    println("hello world");
}
```

Installation
=================

## Compiling from source code

You need to have Rust installed on your system. Then run:

```
cargo install --path crates/ry
```

Overview
=================

## Pattern matching

Ry supports matching patterns by having a `match` expression:

```
match tuple {
    (1, ..) -> {
        println("First element is 1");
    }
    (.., 'b', true) | (.., 'a', true) -> {
        println("Second element is 'b' or 'a', and third element is true");
    }
    (.., false) -> {
        println("Third element is false");
    }
    _ -> {
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

## Everything is expression

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
fun do_stuff_with(a: uint32, b: uint32, fn: fun (uint32, uint32)) {
    fn(a, b)
}
```

The language also has an analog of sum types: _enums_:

```
enum Result[T, E] {
    Ok(T),
    Err(E)

    fun ok(self): Option[T] {
        match self {
            Self.Ok(t) -> Option.Some(t),
            _ -> Option.None,
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
    let to_string = 3 as dyn ToString;

    assert(to_string.to_string() == "3");
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


Roadmap
=================

![roadmap](https://github.com/quantumatic/ry/assets/68709264/21e8b205-57b2-46d9-b82f-c3a0be30a55a)

## 0.1.0

* No runtime yet (no gc and async)
* Simple Cranelift codegen
* No standard library
* No package managment
* No documentation generation

## 0.2.0

* Official website
* Package manager backend
* Package manager website
* Package manager client in the compiler
* Caching results of type checking and compilation in a compiler
* Documentation generation
* GC

## 0.3.0

* LLVM codegen

## 0.4.0

* Official docker image
* Start of the Standard library
* LSP server written in Rust
* LSP client for neovim and vscode (for a while)

## 0.5.0

* Async runtime and more builtin types into a compiler
* More improvements into standart library

## 0.6.0 - 0.29.9

Small pathway into the release stage! A lot of stuff like metaprogramming, optimizations to std and compiler

## 1.0.0

Release (4-5 are required years to achieve that!)


