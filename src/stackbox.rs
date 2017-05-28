use std::ops;
use std::mem;
use std::ptr;
use std::slice;
use std::marker;

const DEFAULT_SIZE: usize = 4 + 1;

type Space = [usize; DEFAULT_SIZE];

/// On-stack allocation for dynamically-sized type.
///
/// # Examples
///
/// ```
/// use smallbox::StackBox;
/// 
/// let val: StackBox<PartialEq<usize>> = StackBox::new(5usize).unwrap();
/// 
/// assert!(*val == 5)
/// ```
pub struct StackBox<T: ?Sized> {
    // force alignment to be usize
    _align: [usize; 0],
    _pd: marker::PhantomData<T>,
    space: Space,
}

unsafe fn ptr_as_slice<'p, T: ?Sized>(ptr: &'p mut *const T) -> &'p mut [usize] {
    let words = mem::size_of::<&T>() / mem::size_of::<usize>();
    slice::from_raw_parts_mut(ptr as *mut _ as *mut usize, words)
}

impl<T: ?Sized> StackBox<T> {
    /// Alloc on stack and try to box val, return Err<T> 
    /// when val is too large (about 4 words)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::any::Any;
    /// use smallbox::StackBox;
    /// 
    /// assert!(StackBox::<Any>::new(5usize).is_ok());
    /// assert!(StackBox::<Any>::new([5usize; 8]).is_err());
    /// ```
    pub fn new<U>(val: U) -> Result<StackBox<T>, U>
        where U: marker::Unsize<T>
    {
        if mem::size_of::<&T>() + mem::size_of::<U>() - mem::size_of::<usize>() >
           mem::size_of::<Space>() {
            Err(val)
        } else {
            unsafe { Ok(Self::box_up(val)) }
        }
    }

    // store value and metadata(for example: array length)
    unsafe fn box_up<U>(val: U) -> StackBox<T>
        where U: marker::Unsize<T>
    {
        // raw fat pointer
        // memory layout: (ptr: usize, info: [usize])
        let mut ptr: *const T = &val;

        let ptr_words = ptr_as_slice(&mut ptr);

        debug_assert!(ptr_words[0] == &val as *const _ as usize,
                      "Pointer layout is not (data_ptr, info ...)");

        debug_assert!(mem::align_of::<Self>() == mem::size_of::<usize>(),
                      "Self alignment should equal usize's");

        debug_assert!(mem::align_of::<U>() <= mem::align_of::<Self>(),
                      "Self alignment should ge than T's: {} (current is {})",
                      mem::align_of::<U>(),
                      mem::align_of::<Self>());

        // Space memeroy layout: (U, padding, info)
        let mut space = mem::uninitialized::<Space>();

        // move data into space
        ptr::copy_nonoverlapping(&val, (&mut space).as_ptr() as *mut U, 1);

        // place pointer information at the end of the region
        {
            let info = &ptr_words[1..];
            let space_info_offset = space.len() - info.len();
            let space_info = &mut space[space_info_offset..];
            space_info.clone_from_slice(info);
        }

        mem::forget(val);

        StackBox {
            _align: [],
            _pd: marker::PhantomData,
            space: space,
        }
    }

    // make a fat pointer to self.space with metadata
    pub(crate) unsafe fn as_fat_ptr(&self) -> *const T {
        let mut ptr: *const T = mem::zeroed();

        {
            let ptr_words = ptr_as_slice(&mut ptr);

            // set pointer
            ptr_words[0] = self.space.as_ptr() as usize;

            // set info
            let info = &mut ptr_words[1..];
            let space_info_offset = self.space.len() - info.len();
            let space_info = &self.space[space_info_offset..];
            info.clone_from_slice(space_info);
        }

        ptr as _
    }
}

impl<T: ?Sized> ops::Deref for StackBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.as_fat_ptr() }
    }
}

impl<T: ?Sized> ops::DerefMut for StackBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.as_fat_ptr() as *mut T) }
    }
}

impl<T: ?Sized> ops::Drop for StackBox<T> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(&mut **self) }
    }
}
