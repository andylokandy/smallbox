use core::any::Any;
use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::hash::{self};
use core::marker::PhantomData;
#[cfg(feature = "coerce")]
use core::marker::Unsize;
use core::mem::ManuallyDrop;
use core::mem::MaybeUninit;
use core::mem::{self};
use core::ops;
#[cfg(feature = "coerce")]
use core::ops::CoerceUnsized;
use core::ptr;

use ::alloc::alloc;
use ::alloc::alloc::Layout;

use crate::sptr;

#[cfg(feature = "coerce")]
impl<T: ?Sized + Unsize<U>, U: ?Sized, Space> CoerceUnsized<SmallBox<U, Space>>
    for SmallBox<T, Space>
{
}

/// Box value on stack or on heap depending on its size
///
/// This macro is similar to `SmallBox::new`, which is used to create a new `Smallbox` instance,
/// but relaxing the constraint `T: Sized`.
/// In order to do that, this macro will check the coersion rules from type `T` to
/// the target type. This macro will invoke a complie-time error on any invalid type coersion.
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
/// use smallbox::space::*;
/// use smallbox::SmallBox;
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
        let ptr = ::core::ptr::addr_of!(val);
        #[allow(unsafe_code)]
        unsafe {
            $crate::SmallBox::new_unchecked(val, ptr)
        }
    }};
}

/// An optimized box that store value on stack or on heap depending on its size
pub struct SmallBox<T: ?Sized, Space> {
    space: MaybeUninit<Space>,
    ptr: *mut T,
    _phantom: PhantomData<T>,
}

