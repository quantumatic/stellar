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

> Only **absolute** paths are allowed.

Ry firstly checks if a path begins with `std` and if it does, then it searches for modules inside the folder `$RYPATH/std/`.

If it doesn't, then it goes to `$RYPATH/site-packages/`. It stores them in such format: `package_name@version_number`. Ry compiler chooses the version written down in `dependencies.toml`.
