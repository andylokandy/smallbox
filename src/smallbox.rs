use std::ops;
use std::marker;
use std::hash;
use std::hash::Hash;
use std::cmp::Ordering;

use super::StackBox;

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
pub enum SmallBox<T: ?Sized> {
    Stack(StackBox<T>),
    Box(Box<T>),
}

impl<T: ?Sized> SmallBox<T> {
    /// Box val on stack or heap depending on its size
    pub fn new<U>(val: U) -> SmallBox<T>
        where U: marker::Unsize<T>
    {
        match StackBox::new(val) {
            Ok(x) => SmallBox::Stack(x),
            Err(x) => SmallBox::Box(box x),
        }
    }
}

impl<T: ?Sized> ops::Deref for SmallBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            SmallBox::Stack(ref x) => &*x,
            SmallBox::Box(ref x) => &*x,
        }
    }
}

impl<T: ?Sized> ops::DerefMut for SmallBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        match *self {
            SmallBox::Stack(ref mut x) => &mut *x,
            SmallBox::Box(ref mut x) => &mut *x,
        }
    }
}

impl<T: ?Sized + PartialEq> PartialEq for SmallBox<T> {
    #[inline]
    fn eq(&self, other: &SmallBox<T>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
    #[inline]
    fn ne(&self, other: &SmallBox<T>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd for SmallBox<T> {
    #[inline]
    fn partial_cmp(&self, other: &SmallBox<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &SmallBox<T>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    #[inline]
    fn le(&self, other: &SmallBox<T>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    #[inline]
    fn ge(&self, other: &SmallBox<T>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    #[inline]
    fn gt(&self, other: &SmallBox<T>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: ?Sized + Ord> Ord for SmallBox<T> {
    #[inline]
    fn cmp(&self, other: &SmallBox<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Eq> Eq for SmallBox<T> {}

impl<T: ?Sized + Hash> Hash for SmallBox<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}