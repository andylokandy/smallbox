use std::cmp::Ordering;
use std::fmt;
use std::hash;
use std::hash::Hash;
use std::ops;

#[cfg(feature = "unsize")]
use std::marker::Unsize;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use super::StackBox;

/// A box container that optimizes small item to be stored on stack
pub enum SmallBox<T: ?Sized, Space> {
    /// Stack-allocated item
    Stack(StackBox<T, Space>),
    /// Heap-allocated item
    Box(Box<T>),
}

impl<T: ?Sized, Space> SmallBox<T, Space> {
    /// Box val on stack or heap depending on its size
    ///
    /// # Examples
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::S4;
    ///
    /// let val: SmallBox<_, S4> = SmallBox::new([0usize, 1]);
    ///
    /// assert!(val.len() == 2)
    /// ```
    #[cfg(not(feature = "unsize"))]
    pub fn new(val: T) -> SmallBox<T, Space>
    where
        T: Sized,
    {
        match StackBox::new(val) {
            Ok(x) => SmallBox::Stack(x),
            Err(x) => SmallBox::Box(Box::new(x)),
        }
    }

    /// Box val on stack or heap depending on its size
    ///
    /// # Examples
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::S4;
    ///
    /// let val: SmallBox<[_], S4> = SmallBox::new([0usize, 1]);
    ///
    /// assert!(val.len() == 2)
    /// ```
    #[cfg(feature = "unsize")]
    pub fn new<U>(val: U) -> SmallBox<T, Space>
    where
        U: Sized + Unsize<T>,
    {
        match StackBox::new(val) {
            Ok(x) => SmallBox::Stack(x),
            Err(x) => SmallBox::Box(box x),
        }
    }

    /// Change the capacity by converting into `SmallBox` with
    /// different Space.
    ///
    /// This may re-store stack-allocated item onto heap.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(not(feature = "unsize"))]
    /// # {
    /// use smallbox::SmallBox;
    /// use smallbox::space::{S4, S8};
    ///
    /// let s = SmallBox::<_, S4>::new([0usize; 4]);
    /// let m = s.resize::<S8>();
    /// # }
    /// ```
    #[cfg(not(feature = "unsize"))]
    pub fn resize<ToSpace>(self) -> SmallBox<T, ToSpace>
    where
        T: Sized,
    {
        match self {
            SmallBox::Stack(x) => match x.resize() {
                Ok(x) => SmallBox::Stack(x),
                Err(x) => SmallBox::Box(x.to_box()),
            },
            SmallBox::Box(x) => SmallBox::Box(x),
        }
    }

    /// Get the item wrapped by standard `Box`.
    ///
    /// This may re-store stack-allocated item onto heap.
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::S4;
    ///
    /// let small: SmallBox<_, S4> = SmallBox::new([0usize; 2]);
    ///
    /// let boxed: Box<[usize; 2]> = small.to_box();
    /// # assert_eq!(boxed.len(), 2);
    /// ```
    #[cfg(not(feature = "unsize"))]
    pub fn to_box(self) -> Box<T>
    where
        T: Sized,
    {
        match self {
            SmallBox::Stack(x) => x.to_box(),
            SmallBox::Box(x) => x,
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

#[cfg(not(feature = "unsize"))]
impl<T: Clone, Space> Clone for SmallBox<T, Space> {
    #[inline]
    fn clone(&self) -> Self {
        SmallBox::new((**self).clone())
    }
}

impl<T: ?Sized + fmt::Display, Space> fmt::Display for SmallBox<T, Space> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug, Space> fmt::Debug for SmallBox<T, Space> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized, Space> fmt::Pointer for SmallBox<T, Space> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // It's not possible to extract the inner Unique directly from the Box,
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
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::SmallBox;
    use space::*;

    #[test]
    #[cfg(not(feature = "unsize"))]
    fn test_heap_fallback() {
        let small = SmallBox::<[usize; 2], S4>::new([1; 2]);
        let medium = SmallBox::<[usize; 4], S4>::new([1; 4]);
        let large = SmallBox::<[usize; 5], S4>::new([1; 5]);

        if let SmallBox::Box(_) = small {
            unreachable!()
        }
        if let SmallBox::Box(_) = medium {
            unreachable!()
        }
        if let SmallBox::Stack(_) = large {
            unreachable!()
        }
    }

    #[test]
    #[cfg(feature = "unsize")]
    fn test_heap_fallback() {
        let small = SmallBox::<[usize], S4>::new([1; 2]);
        let medium = SmallBox::<[usize], S4>::new([1; 4]);
        let large = SmallBox::<[usize], S4>::new([1; 5]);

        if let SmallBox::Box(_) = small {
            unreachable!()
        }
        if let SmallBox::Box(_) = medium {
            unreachable!()
        }
        if let SmallBox::Stack(_) = large {
            unreachable!()
        }
    }
}
