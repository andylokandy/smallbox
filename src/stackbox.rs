use std::cmp::Ordering;
use std::fmt;
use std::hash;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem;
use std::mem::ManuallyDrop;
use std::ops;
use std::ptr;

#[cfg(feature = "unsize")]
use std::marker::Unsize;

#[cfg(all(feature = "heap", not(feature = "std")))]
use alloc::boxed::Box;

/// A box container that only stores item on stack
pub struct StackBox<T: ?Sized, Space> {
    space: ManuallyDrop<Space>,
    #[cfg(feature = "unsize")]
    meta: usize,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized, Space> StackBox<T, Space> {
    /// Try to alloc on stack, and return Err<T>
    /// if val is larger than capacity of `Space`
    ///
    /// # Examples
    ///
    /// ```
    /// use smallbox::StackBox;
    /// use smallbox::space::S2;
    ///
    /// assert!(StackBox::<_, S2>::new([0usize; 1]).is_ok());
    /// assert!(StackBox::<_, S2>::new([0usize; 8]).is_err());
    /// ```
    #[cfg(not(feature = "unsize"))]
    pub fn new(val: T) -> Result<StackBox<T, Space>, T>
    where
        T: Sized,
    {
        if mem::size_of::<T>() > mem::size_of::<Space>() {
            Err(val)
        } else {
            unsafe {
                let mut space = ManuallyDrop::new(mem::uninitialized::<Space>());

                ptr::copy_nonoverlapping(&val, &mut space as *mut _ as *mut T, 1);
                mem::forget(val);

                Ok(StackBox {
                    space,
                    _phantom: PhantomData,
                })
            }
        }
    }

    /// Try to alloc on stack, and return Err<T>
    /// if val is larger than capacity of `Space`
    ///
    /// # Examples
    ///
    /// ```
    /// use smallbox::StackBox;
    /// use smallbox::space::S2;
    ///
    /// assert!(StackBox::<[_], S2>::new([0usize; 1]).is_ok());
    /// assert!(StackBox::<[_], S2>::new([0usize; 8]).is_err());
    /// ```
    #[cfg(feature = "unsize")]
    pub fn new<U>(val: U) -> Result<StackBox<T, Space>, U>
    where
        U: Sized + Unsize<T>,
    {
        if mem::size_of::<U>() > mem::size_of::<Space>() {
            Err(val)
        } else {
            unsafe {
                let mut space = ManuallyDrop::new(mem::uninitialized::<Space>());

                debug_assert!(mem::size_of::<*const T>() == mem::size_of::<usize>() * 2);
                let meta = {
                    let ptr = &val as *const T;
                    let ptr_ptr = &ptr as *const _ as *const usize;
                    ptr::read(ptr_ptr.offset(1))
                };

                ptr::copy_nonoverlapping(&val, &mut space as *mut _ as *mut U, 1);
                mem::forget(val);

                Ok(StackBox {
                    meta,
                    space,
                    _phantom: PhantomData,
                })
            }
        }
    }

    /// Try to change the capacity by converting into `StackBox<T>` with
    /// different Space.
    ///
    /// This may fail if the item can't fit in the new Space.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(not(feature = "unsize"))]
    /// # {
    /// use smallbox::StackBox;
    /// use smallbox::space::{S2, S4, S8};
    ///
    /// let s = StackBox::<_, S4>::new([0usize; 4]).unwrap();
    /// assert!(s.resize::<S8>().is_ok());
    ///
    /// let s = StackBox::<_, S4>::new([0usize; 4]).unwrap();
    /// assert!(s.resize::<S2>().is_err());
    /// # }
    /// ```
    pub fn resize<ToSpace>(self) -> Result<StackBox<T, ToSpace>, Self> {
        let size = mem::size_of_val::<T>(&*self);
        if size > mem::size_of::<ToSpace>() {
            Err(self)
        } else {
            unsafe {
                let mut space = ManuallyDrop::new(mem::uninitialized::<ToSpace>());

                #[cfg(feature = "unsize")]
                let meta = self.meta;

                ptr::copy_nonoverlapping(
                    &self.space as *const _ as *const u8,
                    &mut space as *mut _ as *mut u8,
                    size,
                );

                mem::forget(self);

                Ok(StackBox {
                    #[cfg(feature = "unsize")]
                    meta,
                    space,
                    _phantom: PhantomData,
                })
            }
        }
    }

    /// Get the item wrapped by standard `Box`.
    ///
    /// ```
    /// use smallbox::StackBox;
    /// use smallbox::space::S4;
    ///
    /// let small: StackBox<_, S4> = StackBox::new([0usize; 2]).unwrap();
    ///
    /// let boxed: Box<[usize; 2]> = small.to_box();
    /// # assert_eq!(boxed.len(), 2);
    /// ```
    #[cfg(all(feature = "heap", not(feature = "unsize")))]
    pub fn to_box(self) -> Box<T>
    where
        T: Sized,
    {
        unsafe {
            let mut val: T = mem::uninitialized();
            ptr::copy_nonoverlapping(&self.space as *const _ as *const T, &mut val as *mut T, 1);
            mem::forget(self);
            Box::new(val)
        }
    }

    unsafe fn as_ptr(&self) -> *const T {
        #[cfg(feature = "unsize")]
        debug_assert!(mem::size_of::<*const T>() == mem::size_of::<usize>() * 2);

        let mut ptr: *const T = mem::uninitialized();
        let ptr_ptr = &mut ptr as *mut _ as *mut usize;

        ptr::write(ptr_ptr, mem::transmute(&self.space));

        #[cfg(feature = "unsize")]
        ptr::write(ptr_ptr.offset(1), self.meta);

        ptr
    }
}

impl<T: ?Sized, Space> ops::Deref for StackBox<T, Space> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }
}