impl<T: Default, Space> Default for SmallBox<T, Space> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized, Space> SmallBox<T, Space> {
    /// Box value on stack or on heap depending on its size.
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::space::*;
    /// use smallbox::SmallBox;
    ///
    /// let small: SmallBox<_, S4> = SmallBox::new([0usize; 2]);
    /// let large: SmallBox<_, S4> = SmallBox::new([1usize; 8]);
    ///
    /// assert_eq!(small.len(), 2);
    /// assert_eq!(large[7], 1);
    ///
    /// assert!(large.is_heap() == true);
    /// ```
    #[inline(always)]
    pub fn new(val: T) -> SmallBox<T, Space>
    where T: Sized {
        smallbox!(val)
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn new_unchecked<U>(val: U, ptr: *const T) -> SmallBox<T, Space>
    where U: Sized {
        let val = ManuallyDrop::new(val);
        Self::new_copy(&val, ptr)
    }

    /// Change the capacity of `SmallBox`.
    ///
    /// This method may move stack-allocated data from stack to heap
    /// when inline space is not sufficient. And once the data
    /// is moved to heap, it'll never be moved again.
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::space::S2;
    /// use smallbox::space::S4;
    /// use smallbox::SmallBox;
    ///
    /// let s: SmallBox<_, S4> = SmallBox::new([0usize; 4]);
    /// let m: SmallBox<_, S2> = s.resize();
    /// ```
    pub fn resize<ToSpace>(self) -> SmallBox<T, ToSpace> {
        let this = ManuallyDrop::new(self);

        if this.is_heap() {
            // don't change anything if data is already on heap
            let space = MaybeUninit::<ToSpace>::uninit();
            SmallBox {
                space,
                ptr: this.ptr,
                _phantom: PhantomData,
            }
        } else {
            let val: &T = &this;
            unsafe { SmallBox::<T, ToSpace>::new_copy(val, sptr::from_ref(val)) }
        }
    }

    /// Returns true if data is allocated on heap.
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::space::S1;
    /// use smallbox::SmallBox;
    ///
    /// let stacked: SmallBox<usize, S1> = SmallBox::new(0usize);
    /// assert!(!stacked.is_heap());
    ///
    /// let heaped: SmallBox<(usize, usize), S1> = SmallBox::new((0usize, 1usize));
    /// assert!(heaped.is_heap());
    /// ```
    #[inline]
    pub fn is_heap(&self) -> bool {
        !self.ptr.is_null()
    }

    unsafe fn new_copy<U>(val: &U, metadata_ptr: *const T) -> SmallBox<T, Space>
    where U: ?Sized {
        let size = mem::size_of_val::<U>(val);
        let align = mem::align_of_val::<U>(val);

        let mut space = MaybeUninit::<Space>::uninit();

        let (ptr_this, val_dst): (*mut u8, *mut u8) = if size == 0 {
            (ptr::null_mut(), sptr::without_provenance_mut(align))
        } else if size > mem::size_of::<Space>() || align > mem::align_of::<Space>() {
            // Heap
            let layout = Layout::for_value::<U>(val);
            let heap_ptr = alloc::alloc(layout);

            (heap_ptr, heap_ptr)
        } else {
            // Stack
            (ptr::null_mut(), space.as_mut_ptr().cast())
        };

        // `self.ptr` always holds the metadata, even if stack allocated
        let ptr = sptr::with_metadata_of_mut(ptr_this, metadata_ptr);

        ptr::copy_nonoverlapping(sptr::from_ref(val).cast(), val_dst, size);

        SmallBox {
            space,
            ptr,
            _phantom: PhantomData,
        }
    }

    unsafe fn downcast_unchecked<U: Any>(self) -> SmallBox<U, Space> {
        let size = mem::size_of::<U>();
        let mut space = MaybeUninit::<Space>::uninit();

        if !self.is_heap() {
            ptr::copy_nonoverlapping::<u8>(
                self.space.as_ptr().cast(),
                space.as_mut_ptr().cast(),
                size,
            );
        };

        let ptr = self.ptr.cast();

        mem::forget(self);

        SmallBox {
            space,
            ptr,
            _phantom: PhantomData,
        }
    }

    #[inline]
    unsafe fn as_ptr(&self) -> *const T {
        if self.is_heap() {
            self.ptr
        } else {
            sptr::with_metadata_of(self.space.as_ptr(), self.ptr)
        }
    }

    #[inline]
    unsafe fn as_mut_ptr(&mut self) -> *mut T {
        if self.is_heap() {
            self.ptr
        } else {
            sptr::with_metadata_of_mut(self.space.as_mut_ptr(), self.ptr)
        }
    }

    /// Consumes the SmallBox and returns ownership of the boxed value
    ///
    /// # Examples
    /// ```
    /// use smallbox::space::S1;
    /// use smallbox::SmallBox;
    ///
    /// let stacked: SmallBox<_, S1> = SmallBox::new([21usize]);
    /// let val = stacked.into_inner();
    /// assert_eq!(val[0], 21);
    ///
    /// let boxed: SmallBox<_, S1> = SmallBox::new(vec![21, 56, 420]);
    /// let val = boxed.into_inner();
    /// assert_eq!(val[1], 56);
    /// ```
    #[inline]
    pub fn into_inner(self) -> T
    where T: Sized {
        let this = ManuallyDrop::new(self);
        let ret_val: T = unsafe { this.as_ptr().read() };

        // Just drops the heap without dropping the boxed value
        if this.is_heap() {
            let layout = Layout::new::<T>();
            unsafe {
                alloc::dealloc(this.ptr.cast(), layout);
            }
        }

        ret_val
    }
}

impl<Space> SmallBox<dyn Any, Space> {
    /// Attempt to downcast the box to a concrete type.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use]
    /// extern crate smallbox;
    ///
    /// # fn main() {
    /// use core::any::Any;
    ///
    /// use smallbox::space::*;
    /// use smallbox::SmallBox;
    ///
    /// fn print_if_string(value: SmallBox<dyn Any, S1>) {
    ///     if let Ok(string) = value.downcast::<String>() {
    ///         println!("String ({}): {}", string.len(), string);
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let my_string = "Hello World".to_string();
    ///     print_if_string(smallbox!(my_string));
    ///     print_if_string(smallbox!(0i8));
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn downcast<T: Any>(self) -> Result<SmallBox<T, Space>, Self> {
        if self.is::<T>() {
            unsafe { Ok(self.downcast_unchecked()) }
        } else {
            Err(self)
        }
    }
}

impl<Space> SmallBox<dyn Any + Send, Space> {
    /// Attempt to downcast the box to a concrete type.
    ///
    /// # Examples
    ///
    /// ```
    /// #[macro_use]
    /// extern crate smallbox;
    ///
    /// # fn main() {
    /// use core::any::Any;
    ///
    /// use smallbox::space::*;
    /// use smallbox::SmallBox;
    ///
    /// fn print_if_string(value: SmallBox<dyn Any, S1>) {
    ///     if let Ok(string) = value.downcast::<String>() {
    ///         println!("String ({}): {}", string.len(), string);
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let my_string = "Hello World".to_string();
    ///     print_if_string(smallbox!(my_string));
    ///     print_if_string(smallbox!(0i8));
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn downcast<T: Any>(self) -> Result<SmallBox<T, Space>, Self> {
        if self.is::<T>() {
            unsafe { Ok(self.downcast_unchecked()) }
        } else {
            Err(self)
        }
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
        unsafe { &mut *self.as_mut_ptr() }
    }
}

impl<T: ?Sized, Space> ops::Drop for SmallBox<T, Space> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::for_value::<T>(&*self);
            ptr::drop_in_place::<T>(&mut **self);
            if self.is_heap() {
                alloc::dealloc(self.ptr.cast(), layout);
            }
        }
    }
}

