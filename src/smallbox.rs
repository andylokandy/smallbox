#[cfg(not(feature="std"))]
use alloc::boxed::Box;

use std::ops;
use std::marker;
use std::fmt;
use std::hash;
use std::hash::Hash;
use std::cmp::Ordering;

use super::StackBox;
use super::space::U4;

/// Stack allocation with heap fallback
///
/// # Examples
///
/// ```
/// use smallbox::SmallBox;
///
/// let val: SmallBox<PartialEq<usize>> = SmallBox::new(5usize);
///
/// assert!(*val == 5)
/// ```
pub enum SmallBox<T: ?Sized, Space = U4> {
    Stack(StackBox<T, Space>),
    Box(Box<T>),
}

impl<T: ?Sized, Space> SmallBox<T, Space> {
    /// Box val on stack or heap depending on its size
    pub fn new<U>(val: U) -> SmallBox<T, Space>
        where U: marker::Unsize<T>
    {
        match StackBox::new(val) {
            Ok(x) => SmallBox::Stack(x),
            Err(x) => SmallBox::Box(box x),
        }
    }
}

impl<T: ?Sized, Space> ops::Deref for SmallBox<T, Space> {
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            SmallBox::Stack(ref x) => &*x,
            SmallBox::Box(ref x) => &*x,
        }
    }
}

impl<T: ?Sized, Space> ops::DerefMut for SmallBox<T, Space> {
    fn deref_mut(&mut self) -> &mut T {
        match *self {
            SmallBox::Stack(ref mut x) => &mut *x,
            SmallBox::Box(ref mut x) => &mut *x,
        }
    }
}


impl<T: fmt::Display + ?Sized, Space> fmt::Display for SmallBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug + ?Sized, Space> fmt::Debug for SmallBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized, Space> fmt::Pointer for SmallBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // It's not possible to extract the inner Uniq directly from the Box,
        // instead we cast it to a *const which aliases the Unique
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<T: ?Sized + PartialEq, Space> PartialEq for SmallBox<T, Space> {
    #[inline]
    fn eq(&self, other: &SmallBox<T, Space>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
    #[inline]
    fn ne(&self, other: &SmallBox<T, Space>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: ?Sized + PartialOrd, Space> PartialOrd for SmallBox<T, Space> {
    #[inline]
    fn partial_cmp(&self, other: &SmallBox<T, Space>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &SmallBox<T, Space>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    #[inline]
    fn le(&self, other: &SmallBox<T, Space>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    #[inline]
    fn ge(&self, other: &SmallBox<T, Space>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    #[inline]
    fn gt(&self, other: &SmallBox<T, Space>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: ?Sized + Ord, Space> Ord for SmallBox<T, Space> {
    #[inline]
    fn cmp(&self, other: &SmallBox<T, Space>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Eq, Space> Eq for SmallBox<T, Space> {}

impl<T: ?Sized + Hash, Space> Hash for SmallBox<T, Space> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}