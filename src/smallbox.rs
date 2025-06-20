use core::any::Any;
use core::cell::UnsafeCell;
use core::cmp::Ordering;
use core::fmt;
use core::future::Future;
use core::hash::Hash;
use core::hash::{self};
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
#[cfg(feature = "coerce")]
use core::marker::Unsize;
use core::mem::ManuallyDrop;
use core::mem::MaybeUninit;
use core::mem::{self};
use core::ops;
#[cfg(feature = "coerce")]
use core::ops::CoerceUnsized;
use core::pin::Pin;
use core::ptr;
use core::ptr::NonNull;

use ::alloc::alloc;
use ::alloc::alloc::Layout;
use ::alloc::alloc::handle_alloc_error;

use crate::sptr;

/// A sentinel pointer that signals that the value is stored on the stack
///
/// It is never supposed to be dereferenced
const INLINE_SENTINEL: *mut u8 = sptr::without_provenance_mut(0x1);

/// Minimum alignment for allocations
///
/// Forcing a minimum alignment prevents the allocator
/// from returning a pointer with the same address as `INLINE_SENTINEL`
const MIN_ALIGNMENT: usize = 2;

#[cfg(feature = "coerce")]
impl<T: ?Sized + Unsize<U>, U: ?Sized, Space> CoerceUnsized<SmallBox<U, Space>>
    for SmallBox<T, Space>
{
}

/// Box value on stack or on heap depending on its size
///
/// This macro is similar to `SmallBox::new`, which is used to create a new `SmallBox` instance,
/// but relaxes the constraint `T: Sized`.
/// In order to do that, this macro will check the coercion rules from type `T` to
/// the target type. This macro will invoke a compile-time error on any invalid type coercion.
///
/// You can think that it has the signature of `smallbox!<U: Sized, T: ?Sized>(val: U) ->
/// SmallBox<T, Space>`
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
        let ptr = ::core::ptr::addr_of!(val);
        #[allow(unsafe_code)]
        unsafe {
            $crate::SmallBox::new_unchecked(val, ptr)
        }
    }};
}

