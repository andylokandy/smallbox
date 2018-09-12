//! `Small Box` optimization: store small item on the stack and fallback to heap for large item.
//!
//! # Usage
//!
//! First, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! smallbox = "0.5"
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
//! smallbox = { version = "0.5", features = ["unsize"] }
//! ```
//!
//! Currently `smallbox` by default links to the standard library, but if you would
//! instead like to use this crate in a `#![no_std]` situation or crate, and want to
//! opt out heap dependency and `SmallBox<T>` type, you can request this via:
//!
//! ```toml
//! [dependencies.smallbox]
//! version = "0.5"
//! features = ["unsize"]
//! default-features = false
//! ```
//!
//! Enable `heap` feature for `#![no_std]` build to link to `alloc` crate
//! and bring `SmallBox<T>` back.
//!
//! ```toml
//! [dependencies.smallbox]
//! version = "0.5"
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
//!
//!
//! - `heap`
//!   - Optional, enabled by default
//!   - Support heap fallback by including `SmallBox<T>`
//!   - If `std` feature flag is opted out, this will link
//!   `alloc` crate, and it need nightly rust for that.
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
//! Once the feature `unsize` is enabled, the item type `T` of `SmallBox` and `StackBox` can
//! and should be a unsized type, such as trait object or owned array slice.
//!
//! # Overview
//! This crate delivers two core type:
//!
//! - `SmallBox<T, Space>`: Stores `T` on heap or stack depending on the size of `T`. It takes `StackBox<T, Space>` as an varience to store small item, and then fallback to heap allocated `Box<T>` when type `T` is larger then the capacity of `Space`.
//!
//! - `StackBox<T, Space>`: Represents as a fixed-capacity allocation, and  stores item on stack.
//!
//! # Example
//!
//! Eliminate heap alloction for small items by `SmallBox`:
//!
//! ```rust
//! # #[cfg(not(feature = "unsize"))]
//! # {
//! use smallbox::SmallBox;
//! use smallbox::space::S4;
//!
//! let small: SmallBox<_, S4> = SmallBox::new([0; 2]);
//! let large: SmallBox<_, S4> = SmallBox::new([0; 32]);
//!
//! assert_eq!(small.len(), 2);
//! assert_eq!(large.len(), 32);
//!
//! match small {
//!     SmallBox::Stack(val) => assert_eq!(*val, [0; 2]),
//!     _ => unreachable!()
//! }
//!
//! match large {
//!     SmallBox::Box(val) => assert_eq!(*val, [0; 32]),
//!     _ => unreachable!()
//! }
//! # }
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
//! use smallbox::StackBox;
//! use smallbox::space::S1;
//!  
//! let val: StackBox<PartialEq<usize>, S1> = StackBox::new(5usize).unwrap();
//!  
//! assert!(*val == 5)
//! # }
//! ```
//!
//! `Any` downcasting:
//!
//! ```rust
//! # #[cfg(feature = "heap unsize")]
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
//! The capacity of `SmallBox<T, Space>` and `StackBox<T，Space>` is expressed by the size of type **`Space`**, regardless of what the `Space` actually is.
//!
//! This crate provides some spaces in module `smallbox::space`, from `S2`, 'S4' to `S64`, representing a `"n * usize"` space.
//!
//! Anyway, you can defind your own space type, such as a byte array `[u8;64]`.
//!
//! The `resize()` method on `StackBox<T, Space>` and `SmallBox<T, Space>` is used to transforms the capacity.
//!
//! ```rust
//! # #[cfg(not(feature = "unsize"))]
//! # {
//! use smallbox::SmallBox;
//! use smallbox::space::{S8, S16};
//!
//! let s = SmallBox::<[usize; 8], S8>::new([0usize; 8]);
//! let m = s.resize::<S16>();
//! # }
//! ```

#![cfg_attr(feature = "unsize", feature(unsize))]
#![cfg_attr(all(not(feature = "std"), doctest), no_std)]
#![cfg_attr(all(feature = "heap", not(feature = "std")), feature(alloc))]

#[cfg(not(any(feature = "heap", feature = "unsize")))]
compile_error!("Either feature \"heap\" or \"unsize\" must be enabled for this crate.");

#[cfg(all(feature = "heap", not(feature = "std")))]
extern crate alloc;
#[cfg(all(not(feature = "std"), doctest))]
extern crate core as std;

#[cfg(feature = "heap")]
mod smallbox;
pub mod space;
mod stackbox;

#[cfg(feature = "heap")]
pub use smallbox::SmallBox;
pub use stackbox::StackBox;
