//! # SmallBox: Space-Efficient Smart Pointers
//!
//! [`SmallBox`] stores small values inline and uses the heap when needed.
//! Keeping values inside the [`SmallBox`] itself can avoid heap allocations.
//!
//! ## Core Concept
//!
//! The `Space` parameter provides storage inside the [`SmallBox`]. Values created with
//! [`SmallBox::new`] or [`smallbox!`] are stored there when their size and alignment fit,
//! and use the heap otherwise.
//!
//! ## Quick Start
//!
//! Add SmallBox to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! smallbox = "0.8"
//! ```
//!
//! Basic usage:
//!
//! ```rust
//! use smallbox::SmallBox;
//! use smallbox::space::S4;
//!
//! // Small values are stored inline
//! let small: SmallBox<[u32; 2], S4> = SmallBox::new([1, 2]);
//! assert!(!small.is_heap());
//!
//! // Large values automatically use heap allocation
//! let large: SmallBox<[u32; 32], S4> = SmallBox::new([0; 32]);
//! assert!(large.is_heap());
//!
//! // Use like a regular Box
//! println!("Values: {:?} and length {}", *small, large.len());
//! ```
//!
//! ## Configuration
//!
//! ### Feature Flags
//!
//! - **`std`** (enabled by default)
//!   - Links to the standard library
//!   - Disable for `#![no_std]` environments: `default-features = false`
//!
//! - **`coerce`** (optional, requires nightly)
//!   - Enables automatic coercion from `SmallBox<T>` to `SmallBox<dyn Trait>`
//!   - Allows more ergonomic usage with trait objects
//!
//! ### No-std Usage
//!
//! SmallBox works in `#![no_std]` environments:
//!
//! ```toml
//! [dependencies]
//! smallbox = { version = "0.8", default-features = false }
//! ```
//!
//! ### Custom Space Types
//!
//! Define custom capacities for specific needs:
//!
//! ```rust
//! use smallbox::SmallBox;
//!
//! // Custom 128-byte capacity
//! type MySpace = [u8; 128];
//! type MySmallBox<T> = SmallBox<T, MySpace>;
//!
//! let value: MySmallBox<[u8; 100]> = SmallBox::new([0; 100]);
//! assert!(!value.is_heap()); // Fits in custom space
//! ```
//!
//! The value must fit both the size and alignment of the chosen space to be stored inline.
//!
//! ## Working with Unsized Types
//!
//! SmallBox supports unsized types like trait objects and slices through two approaches:
//!
//! ### 1. Using the `smallbox!()` Macro (Stable Rust)
//!
//! ```rust
//! #[macro_use]
//! extern crate smallbox;
//!
//! # fn main() {
//! use std::any::Any;
//!
//! use smallbox::SmallBox;
//! use smallbox::space::S4;
//!
//! // Create trait objects
//! let values: Vec<SmallBox<dyn Any, S4>> = vec![
//!     smallbox!(42u32),
//!     smallbox!("hello world"),
//!     smallbox!(vec![1, 2, 3]),
//! ];
//!
//! // Create slices
//! let slice: SmallBox<[i32], S4> = smallbox!([1, 2, 3, 4]);
//! assert_eq!(slice.len(), 4);
//! # }
//! ```
//!
//! ### 2. Automatic Coercion (Nightly with `coerce` feature)
//!
//! ```rust
//! # #[cfg(feature = "coerce")]
//! # {
//! use std::any::Any;
//!
//! use smallbox::SmallBox;
//! use smallbox::space::S4;
//!
//! // Automatic coercion to trait objects
//! let num: SmallBox<dyn Any, S4> = SmallBox::new(42u32);
//! let text: SmallBox<dyn Any, S4> = SmallBox::new("hello");
//!
//! // Automatic coercion to slices
//! let slice: SmallBox<[i32], S4> = SmallBox::new([1, 2, 3, 4]);
//! # }
//! ```
//!
//! ## Advanced Usage
//!
//! ### Type Downcasting
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
//! let value: SmallBox<dyn Any, S2> = smallbox!(42u32);
//!
//! match value.downcast::<u32>() {
//!     Ok(num) => println!("Got number: {}", *num),
//!     Err(original) => println!("Not a u32, got: {:?}", original.type_id()),
//! }
//! # }
//! ```
//!
//! ### Interoperability with `Box`
//!
//! Convert between [`SmallBox`] and [`Box`] when needed:
//!
//! ```rust
//! use smallbox::SmallBox;
//! use smallbox::space::S4;
//!
//! // Box -> SmallBox (data stays on heap)
//! let boxed = Box::new([1, 2, 3, 4]);
//! let small_box: SmallBox<_, S4> = SmallBox::from_box(boxed);
//! assert!(small_box.is_heap());
//!
//! // SmallBox -> Box (data moves to heap if needed)
//! let back_to_box: Box<[i32; 4]> = SmallBox::into_box(small_box);
//! ```

#![cfg_attr(feature = "nightly", feature(strict_provenance, set_ptr_value))]
#![cfg_attr(feature = "coerce", feature(unsize, coerce_unsized))]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(stable_features)]
#![deny(missing_docs)]
#![deny(clippy::as_conversions)]

extern crate alloc;

mod smallbox;
pub mod space;
mod sptr;

pub use crate::smallbox::SmallBox;