impl<T: Clone, Space> Clone for SmallBox<T, Space>
where T: Sized
{
    fn clone(&self) -> Self {
        let val: &T = self;
        SmallBox::new(val.clone())
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
    fn eq(&self, other: &SmallBox<T, Space>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<T: ?Sized + PartialOrd, Space> PartialOrd for SmallBox<T, Space> {
    fn partial_cmp(&self, other: &SmallBox<T, Space>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    fn lt(&self, other: &SmallBox<T, Space>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    fn le(&self, other: &SmallBox<T, Space>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    fn ge(&self, other: &SmallBox<T, Space>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    fn gt(&self, other: &SmallBox<T, Space>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: ?Sized + Ord, Space> Ord for SmallBox<T, Space> {
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
    use core::any::Any;
    use core::ptr::addr_of;

    use ::alloc::boxed::Box;
    use ::alloc::vec;

    use super::SmallBox;
    use crate::space::*;

    #[test]
    fn test_basic() {
        let stacked: SmallBox<usize, S1> = SmallBox::new(1234usize);
        assert!(*stacked == 1234);

        let heaped: SmallBox<(usize, usize), S1> = SmallBox::new((0, 1));
        assert!(*heaped == (0, 1));
    }

    #[test]
    fn test_new_unchecked() {
        let val = [0usize, 1];
        let ptr = addr_of!(val);

        unsafe {
            let stacked: SmallBox<[usize], S2> = SmallBox::new_unchecked(val, ptr);
            assert!(*stacked == [0, 1]);
            assert!(!stacked.is_heap());
        }

        let val = [0usize, 1, 2];
        let ptr = addr_of!(val);

        unsafe {
            let heaped: SmallBox<dyn Any, S2> = SmallBox::new_unchecked(val, ptr);
            assert!(heaped.is_heap());

            if let Some(array) = heaped.downcast_ref::<[usize; 3]>() {
                assert_eq!(*array, [0, 1, 2]);
            } else {
                unreachable!();
            }
        }
    }

    #[test]
    #[deny(unsafe_code)]
    fn test_macro() {
        let stacked: SmallBox<dyn Any, S1> = smallbox!(1234usize);
        if let Some(num) = stacked.downcast_ref::<usize>() {
            assert_eq!(*num, 1234);
        } else {
            unreachable!();
        }

        let heaped: SmallBox<dyn Any, S1> = smallbox!([0usize, 1]);
        if let Some(array) = heaped.downcast_ref::<[usize; 2]>() {
            assert_eq!(*array, [0, 1]);
        } else {
            unreachable!();
        }

        let is_even: SmallBox<dyn Fn(u8) -> bool, S1> = smallbox!(|num: u8| num % 2 == 0);
        assert!(!is_even(5));
        assert!(is_even(6));
    }

    #[test]
    #[cfg(feature = "coerce")]
    fn test_coerce() {
        let stacked: SmallBox<dyn Any, S1> = SmallBox::new(1234usize);
        if let Some(num) = stacked.downcast_ref::<usize>() {
            assert_eq!(*num, 1234);
        } else {
            unreachable!();
        }

        let heaped: SmallBox<dyn Any, S1> = SmallBox::new([0usize, 1]);
        if let Some(array) = heaped.downcast_ref::<[usize; 2]>() {
            assert_eq!(*array, [0, 1]);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_drop() {
        use core::cell::Cell;

        #[allow(dead_code)]
        struct Struct<'a>(&'a Cell<bool>, u8);
        impl<'a> Drop for Struct<'a> {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let flag = Cell::new(false);
        let stacked: SmallBox<_, S2> = SmallBox::new(Struct(&flag, 0));
        assert!(!stacked.is_heap());
        assert!(!flag.get());
        drop(stacked);
        assert!(flag.get());

        let flag = Cell::new(false);
        let heaped: SmallBox<_, S1> = SmallBox::new(Struct(&flag, 0));
        assert!(heaped.is_heap());
        assert!(!flag.get());
        drop(heaped);
        assert!(flag.get());
    }

    #[test]
    fn test_dont_drop_space() {
        #[allow(dead_code)]
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
        let fit = SmallBox::<_, S1>::new([1usize]);
        let oversize = SmallBox::<_, S1>::new([1usize, 2]);
        assert!(!fit.is_heap());
        assert!(oversize.is_heap());
    }

    #[test]
    fn test_resize() {
        let m = SmallBox::<_, S4>::new([1usize, 2]);
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
        assert_eq!(*m, [1usize, 2]);
    }

    #[test]
    fn test_clone() {
        let stacked: SmallBox<[usize; 2], S2> = smallbox!([1usize, 2]);
        assert_eq!(stacked, stacked.clone())
    }

    #[test]
    fn test_zst() {
        struct ZSpace;

        let zst: SmallBox<[usize], S1> = smallbox!([1usize; 0]);
        assert_eq!(*zst, [1usize; 0]);

        let zst: SmallBox<[usize], ZSpace> = smallbox!([1usize; 0]);
        assert_eq!(*zst, [1usize; 0]);
        let zst: SmallBox<[usize], ZSpace> = smallbox!([1usize; 2]);
        assert_eq!(*zst, [1usize; 2]);
    }

    #[test]
    fn test_downcast() {
        let stacked: SmallBox<dyn Any, S1> = smallbox!(0x01u32);
        assert!(!stacked.is_heap());
        assert_eq!(SmallBox::new(0x01), stacked.downcast::<u32>().unwrap());

        let heaped: SmallBox<dyn Any, S1> = smallbox!([1usize, 2]);
        assert!(heaped.is_heap());
        assert_eq!(
            smallbox!([1usize, 2]),
            heaped.downcast::<[usize; 2]>().unwrap()
        );

        let stacked_send: SmallBox<dyn Any + Send, S1> = smallbox!(0x01u32);
        assert!(!stacked_send.is_heap());
        assert_eq!(SmallBox::new(0x01), stacked_send.downcast::<u32>().unwrap());

        let heaped_send: SmallBox<dyn Any + Send, S1> = smallbox!([1usize, 2]);
        assert!(heaped_send.is_heap());
        assert_eq!(
            SmallBox::new([1usize, 2]),
            heaped_send.downcast::<[usize; 2]>().unwrap()
        );

        let mismatched: SmallBox<dyn Any, S1> = smallbox!(0x01u32);
        assert!(mismatched.downcast::<u8>().is_err());
        let mismatched: SmallBox<dyn Any, S1> = smallbox!(0x01u32);
        assert!(mismatched.downcast::<u64>().is_err());
    }

    #[test]
    fn test_option_encoding() {
        let tester: SmallBox<Box<()>, S2> = SmallBox::new(Box::new(()));
        assert!(Some(tester).is_some());
    }

    #[test]
    fn test_into_inner() {
        let tester: SmallBox<_, S1> = SmallBox::new([21usize]);
        let val = tester.into_inner();
        assert_eq!(val[0], 21);

        let tester: SmallBox<_, S1> = SmallBox::new(vec![21, 56, 420]);
        let val = tester.into_inner();
        assert_eq!(val[1], 56);
    }
}
