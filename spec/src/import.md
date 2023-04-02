# Import

> **<sup>Syntax</sup>**\
> _Import_ :\
> &nbsp;&nbsp; `import` _ImportPath_ `;`
> 
> _ImportPath_ :\
> &nbsp;&nbsp; [IDENTIFIER] (`.` [IDENTIFIER])<sup>\*</sup>

[IDENTIFIER]: ./identifier.md

## Examples:

```ry
import std.io.println;
import std.fs.File;
```

In this case _ImportPath_ represents absolute path to **some concrete object** like function, trait, struct, etc.

There's no way to use relative paths like this:

```ry
// main.ry
import some_another_module;

fun main() {
    some_another_module.test();
}
```

```ry
// some_another_module.ry
fun test() {

}
```

Instead you're able to do something like this:
```ry
import your_project::some_folder::some_another_module;

fun main() {
    some_another_module.test();
}
```
