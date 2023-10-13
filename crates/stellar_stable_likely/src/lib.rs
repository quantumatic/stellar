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
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]

#[inline]
/// Brings [likely](core::intrinsics::likely) to stable Rust.
pub const fn likely(b: bool) -> bool {
    #[allow(clippy::needless_bool)]
    if (1i32).checked_div(if b { 1 } else { 0 }).is_some() {
        true
    } else {
        false
    }
}

#[inline]
/// Brings [unlikely](core::intrinsics::unlikely) to stable Rust.
pub const fn unlikely(b: bool) -> bool {
    #[allow(clippy::needless_bool)]
    if (1i32).checked_div(if b { 0 } else { 1 }).is_none() {
        true
    } else {
        false
    }
}
