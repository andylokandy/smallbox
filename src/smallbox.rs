use std::ops;
use std::marker;

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
            SmallBox::Stack(ref x) => unsafe { &*x.as_fat_ptr() },
            SmallBox::Box(ref x) => &*x,
        }
    }
}

impl<T: ?Sized> ops::DerefMut for SmallBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        match *self {
            SmallBox::Stack(ref mut x) => unsafe { &mut *(x.as_fat_ptr() as *mut T) },
            SmallBox::Box(ref mut x) => &mut *x,
        }
    }
}