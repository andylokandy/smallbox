use std::ptr;

#[allow(dead_code)]
struct Sample(usize);

trait SomeTrait {
    fn call_me(&self) -> bool {
        true
    }
}

impl SomeTrait for Sample {}

fn layout_broken(what: &str) {
    panic!(
        concat!(
            "Assumptions on layout are broken, this crate relies on ",
            "`unsafe code guidelines` layout specification, ",
            "now layout of {:?} is broken, report about it on github"
        ),
        what
    );
}

/// Tests some layout assumptions, these cases as of now:
///
/// 1. data pointer and vtable pointer location of trait objects
/// 2. data pointer and size location of slice objects
fn test_ptr_layouts() {
    // Test for dyn object
    {
        #[repr(C)]
        struct DynObj {
            data_ptr: *const u8,
            vtable: *const u8,
        }

        let sample = Box::new(Sample(100));
        let data_ptr = Box::into_raw(sample);

        let trait_obj: *const dyn SomeTrait = data_ptr;
        let dyn_obj_repr: DynObj = unsafe { ptr::read(ptr::addr_of!(trait_obj) as *const DynObj) };

        if dyn_obj_repr.data_ptr != data_ptr as *const u8 {
            layout_broken("trait objects");
        }
        let out = unsafe { Box::from_raw(data_ptr) };
        out.call_me();
    }

    // Test for slice object
    {
        let array = [1, 2, 3];
        let slice: &[u8] = &array;

        #[repr(C)]
        struct Slice {
            data_ptr: *const u8,
            size: usize,
        }

        let slice_repr: Slice = unsafe { ptr::read(ptr::addr_of!(slice) as *const Slice) };

        if slice_repr.data_ptr != slice.as_ptr() || slice_repr.size != slice.len() {
            layout_broken("slices");
        }
    }
}

fn main() {
    // NOTE: this will not protect from every possible case,
    // for example, rust may add one more fat pointer type which this test
    // will not check, host layout may be different from target layout,
    // and probably more.
    test_ptr_layouts();
}
