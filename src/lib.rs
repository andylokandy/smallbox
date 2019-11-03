//! `Small Box` optimization: store small item on stack and fallback to heap for large item.
//!
//! # Usage
//!
//! First, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! smallbox = "0.8"
//! ```
//!
//! Next, add this to your crate root:
//!
//! ```rust
//! extern crate smallbox;
//! ```
//!
//! If you want this crate to work with dynamic-sized type, you can request it via:
//!
//! ```toml
//! [dependencies]
//! smallbox = { version = "0.8", features = ["coerce"] }
//! ```
//!
//! Currently `smallbox` by default links to the standard library, but if you would
//! instead like to use this crate in a `#![no_std]` situation or crate, you can request this via:
//!
//! ```toml
//! [dependencies.smallbox]
//! version = "0.8"
//! features = ["coerce"]
//! default-features = false
//! ```
//!
//!
//! # Feature Flags
//!
//! This crate has the following cargo feature flags:
//!
//! - `std`
//!   - Optional, enabled by default
//!   - Use libstd
//!   - If `std` feature flag is opted out, `alloc` crate
//!     will be linked, which requires nightly rust.
//!
//! - `coerce`
//!   - Optional
//!   - Require nightly rust
//!   - Allow automatic coersion from sized `SmallBox` to unsized `SmallBox`.
//!
//!
//! # Unsized Type
//!
//! There are two ways to have an unsized `SmallBox`: Using `smallbox!()` macro or coercing from a sized `SmallBox` instance.
//!
//! Using the `smallbox!()` macro is the only option on stable rust. This macro will check the types of the expression and
//! the expected type `T`. For any invalid type coersions, this macro invokes a compiler error.
//!
//! Once the feature `coerce` is enabled, sized `SmallBox<T>` can be coerced into `SmallBox<T: ?Sized>` if necessary.
//!
//! # Example
//!
//! Eliminate heap alloction for small items by `SmallBox`:
//!
//! ```rust
//! use smallbox::SmallBox;
//! use smallbox::space::S4;
//!
//! let small: SmallBox<_, S4> = SmallBox::new([0; 2]);
//! let large: SmallBox<_, S4> = SmallBox::new([0; 32]);
//!
//! assert_eq!(small.len(), 2);
//! assert_eq!(large.len(), 32);
//!
//! assert_eq!(*small, [0; 2]);
//! assert_eq!(*large, [0; 32]);
//!
//! assert!(small.is_heap() == false);
//! assert!(large.is_heap() == true);
//! ```
//!
//! ## Unsized type
//!
//! Construct with `smallbox!()` macro:
//!
//! ```rust
//! #[macro_use]
//! extern crate smallbox;
//!
//! # fn main() {
//! use smallbox::SmallBox;
//! use smallbox::space::*;
//!
//! let array: SmallBox<[usize], S2> = smallbox!([0usize, 1]);
//!
//! assert_eq!(array.len(), 2);
//! assert_eq!(*array, [0, 1]);
//! # }
//! ```
//!
//! With `coerce` feature:
//!
//! ```rust
//! # #[cfg(feature = "coerce")]
//! # {
//! use smallbox::SmallBox;
//! use smallbox::space::*;
//!  
//! let array: SmallBox<[usize], S2> = SmallBox::new([0usize, 1]);
//!
//! assert_eq!(array.len(), 2);
//! assert_eq!(*array, [0, 1]);
//! # }
//! ```
//!
//! `Any` downcasting:
//!
//! ```rust
//! #[macro_use]
//! extern crate smallbox;
//!
//! # fn main() {
//! use std::any::Any;
//! use smallbox::SmallBox;
//! use smallbox::space::S2;
//!
//! let num: SmallBox<Any, S2> = smallbox!(1234u32);
//!
//! if let Some(num) = num.downcast_ref::<u32>() {
//!     assert_eq!(*num, 1234);
//! } else {
//!     unreachable!();
//! }
//! # }
//! ```
//!
//!
//! # Capacity
//!
//! The capacity is expressed by the size of type parameter `Space`,
//! regardless of what actually the `Space` is.
//!
//! The crate provides some spaces in module `smallbox::space`,
//! from `S1`, `S2`, `S4` to `S64`, representing `"n * usize"` spaces.
//! Anyway, you can defind your own space type
//! such as byte array `[u8; 64]`.

#![cfg_attr(feature = "coerce", feature(unsize, coerce_unsized))]
#![cfg_attr(not(feature = "std"), feature(cfg_doctest))]
#![cfg_attr(all(not(feature = "std"), doctest), no_std)]

#[cfg(not(any(feature = "std", doctest)))]
extern crate alloc;
#[cfg(all(not(feature = "std"), doctest))]
extern crate core as std;

mod smallbox;
pub mod space;

pub use crate::smallbox::SmallBox;
