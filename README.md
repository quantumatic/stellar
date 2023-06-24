<p align="center"><img width="70%" src="additional/icon/banner.png" alt="rycon"></p>

An open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

Example of hello world program:

```
pub fun main() {
    println("hello world");
}
```

Pattern matching is supported!

```
match tuple {
    #(1, ..) => {
        println("First element is 1");
    }
    #(.., 'b', true) | #(.., 'a', true) => {
        println("Second element is 'b' or 'a', and third element is true");
    }
    #(.., false) => {
        println("Third element is false");
    }
    .. => {
        println("Default case");
    }
}
```

Ry follows "most of stuff is expression" philosophy. So `if`, `match`, `while`, etc. are expressions:

```
fun factorial(n: uint32): uint32 {
    if n < 2 {
        1
    } else {
        factorial(n - 1) * n
    }
}
```

> `let` is a statement!

It can also be used in `let` statement for destructuring:

```
let Person {
    name,
    age,
} = get_person();
```

Ry also supports Rust trait system:

```
trait Foo {
    fun foo();
}

impl Foo for Bar {
    fun foo() {
        println("foo");
    }   
}
```

With generics, associated types and type aliases:

```
trait Iterator {
    type Item;

    fun next(self): Option[Self.Item];
}

trait Add[RHS = Self] {
    type Output;

    fun add(self, rhs: RHS): Self.Output;
}

type HashMapItem[K, V] = [HashMap[K, V] as IntoIterator].Item;
```

Ry also supports super traits:

```
trait DebugAndDefault: Debug + Default {}
```

and negative trait bounds:

```
trait NotDefault: Not[Default] {}
```

The language supports where clause in top level items:

```
fun foo[S](s: S) where S: ToString { ... }
```

And function types:

```
fun do_stuff_with(a: uint32, b: uint32, fn: (uint32, uint32): Unit) {
    fn(a, b)
}
```

Ry also has an analog of sum types: enums:

```
enum Result[T, E] {
    Ok(T),
    Err(E),
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

If type implements 2 traits having functions with the same names, you can use type qualification:

```
struct S;

impl S {
    fun f() { println("S"); }
}

trait T1 {
    fun f() { println("T1 f"); }
}

impl T1 for S {}

trait T2 {
    fun f() { println("T2 f"); }
}

impl T2 for S {}

fun main() {
    [S as T1].f(); // T1 f
    [S as T2].f(); // T2 f
}
```

If you want to have to deal with dynamic dispatch, you can use `Dispatcher` type:

```
fun main() {
    let iter = [1, 2, 3].into_iter() as Dispatcher[Iterator[Item = uint32]];

    assert(iter.next(), Some(1));
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

Ry is going to have documentation generation tool, package manager and more!

# Installation

## Compiling from source code

You need to have Rust installed on your system. Then run:

```
<b>cargo</b> install --path crates/ry
```

Then you're good to go coding in Ry!

# Documentation

> Not made
