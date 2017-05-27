extern crate smallbox;

use smallbox::SmallBox;

#[test]
// A trivial check that ensures that methods are correctly called
fn trivial_type() {
    let val = SmallBox::<PartialEq<u32>>::new(1234u32).unwrap();
    assert!(*val == 1234);
    assert!(*val != 1233);
}

#[test]
// Create an instance with a Drop implementation, and ensure the drop handler fires when destructed
// This also ensures that lifetimes are correctly handled
fn ensure_drop() {
    use std::cell::Cell;
    #[derive(Debug)]
    struct Struct<'a>(&'a Cell<bool>);
    impl<'a> Drop for Struct<'a> {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let flag = Cell::new(false);
    let val: SmallBox<::std::fmt::Debug> = SmallBox::new(Struct(&flag)).unwrap();
    assert!(flag.get() == false);
    drop(val);
    assert!(flag.get() == true);
}

#[test]
fn many_instances() {
    trait TestTrait {
        fn get_value(&self) -> u32;
    }

    #[inline(never)]
    fn instance_one() -> SmallBox<TestTrait> {
        #[derive(Debug)]
        struct OneStruct(u32);
        impl TestTrait for OneStruct {
            fn get_value(&self) -> u32 {
                self.0
            }
        }
        SmallBox::new(OneStruct(12345)).unwrap()
    }

    #[inline(never)]
    fn instance_two() -> SmallBox<TestTrait> {
        #[derive(Debug)]
        struct TwoStruct;
        impl TestTrait for TwoStruct {
            fn get_value(&self) -> u32 {
                54321
            }
        }
        SmallBox::new(TwoStruct).unwrap()
    }

    #[inline(never)]
    fn instance_three() -> SmallBox<[u8]> {
        SmallBox::new([0; 8]).unwrap()
    }

    let i1 = instance_one();
    let i2 = instance_two();
    let i3: SmallBox<[u8]> = instance_three();
    assert_eq!(i1.get_value(), 12345);
    assert_eq!(i2.get_value(), 54321);
    assert_eq!(i3.len(), 8);
}


#[test]
fn closure() {
    let v1 = 1234u64;
    let c: SmallBox<Fn() -> String> = SmallBox::new(|| format!("{}", v1))
        .map_err(|_| "Oops")
        .unwrap();
    assert_eq!(c(), "1234");
}


#[test]
fn oversize() {
    use std::any::Any;
    const MAX_SIZE_PTRS: usize = 4;
    assert!(SmallBox::<Any>::new([0usize; MAX_SIZE_PTRS]).is_ok());
    assert!(SmallBox::<Any>::new([0usize; MAX_SIZE_PTRS + 1]).is_err());
}
