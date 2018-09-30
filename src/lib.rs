//! `Small Box` optimization: store small item on the stack or fallback to heap for large item.
//!
//! # Usage
//!
//! First, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! smallbox = "0.6"
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
//! smallbox = { version = "0.6", features = ["unsize"] }
//! ```
//!
//! Currently `smallbox` by default links to the standard library, but if you would
//! instead like to use this crate in a `#![no_std]` situation or crate, and want to
//! opt out heap dependency and `SmallBox<T>` type, you can request this via:
//!
//! ```toml
//! [dependencies.smallbox]
//! version = "0.6"
//! features = ["unsize"]
//! default-features = false
//! ```
//!
//! Enable `heap` feature for `#![no_std]` build to link to `alloc` crate
//! and bring `SmallBox<T>` back.
//!
//! ```toml
//! [dependencies.smallbox]
//! version = "0.6"
//! features = ["unsize", "heap"]
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
//!     will be linked, which would require nightly rust.
//!
//! - `unsize`
//!   - Optional
//!   - Require nightly rust
//!   - Enable support for `DST` (dynamic-sized type).
//!
//!
//! # Stable Rust
//!
//! The only possible way to use this crate on stable rust is to use the default feature flag, which means you can't use it in `no_std`
//! environment or use it with `DST` (dynamic-sized type).
//!
//! # Unsized Type
//!
//! Once the feature `unsize` is enabled, sized `SmallBox<T>` can be coerced into `SmallBox<T: ?Sized>` if necessary.
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
//! assert!(small.heaped() == false);
//! assert!(large.heaped() == true);
//! ```
//!
//! ## DST
//!
//! The following examples requires `unsize` feature flag enabled.
//!
//! Trait object dynamic-dispatch:
//!
//! ```rust
//! # #[cfg(feature = "unsize")]
//! # {
//! use smallbox::SmallBox;
//! use smallbox::space::S1;
//!  
//! let val: SmallBox<PartialEq<usize>, S1> = SmallBox::new(5usize);
//!  
//! assert!(*val == 5)
//! # }
//! ```
//!
//! `Any` downcasting:
//!
//! ```rust
//! # #[cfg(feature = "unsize")]
//! # {
//! use std::any::Any;
//! use smallbox::SmallBox;
//! use smallbox::space::S2;
//!
//! let num: SmallBox<Any, S2> = SmallBox::new(1234u32);
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
//! The capacity of `SmallBox<T, Space>` is expressed by the size of type parameter **`Space`**, 
//! regardless of what the `Space` actually is.
//!
//! This crate provides some spaces in module `smallbox::space`, 
//! from `S1`, `S2`, `S4` to `S64`, representing `"n * usize"` spaces.
//!
//! Anyway, you can defind your own space type, 
//! such as a byte array `[u8; 64]`.
//!
//! The `resize()` method on `SmallBox` is used to change its capacity.
//!
//! ```rust
//! use smallbox::SmallBox;
//! use smallbox::space::{S8, S16};
//!
//! let s: SmallBox::<_, S8> = SmallBox::new([0usize; 8]);
//! let m: SmallBox<_, S16> = s.resize();
//! ```

#![cfg_attr(feature = "unsize", feature(unsize, coerce_unsized))]
#![cfg_attr(all(not(feature = "std"), doctest), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(all(not(feature = "std"), doctest))]
extern crate core as std;

mod smallbox;
pub mod space;

pub use smallbox::SmallBox;