impl<T: ?Sized, Space> ops::DerefMut for StackBox<T, Space> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.as_ptr() as *const _ as *mut _) }
    }
}

impl<T: ?Sized, Space> ops::Drop for StackBox<T, Space> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(&mut **self) }
    }
}

impl<T: ?Sized + fmt::Display, Space> fmt::Display for StackBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug, Space> fmt::Debug for StackBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized, Space> fmt::Pointer for StackBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // It's not possible to extract the inner Unique directly from the Box,
        // instead we cast it to a *const which aliases the Unique
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<T: ?Sized + PartialEq, Space> PartialEq for StackBox<T, Space> {
    #[inline]
    fn eq(&self, other: &StackBox<T, Space>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
    #[inline]
    fn ne(&self, other: &StackBox<T, Space>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: ?Sized + PartialOrd, Space> PartialOrd for StackBox<T, Space> {
    #[inline]
    fn partial_cmp(&self, other: &StackBox<T, Space>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &StackBox<T, Space>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    #[inline]
    fn le(&self, other: &StackBox<T, Space>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    #[inline]
    fn ge(&self, other: &StackBox<T, Space>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    #[inline]
    fn gt(&self, other: &StackBox<T, Space>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: ?Sized + Ord, Space> Ord for StackBox<T, Space> {
    #[inline]
    fn cmp(&self, other: &StackBox<T, Space>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Eq, Space> Eq for StackBox<T, Space> {}

impl<T: ?Sized + Hash, Space> Hash for StackBox<T, Space> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::StackBox;
    use space::*;
    #[cfg(feature = "unsize")]
    use std::any::Any;

    #[cfg(not(feature = "unsize"))]
    macro_rules! Wildcard {
        () => {
            _
        };
    }

    #[cfg(feature = "unsize")]
    macro_rules! Wildcard {
        () => {
            [_]
        };
    }

    #[test]
    #[cfg(not(feature = "unsize"))]
    fn basic() {
        let stack = StackBox::<usize, S1>::new(1234usize).unwrap();
        assert!(*stack == 1234);
    }

    #[test]
    #[cfg(feature = "unsize")]
    fn basic() {
        let stack = StackBox::<Any, S1>::new(1234usize).unwrap();
        if let Some(num) = stack.downcast_ref::<usize>() {
            assert_eq!(*num, 1234);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_drop() {
        use std::cell::Cell;

        #[derive(Debug)]
        struct Struct<'a>(&'a Cell<bool>);
        impl<'a> Drop for Struct<'a> {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let flag = Cell::new(false);

        let val: StackBox<Wildcard!(), S2> = StackBox::new([Struct(&flag)]).unwrap();

        assert!(flag.get() == false);
        drop(val);
        assert!(flag.get() == true);
    }

    #[test]
    fn test_dont_drop_space() {
        struct NoDrop(S1);
        impl Drop for NoDrop {
            fn drop(&mut self) {
                unreachable!();
            }
        }

        drop(StackBox::<Wildcard!(), NoDrop>::new([true]).unwrap());
    }

    #[test]
    fn test_oversize() {
        let fit = StackBox::<Wildcard!(), S1>::new([0usize; 1]);
        let oversize = StackBox::<Wildcard!(), S1>::new([0usize; 2]);
        assert!(fit.is_ok());
        assert!(oversize.is_err());
    }

    #[test]
    fn test_resize() {
        let m = StackBox::<Wildcard!(), S4>::new([0usize; 2]).unwrap();
        let l = m.resize::<S8>().unwrap();
        let m = l.resize::<S4>().unwrap();
        let s = m.resize::<S2>().unwrap();
        let xs = s.resize::<S1>();
        assert!(xs.is_err());
    }

    #[test]
    #[cfg(not(feature = "unsize"))]
    fn test_zst() {
        let zst = StackBox::<_, S1>::new([0usize; 0]).unwrap();
        assert_eq!(*zst, [0usize; 0]);
    }

    #[test]
    #[cfg(feature = "unsize")]
    fn test_zst() {
        let zst = StackBox::<Any, S1>::new([0usize; 0]).unwrap();
        if let Some(array) = zst.downcast_ref::<[usize; 0]>() {
            assert_eq!(*array, [0usize; 0]);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_to_box() {
        let m = StackBox::<Wildcard!(), S4>::new([0usize; 2]).unwrap();
        let l = m.resize::<S8>().unwrap();
        let m = l.resize::<S4>().unwrap();
        let s = m.resize::<S2>().unwrap();
        let xs = s.resize::<S1>();
        assert!(xs.is_err());
    }
}
