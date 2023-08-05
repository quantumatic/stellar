//! # Branch prediction optimization
//!
//! If you are sure that one branch is more likely to be taken than the other,
//! you can use the [`likely`] and [`unlikely`].
//!
//! * This is a stable replacement for the [`intrinsics::likely`] and [`intrinsics::unlikely`].
//!
//! [`intrinsics::likely`]: https://doc.rust-lang.org/std/intrinsics/fn.likely.html
//! [`intrinsics::unlikely`]: https://doc.rust-lang.org/std/intrinsics/fn.unlikely.html

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]

#[cold]
fn cold() {}

/// The function allows to tell the compiler that the condition is likely to be
/// `true`.
#[must_use]
pub fn likely(b: bool) -> bool {
    // If `b` is `false`, it calls the `cold()` function. The purpose of calling `cold()`
    // in this case is to potentially hint to the compiler that the code path
    // where `b` is false is unlikely to be taken frequently. After that, the
    // function returns the value of `b`.
    if !b {
        cold();
    }

    b
}

/// The function allows to tell the compiler that the condition is unlikely to be
/// `true`.
#[must_use]
pub fn unlikely(b: bool) -> bool {
    // It checks if `b` is true instead. If b is true, it calls the `cold()` function.
    // Again, the purpose is to potentially hint to the compiler that the code path
    // where `b` is `true` is unlikely to be taken frequently.
    if b {
        cold();
    }

    b
}
