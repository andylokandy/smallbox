#![feature(unsize)]

use std::ops;
use std::mem;
use std::ptr;
use std::slice;
use std::marker;

pub const DEFAULT_SIZE: usize = 4 + 1;

type Space = [usize; DEFAULT_SIZE];

/// Stack-allocated dynamically sized type
pub struct SmallBox<T: ?Sized> {
    // force alignment to be word
    _align: [usize; 0],
    _pd: marker::PhantomData<T>,
    // (T, (padding), fat_pointer_info)
    space: Space,
}

unsafe fn ptr_as_slice<'p, T: ?Sized>(ptr: &'p mut *const T) -> &'p mut [usize] {
    let words = mem::size_of::<&T>() / mem::size_of::<usize>();
    slice::from_raw_parts_mut(ptr as *mut _ as *mut usize, words)
}

impl<T: ?Sized> SmallBox<T> {
    pub fn new<U>(val: U) -> Result<SmallBox<T>, U>
        where U: marker::Unsize<T>
    {
        if mem::size_of::<&T>() + mem::size_of::<U>() - mem::size_of::<usize>() >
           mem::size_of::<Space>() {
            return Err(val);
            // TODO:
        } else {
            return unsafe { Ok(Self::new_inline(val)) };
        }
    }

    pub unsafe fn new_inline<U>(val: U) -> SmallBox<T>
        where U: marker::Unsize<T>
    {
        // fat pointer (cast to avoid brrowck)
        // memory layout: (ptr: usize, info: [usize])
        let mut ptr: *const T = &val;
        // let mut ptr: &T = &*(&val as *const T);

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

        SmallBox {
            _align: [],
            _pd: marker::PhantomData,
            space: space,
        }
    }

    unsafe fn as_fat_ptr(&self) -> *const T {
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
impl<T: ?Sized> ops::Deref for SmallBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.as_fat_ptr() }
    }
}

impl<T: ?Sized> ops::DerefMut for SmallBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.as_fat_ptr() as *mut T) }
    }
}

impl<T: ?Sized> ops::Drop for SmallBox<T> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(&mut **self) }
    }
}
