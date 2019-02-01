//! Space types that is used to define capacity

/// Represent 1 * usize space
pub struct S1 {
    _inner: [usize; 1],
}

/// Represent 2 * usize space
pub struct S2 {
    _inner: [usize; 2],
}

/// Represent 4 * usize space
pub struct S4 {
    _inner: [usize; 4],
}

/// Represent 8 * usize space
pub struct S8 {
    _inner: [usize; 8],
}

/// Represent 16 * usize space
pub struct S16 {
    _inner: [usize; 16],
}

/// Represent 32 * usize space
pub struct S32 {
    _inner: [usize; 32],
}

/// Represent 64 * usize space
pub struct S64 {
    _inner: [usize; 64],
}
