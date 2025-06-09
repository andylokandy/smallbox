//! Space types that are used to define capacity

/// Represents 1 * usize space
pub struct S1 {
    _inner: [usize; 1],
}

/// Represents 2 * usize space
pub struct S2 {
    _inner: [usize; 2],
}

/// Represents 4 * usize space
pub struct S4 {
    _inner: [usize; 4],
}

/// Represents 8 * usize space
pub struct S8 {
    _inner: [usize; 8],
}

/// Represents 16 * usize space
pub struct S16 {
    _inner: [usize; 16],
}

/// Represents 32 * usize space
pub struct S32 {
    _inner: [usize; 32],
}

/// Represents 64 * usize space
pub struct S64 {
    _inner: [usize; 64],
}
