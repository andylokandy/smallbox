#[cfg(not(feature="std"))]
use alloc::boxed::Box;

use std::ops;
use std::fmt;
use std::hash;
#[cfg(feature = "nightly")]
use std::marker::Unsize;
use std::hash::Hash;
use std::cmp::Ordering;

use super::StackBox;
use super::space::S2;

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
pub enum SmallBox<T: ?Sized, Space = S2> {
    Stack(StackBox<T, Space>),
    Box(Box<T>),
}

impl<T: ?Sized, Space> SmallBox<T, Space> {
    /// Box val on stack or heap depending on its size
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::*;
    ///
    /// let tiny: SmallBox<[u64], S4> = SmallBox::new([0; 2]);
    /// let big: SmallBox<[u64], S4> = SmallBox::new([1; 8]);
    ///
    /// assert_eq!(tiny.len(), 2);
    /// assert_eq!(big[7], 1);
    ///
    /// match tiny {
    ///     SmallBox::Stack(val) => assert_eq!(*val, [0; 2]),
    ///     _ => unreachable!()
    /// }
    ///
    /// match big {
    ///     SmallBox::Box(val) => assert_eq!(*val, [1; 8]),
    ///     _ => unreachable!()
    /// }
    /// ```
    #[cfg(feature = "nightly")]
    pub fn new<U>(val: U) -> SmallBox<T, Space>
        where U: Unsize<T>
    {
        match StackBox::new(val) {
            Ok(x) => SmallBox::Stack(x),
            Err(x) => SmallBox::Box(box x),
        }
    }

    /// Box val on stack or heap depending on its size
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::*;
    ///
    /// let tiny: SmallBox<[u64], S4> = SmallBox::new([0; 2]);
    /// let big: SmallBox<[u64], S4> = SmallBox::new([1; 8]);
    ///
    /// assert_eq!(tiny.len(), 2);
    /// assert_eq!(big[7], 1);
    ///
    /// match tiny {
    ///     SmallBox::Stack(val) => assert_eq!(*val, [0; 2]),
    ///     _ => unreachable!()
    /// }
    ///
    /// match big {
    ///     SmallBox::Box(val) => assert_eq!(*val, [1; 8]),
    ///     _ => unreachable!()
    /// }
    /// ```
    #[cfg(not(feature = "nightly"))]
    pub fn new(val: T) -> SmallBox<T, Space>
        where T: Sized
    {
        match StackBox::new(val) {
            Ok(x) => SmallBox::Stack(x),
            Err(x) => SmallBox::Box(Box::new(x)),
        }
    }

    /// Try to transforms to the `SmallBox<T>` of bigger capacity,
    /// and return `Err` when target capacity is smaller.
    /// Note that this method will always success
    /// when the allocation is on heap.
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::*;
    ///
    /// let s = SmallBox::<[usize], S8>::new([0usize; 4]);
    /// assert!(s.resize::<S16>().is_ok());
    ///
    /// let s = SmallBox::<[usize], S8>::new([0usize; 4]);
    /// assert!(s.resize::<S4>().is_err());
    /// ```
    pub fn resize<ToSpace>(self) -> Result<SmallBox<T, ToSpace>, Self> {
        match self {
            SmallBox::Stack(x) => x.resize().map(SmallBox::Stack).map_err(SmallBox::Stack),
            SmallBox::Box(x) => Ok(SmallBox::Box(x)),
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
