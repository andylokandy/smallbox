//! Default size type to custom stackbox capacity
//! 
//! The Space type of `StackBox<T, Space>` is not retricted
//! to the following types, instead, it can be any sized type or 
//! even an array.
//! The `resize()` method on `StackBox<T, Space>` is used to transforms itself
//! to the one of bigger capacity
//!
//! # Example
//!
//! ```
//! use smallbox::StackBox;
//! use smallbox::space::*;
//!
//! let s = StackBox::<[usize], S8>::new([0usize; 8]).unwrap();
//! assert!(s.resize::<S16>().is_ok());
//! ```

/// Represent as 4 * usize space
pub struct S4 {
    #[used]
    inner: [usize; 4],
}

/// Represent as 8 * usize space
pub struct S8 {
    #[used]
    inner: [usize; 8],
}

/// Represent as 16 * usize space
pub struct S16 {
    #[used]
    inner: [usize; 16],
}

/// Represent as 32 * usize space
pub struct S32 {
    #[used]
    inner: [usize; 32],
}

/// Represent as 64 * usize space
pub struct S64 {
    #[used]
    inner: [usize; 64],
}