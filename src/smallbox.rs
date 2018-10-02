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

#[cfg(feature = "coerce")]
use std::marker::Unsize;
#[cfg(feature = "coerce")]
use std::ops::CoerceUnsized;

#[cfg(feature = "coerce")]
impl<T: ?Sized + Unsize<U>, U: ?Sized, Space> CoerceUnsized<SmallBox<U, Space>>
    for SmallBox<T, Space>
{}

/// Box value on stack or heap depending on its size
///
/// This macro is similar to `SmallBox::new`, which is used to create a new `Smallbox` instance,
/// but relaxing the constraint of `T: Sized`.
/// In order to do that, this macro will check the coersion rules between type `T` and the
/// expression type. This macro invokes a complier error for any invalid type coersion.
/// 
/// You can think that it has the signature of `smallbox!<U: Sized, T: ?Sized>(val: U) -> SmallBox<T, Space>`
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate smallbox;
///
/// # fn main() {
/// use smallbox::SmallBox;
/// use smallbox::space::*;
///
/// let small: SmallBox<[usize], S4> = smallbox!([0usize; 2]);
/// let large: SmallBox<[usize], S4> = smallbox!([1usize; 8]);
///
/// assert_eq!(small.len(), 2);
/// assert_eq!(large[7], 1);
///
/// assert!(large.is_heap() == true);
/// # }
/// ```
#[macro_export]
macro_rules! smallbox {
    ( $e: expr ) => {{
        let val = $e;
        let ptr = &val as *const _;
        #[allow(unsafe_code)]
        unsafe {
            $crate::SmallBox::new_unchecked(val, ptr)
        }
    }};
}

/// An optimized box that store value on stack or heap depending on its size
pub struct SmallBox<T: ?Sized, Space> {
    space: ManuallyDrop<Space>,
    ptr: *const T,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized, Space> SmallBox<T, Space> {
    /// Box value on stack or heap depending on its size.
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
    /// assert!(large.is_heap() == true);
    /// ```
    pub fn new(val: T) -> SmallBox<T, Space>
    where
        T: Sized,
    {
        smallbox!(val)
    }

    #[doc(hidden)]
    pub unsafe fn new_unchecked<U>(val: U, ptr: *const T) -> SmallBox<T, Space>
    where
        U: Sized,
    {
        let mut space = ManuallyDrop::new(mem::uninitialized::<Space>());

        let (ptr_addr, ptr_copy): (*const u8, *mut U) = if mem::size_of::<U>()
            > mem::size_of::<Space>()
            || mem::align_of::<U>() > mem::align_of::<Space>()
        {
            // Heap
            let layout = Layout::new::<U>();
            let heap_ptr = alloc::alloc(layout);

            (heap_ptr, heap_ptr as *mut U)
        } else {
            // Stack
            (ptr::null(), mem::transmute(&mut space))
        };

        let mut ptr = ptr;
        let ptr_ptr = &mut ptr as *mut _ as *mut usize;
        ptr_ptr.write(ptr_addr as usize);

        ptr::copy_nonoverlapping(&val, ptr_copy, 1);

        mem::forget(val);

        SmallBox {
            space,
            ptr,
            _phantom: PhantomData,
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
    pub fn is_heap(&self) -> bool {
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

unsafe impl<T: ?Sized + Send, Space> Send for SmallBox<T, Space> {}
unsafe impl<T: ?Sized + Sync, Space> Sync for SmallBox<T, Space> {}

#[cfg(test)]
mod tests {
    use super::SmallBox;
    use space::*;
    use std::any::Any;

    #[test]
    fn test_basic() {
        let stacked: SmallBox<usize, S1> = SmallBox::new(1234usize);
        assert!(*stacked == 1234);

        let is_heap: SmallBox<(usize, usize), S1> = SmallBox::new((0, 1));
        assert!(*is_heap == (0, 1));
    }

    #[test]
    fn test_new_unchecked() {
        let val = [0usize, 1];
        let ptr = &val as *const _;

        unsafe {
            let stacked: SmallBox<[usize], S2> = SmallBox::new_unchecked(val, ptr);
            assert!(*stacked == [0, 1]);
            assert!(!stacked.is_heap());
        }

        let val = [0usize, 1, 2];
        let ptr = &val as *const _;

        unsafe {
            let is_heap: SmallBox<Any, S2> = SmallBox::new_unchecked(val, ptr);
            assert!(is_heap.is_heap());

            if let Some(array) = is_heap.downcast_ref::<[usize; 3]>() {
                assert_eq!(*array, [0, 1, 2]);
            } else {
                unreachable!();
            }
        }
    }

    #[test]
    #[deny(unsafe_code)]
    fn test_macro() {
        let stacked: SmallBox<Any, S1> = smallbox!(1234usize);
        if let Some(num) = stacked.downcast_ref::<usize>() {
            assert_eq!(*num, 1234);
        } else {
            unreachable!();
        }

        let is_heap: SmallBox<Any, S1> = smallbox!([0usize, 1]);
        if let Some(array) = is_heap.downcast_ref::<[usize; 2]>() {
            assert_eq!(*array, [0, 1]);
        } else {
            unreachable!();
        }
    }

    #[test]
    #[cfg(feature = "coerce")]
    fn test_coerce() {
        let stacked: SmallBox<Any, S1> = SmallBox::new(1234usize);
        if let Some(num) = stacked.downcast_ref::<usize>() {
            assert_eq!(*num, 1234);
        } else {
            unreachable!();
        }

        let is_heap: SmallBox<Any, S1> = SmallBox::new([0usize, 1]);
        if let Some(array) = is_heap.downcast_ref::<[usize; 2]>() {
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
        assert!(!fit.is_heap());
        assert!(oversize.is_heap());
    }

    #[test]
    fn test_resize() {
        let m = SmallBox::<_, S4>::new([0usize; 2]);
        let l = m.resize::<S8>();
        assert!(!l.is_heap());
        let m = l.resize::<S4>();
        assert!(!m.is_heap());
        let s = m.resize::<S2>();
        assert!(!s.is_heap());
        let xs = s.resize::<S1>();
        assert!(xs.is_heap());
        let m = xs.resize::<S4>();
        assert!(m.is_heap());
    }

    #[test]
    fn test_zst() {
        let zst = SmallBox::<_, S1>::new([0usize; 0]);
        assert_eq!(*zst, [0usize; 0]);
    }
}
