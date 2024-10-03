#[cfg(feature = "nightly")]
mod implementation {
    pub use core::ptr::without_provenance_mut;

    pub fn with_metadata_of<T: ?Sized, U: ?Sized>(ptr: *const T, meta: *const U) -> *const U {
        ptr.with_metadata_of(meta)
    }

    pub fn with_metadata_of_mut<T: ?Sized, U: ?Sized>(ptr: *mut T, meta: *const U) -> *mut U {
        ptr.with_metadata_of(meta)
    }
}

#[cfg(not(feature = "nightly"))]
#[allow(clippy::as_conversions)]
mod implementation {
    use core::ptr::addr_of_mut;

    fn cast_to_mut<T: ?Sized>(ptr: *const T) -> *mut T {
        ptr as _
    }

    pub fn without_provenance_mut<T>(addr: usize) -> *mut T {
        unsafe { core::mem::transmute(addr) }
    }

    pub fn with_metadata_of<T: ?Sized, U: ?Sized>(ptr: *const T, meta: *const U) -> *const U {
        with_metadata_of_mut(cast_to_mut(ptr), meta)
    }

    pub fn with_metadata_of_mut<T: ?Sized, U: ?Sized>(ptr: *mut T, mut meta: *const U) -> *mut U {
        let meta_ptr = addr_of_mut!(meta).cast::<*mut u8>();
        unsafe { meta_ptr.write(ptr.cast::<u8>()) }
        cast_to_mut(meta)
    }
}

pub fn from_ref<T: ?Sized>(val: &T) -> *const T {
    val
}

pub use implementation::*;
