# Comments

> **<sup>Lexer</sup>**\
> COMMENT :\
> &nbsp;&nbsp; &nbsp;&nbsp; `//` (~\[`/` `!` `\n`] | `//`) ~`\n`<sup>\*</sup>\
> &nbsp;&nbsp; | `//`
>
> MODULE_DOCSTRING :\
> &nbsp;&nbsp; `//!` ~\[`\n` _IsolatedCR_]<sup>\*</sup>
>
> LOCAL_DOCSTRING :\
> &nbsp;&nbsp; `///` (~`/` ~\[`\n` _IsolatedCR_]<sup>\*</sup>)<sup>?</sup>
>
> _IsolatedCR_ :\
> &nbsp;&nbsp; _A `\r` not followed by a `\n`_

## Examples

```ry
//! Global module docstring

/// `Add` trait
pub trait Add {
    /// Function that adds two objects
    pub fun add(self: Self, b: Self): Self {
        // comment that doesn't do anything
        // much!
        self + b
    }
}
```
