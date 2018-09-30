use std::cmp::Ordering;
use std::fmt;
use std::hash::{self, Hash};
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop};
use std::ops;
use std::ptr;

#[cfg(not(feature = "std"))]
use alloc::alloc::{self, Layout};
#[cfg(feature = "std")]
use std::alloc::{self, Layout};

#[cfg(feature = "unsize")]
use std::marker::Unsize;
#[cfg(feature = "unsize")]
use std::ops::CoerceUnsized;

#[cfg(feature = "unsize")]
impl<T: ?Sized + Unsize<U>, U: ?Sized, Space> CoerceUnsized<SmallBox<U, Space>>
    for SmallBox<T, Space>
{}

#[cfg(all(feature = "heap", not(feature = "std")))]
use alloc::boxed::Box;

/// A box container that only stores item on stack
pub struct SmallBox<T: ?Sized, Space> {
    space: ManuallyDrop<Space>,
    ptr: *const T,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized, Space> SmallBox<T, Space> {
    /// Box value on stack or heap depending on its size
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::*;
    ///
    /// let small: SmallBox<_, S4> = SmallBox::new([0usize; 2]);
    /// let large: SmallBox<_, S4> = SmallBox::new([1usize; 8]);
    ///
    /// assert_eq!(small.len(), 2);
    /// assert_eq!(large[7], 1);
    ///
    /// assert!(large.heaped() == true);
    /// ```
    pub fn new(val: T) -> SmallBox<T, Space>
    where
        T: Sized,
    {
        unsafe {
            let mut space = ManuallyDrop::new(mem::uninitialized::<Space>());

            let (ptr, ptr_copy): (*const T, *mut T) = if mem::size_of::<T>()
                > mem::size_of::<Space>()
                || mem::align_of::<T>() > mem::align_of::<Space>()
            {
                // Heap
                let layout = Layout::new::<T>();
                let heap_ptr = alloc::alloc(layout) as *mut T;
                (heap_ptr, heap_ptr)
            } else {
                // Stack
                (ptr::null(), mem::transmute(&mut space))
            };

            ptr::copy_nonoverlapping(&val, ptr_copy, 1);

            mem::forget(val);

            SmallBox {
                space,
                ptr,
                _phantom: PhantomData,
            }
        }
    }

    /// Change the capacity of `SmallBox`
    ///
    /// This method may move stack-allocated data to heap
    /// if the inline space is not sufficient. Once the data
    /// is stored on heap, it'll never be moved again.
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::{S2, S4};
    ///
    /// let s: SmallBox::<_, S4> = SmallBox::new([0usize; 4]);
    /// let m: SmallBox::<_, S2> = s.resize();
    /// ```
    pub fn resize<ToSpace>(self) -> SmallBox<T, ToSpace> {
        unsafe {
            let mut space = ManuallyDrop::new(mem::uninitialized::<ToSpace>());

            let ptr = if self.ptr.is_null() {
                // original data is on stack
                let (ptr, ptr_copy): (*const T, *mut u8) = if mem::size_of_val::<T>(&*self)
                    > mem::size_of::<ToSpace>()
                    || mem::align_of_val::<T>(&*self) > mem::align_of::<ToSpace>()
                {
                    // but we have to move it to heap
                    let mut ptr = self.ptr;

                    let layout = Layout::for_value::<T>(&*self);
                    let heap_ptr = alloc::alloc(layout) as *mut u8;

                    let ptr_ptr = &mut ptr as *mut _ as *mut usize;
                    ptr_ptr.write(heap_ptr as usize);

                    (ptr, heap_ptr)
                } else {
                    // still store it on stack
                    (self.ptr, mem::transmute(&mut space))
                };

                ptr::copy_nonoverlapping(
                    &self.space as *const _ as *const u8,
                    ptr_copy,
                    mem::size_of_val::<T>(&*self),
                );

                ptr
            } else {
                // don't touch anything if the data is already on heap
                self.ptr
            };

            mem::forget(self);

            SmallBox {
                space,
                ptr,
                _phantom: PhantomData,
            }
        }
    }

    /// Returns true if the data is heap-allocated
    pub fn heaped(&self) -> bool {
        !self.ptr.is_null()
    }

    unsafe fn as_ptr(&self) -> *const T {
        let mut ptr = self.ptr;

        if ptr.is_null() {
            let ptr_ptr = &mut ptr as *mut _ as *mut usize;
            ptr_ptr.write(mem::transmute(&self.space));
        }

        ptr
    }
}

impl<T: ?Sized, Space> ops::Deref for SmallBox<T, Space> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }
}

impl<T: ?Sized, Space> ops::DerefMut for SmallBox<T, Space> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.as_ptr() as *const _ as *mut _) }
    }
}

impl<T: ?Sized, Space> ops::Drop for SmallBox<T, Space> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::for_value::<T>(&*self);
            ptr::drop_in_place(&mut **self);
            if !self.ptr.is_null() {
                alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T: ?Sized + fmt::Display, Space> fmt::Display for SmallBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug, Space> fmt::Debug for SmallBox<T, Space> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized, Space> fmt::Pointer for SmallBox<T, Space> {
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
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::SmallBox;
    use space::*;
    #[cfg(feature = "unsize")]
    use std::any::Any;

    #[test]
    fn basic() {
        let stacked: SmallBox<usize, S1> = SmallBox::new(1234usize);
        assert!(*stacked == 1234);

        let heaped: SmallBox<(usize, usize), S1> = SmallBox::new((0, 1));
        assert!(*heaped == (0, 1));
    }

    #[test]
    #[cfg(feature = "unsize")]
    fn test_downcast() {
        let stacked: SmallBox<Any, S1> = SmallBox::new(1234usize);
        if let Some(num) = stacked.downcast_ref::<usize>() {
            assert_eq!(*num, 1234);
        } else {
            unreachable!();
        }

        let heaped: SmallBox<Any, S1> = SmallBox::new([0usize, 1]);
        if let Some(array) = heaped.downcast_ref::<[usize; 2]>() {
            assert_eq!(*array, [0, 1]);
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

        let val: SmallBox<_, S2> = SmallBox::new(Struct(&flag));

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

        drop(SmallBox::<_, NoDrop>::new([true]));
    }

    #[test]
    fn test_oversize() {
        let fit = SmallBox::<_, S1>::new([0usize; 1]);
        let oversize = SmallBox::<_, S1>::new([0usize; 2]);
        assert!(!fit.heaped());
        assert!(oversize.heaped());
    }

    #[test]
    fn test_resize() {
        let m = SmallBox::<_, S4>::new([0usize; 2]);
        let l = m.resize::<S8>();
        assert!(!l.heaped());
        let m = l.resize::<S4>();
        assert!(!m.heaped());
        let s = m.resize::<S2>();
        assert!(!s.heaped());
        let xs = s.resize::<S1>();
        assert!(xs.heaped());
        let m = xs.resize::<S4>();
        assert!(m.heaped());
    }

    #[test]
    fn test_zst() {
        let zst = SmallBox::<_, S1>::new([0usize; 0]);
        assert_eq!(*zst, [0usize; 0]);
    }
}
