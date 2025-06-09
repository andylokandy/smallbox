//! **Small Box** optimization: store small items on stack and fall back to heap for large items.
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
//! If you want this crate to work with dynamically-sized types, you can enable it via:
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
//!   - If the `std` feature flag is disabled, the `alloc` crate will be linked, which requires
//!     nightly Rust.
//!
//! - `coerce`
//!   - Optional
//!   - Requires nightly Rust
//!   - Allow automatic coercion from sized `SmallBox` to unsized `SmallBox`.
//!
//! - `nightly`
//!   - Optional
//!   - Enables `coerce`
//!   - Requires nightly Rust
//!   - Uses strict provenance operations to be compatible with `miri`
//!
//! # Unsized Types
//!
//! There are two ways to create an unsized `SmallBox`: using the `smallbox!()` macro or coercing
//! from a sized `SmallBox` instance.
//!
//! Using the `smallbox!()` macro is the only option on stable Rust. This macro will check the types
//! of the expression and the expected type `T`. For any invalid type coercions, this macro triggers
//! a compiler error.
//!
//! Once the `coerce` feature is enabled, sized `SmallBox<T>` can be coerced into `SmallBox<T:
//! ?Sized>` if necessary.
//!
//! # Example
//!
//! Eliminate heap allocation for small items with `SmallBox`:
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
//! assert_eq!(small.is_heap(), false);
//! assert_eq!(large.is_heap(), true);
//! ```
//!
//! ## Unsized Types
//!
//! Construct with the `smallbox!()` macro:
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
//! With the `coerce` feature:
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
//!
//! use smallbox::SmallBox;
//! use smallbox::space::S2;
//!
//! let num: SmallBox<dyn Any, S2> = smallbox!(1234u32);
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
//! The capacity is expressed by the size of the type parameter `Space`,
//! regardless of what the `Space` actually is.
//!
//! The crate provides some space types in the `smallbox::space` module,
//! from `S1`, `S2`, `S4` to `S64`, representing `"n * usize"` spaces.
//!
//! You can also define your own space type,
//! such as a byte array `[u8; 64]`.
//! Please note that space alignment is also important. If the alignment
//! of the space is smaller than the alignment of the value, the value
//! will be stored on the heap.
#![cfg_attr(feature = "nightly", feature(strict_provenance, set_ptr_value))]
#![cfg_attr(feature = "coerce", feature(unsize, coerce_unsized))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::as_conversions)]

extern crate alloc;

mod smallbox;
pub mod space;
mod sptr;

pub use crate::smallbox::SmallBox;