/// An optimized box that store value on stack or on heap depending on its size
pub struct SmallBox<T: ?Sized, Space> {
    space: MaybeUninit<UnsafeCell<Space>>,
    // NonNull enables Null Pointer Optimization
    ptr: NonNull<T>,
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
    /// when inline space is not sufficient. Once the data
    /// is moved to heap, it will never be moved back to stack.
    ///
    /// # Example
    ///
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::S2;
    /// use smallbox::space::S4;
    ///
    /// let s: SmallBox<_, S4> = SmallBox::new([0usize; 4]);
    /// let m: SmallBox<_, S2> = s.resize();
    /// ```
    pub fn resize<ToSpace>(self) -> SmallBox<T, ToSpace> {
        let this = ManuallyDrop::new(self);

        if this.is_heap() {
            // don't change anything if data is already on heap
            let space = MaybeUninit::<UnsafeCell<ToSpace>>::uninit();
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
    /// use smallbox::SmallBox;
    /// use smallbox::space::S1;
    ///
    /// let stacked: SmallBox<usize, S1> = SmallBox::new(0usize);
    /// assert!(!stacked.is_heap());
    ///
    /// let heaped: SmallBox<(usize, usize), S1> = SmallBox::new((0usize, 1usize));
    /// assert!(heaped.is_heap());
    /// ```
    #[inline]
    pub fn is_heap(&self) -> bool {
        self.ptr.as_ptr().cast::<u8>() != INLINE_SENTINEL
    }

    unsafe fn new_copy<U>(val: &U, metadata_ptr: *const T) -> SmallBox<T, Space>
    where U: ?Sized {
        let layout = Layout::for_value::<U>(val);
        let space_layout = Layout::new::<Space>();

        let mut space = MaybeUninit::<UnsafeCell<Space>>::uninit();

        let (ptr_this, val_dst): (*mut u8, *mut u8) =
            if layout.size() <= space_layout.size() && layout.align() <= space_layout.align() {
                // Stack.
                (INLINE_SENTINEL, space.as_mut_ptr().cast())
            } else if layout.size() == 0 {
                // ZST with alignment greater than Space, which will behave like being stored on
                // heap but will not actually allocate.
                (
                    sptr::without_provenance_mut(layout.align()),
                    sptr::without_provenance_mut(layout.align()),
                )
            } else {
                // Heap.
                let layout = Layout::for_value::<U>(val)
                    // Safety: MIN_ALIGNMENT is 2, which is a valid power-of-two alignment.
                    .align_to(MIN_ALIGNMENT)
                    .unwrap_or_else(|_| unreachable_unchecked());
                let heap_ptr = alloc::alloc(layout);

                if heap_ptr.is_null() {
                    handle_alloc_error(layout)
                }

                (heap_ptr, heap_ptr)
            };

        // `self.ptr` always holds the metadata, even if stack allocated.
        let ptr = sptr::with_metadata_of_mut(ptr_this, metadata_ptr);
        // Safety: ptr is either INLINE_SENTINEL or returned from the allocator and checked for
        // null.
        let ptr = NonNull::new_unchecked(ptr);

        ptr::copy_nonoverlapping(sptr::from_ref(val).cast(), val_dst, layout.size());

        SmallBox {
            space,
            ptr,
            _phantom: PhantomData,
        }
    }

    unsafe fn downcast_unchecked<U: Any>(self) -> SmallBox<U, Space> {
        let size = mem::size_of::<U>();
        let mut space = MaybeUninit::<UnsafeCell<Space>>::uninit();

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
            self.ptr.as_ptr()
        } else {
            sptr::with_metadata_of(self.space.as_ptr(), self.ptr.as_ptr())
        }
    }

    #[inline]
    unsafe fn as_mut_ptr(&mut self) -> *mut T {
        if self.is_heap() {
            self.ptr.as_ptr()
        } else {
            sptr::with_metadata_of_mut(self.space.as_mut_ptr(), self.ptr.as_ptr())
        }
    }

    /// Consumes the SmallBox and returns ownership of the boxed value
    ///
    /// # Examples
    /// ```
    /// use smallbox::SmallBox;
    /// use smallbox::space::S1;
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

        // Just deallocates the heap memory without dropping the boxed value
        if this.is_heap() && mem::size_of::<T>() != 0 {
            // Safety: MIN_ALIGNMENT is 2, aligning to 2 should not create an invalid layout
            let layout = unsafe {
                Layout::new::<T>()
                    .align_to(MIN_ALIGNMENT)
                    .unwrap_or_else(|_| unreachable_unchecked())
            };
            unsafe {
                alloc::dealloc(this.ptr.as_ptr().cast::<u8>(), layout);
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
    /// use smallbox::SmallBox;
    /// use smallbox::space::*;
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
    /// use smallbox::SmallBox;
    /// use smallbox::space::*;
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
            let layout = Layout::for_value::<T>(&*self)
                .align_to(MIN_ALIGNMENT)
                .unwrap_or_else(|_| unreachable_unchecked());

            ptr::drop_in_place::<T>(&mut **self);
            if self.is_heap() && layout.size() != 0 {
                alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), layout);
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

// We can implement Future for SmallBox soundly, even though it's not implemented for std Box.
// The reason why it's not implemented for std Box is only because Box<T>: Unpin unconditionally,
// even when T: !Unpin, which always allows getting &mut Box<T> from Pin<&mut Box<T>>.
// For SmallBox, this is not the case, because it might carry the data on the stack, so if T:
// !Unpin, then SmallBox<T>: !Unpin also. That means you can't get &mut SmallBox<T> from Pin<&mut
// SmallBox<T>> in safe code, so it's safe to implement Future for SmallBox directly.
impl<F: Future + ?Sized, S> Future for SmallBox<F, S> {
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        // Safety: When the SmallBox is pinned, the data on the stack is pinned.
        // The data on the heap is also pinned naturally, so all Pin guarantees are satisfied.
        unsafe { Pin::new_unchecked(&mut **self.get_unchecked_mut()) }.poll(cx)
    }
}

unsafe impl<T: ?Sized + Send, Space> Send for SmallBox<T, Space> {}
unsafe impl<T: ?Sized + Sync, Space> Sync for SmallBox<T, Space> {}

#[cfg(test)]
mod tests {
    use core::any::Any;
    use core::mem;
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

    #[test]
    fn test_interior_mutability() {
        use core::cell::Cell;
        let cellbox = SmallBox::<Cell<u32>, S1>::new(Cell::new(0));
        assert!(!cellbox.is_heap());
        cellbox.set(1);
        assert_eq!(cellbox.get(), 1);
    }

    #[test]
    fn test_future() {
        let boxed_fut: SmallBox<_, S1> = SmallBox::new(async { 123 });

        assert_eq!(futures::executor::block_on(boxed_fut), 123);
    }

    #[test]
    fn test_variance() {
        #[allow(dead_code)]
        fn test<'short, 'long: 'short>(val: SmallBox<&'long str, S1>) -> SmallBox<&'short str, S1> {
            val
        }
    }

    #[test]
    fn test_overaligned_zst() {
        #[repr(align(512))]
        #[derive(Debug, PartialEq, Eq)]
        struct OveralignedZst;

        let zst: SmallBox<OveralignedZst, S1> = smallbox!(OveralignedZst);
        let zst_addr = addr_of!(*zst).addr();
        assert_eq!(*zst, OveralignedZst);
        assert_eq!(zst_addr % 512, 0);
    }

    #[test]
    fn test_overaligned_zst_dyn() {
        #[repr(align(512))]
        #[derive(Debug, PartialEq, Eq)]
        struct OveralignedZst;

        trait Foo {}

        impl Foo for OveralignedZst {}

        let zst: SmallBox<dyn Foo, S1> = smallbox!(OveralignedZst);
        let zst_addr = addr_of!(*zst).addr();
        assert_eq!(zst_addr % 512, 0);
    }

    #[test]
    fn test_null_ptr_optimization() {
        assert_eq!(
            mem::size_of::<SmallBox<i32, S1>>(),
            mem::size_of::<Option<SmallBox<i32, S1>>>()
        );
    }
}
