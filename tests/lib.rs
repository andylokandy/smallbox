extern crate smallbox;

use smallbox::StackBox;

// A trivial check that ensures that methods are correctly called
#[test]
fn basic() {
    let val = StackBox::<PartialEq<u32>>::new(1234u32).unwrap();
    assert!(*val == 1234);
}

#[test]
fn _drop() {
    use std::cell::Cell;
    #[derive(Debug)]
    struct Struct<'a>(&'a Cell<bool>);
    impl<'a> Drop for Struct<'a> {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let flag = Cell::new(false);
    let val: StackBox<::std::fmt::Debug> = StackBox::new(Struct(&flag)).unwrap();
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
    fn instance_one() -> StackBox<TestTrait> {
        #[derive(Debug)]
        struct OneStruct(u32);
        impl TestTrait for OneStruct {
            fn get_value(&self) -> u32 {
                self.0
            }
        }
        StackBox::new(OneStruct(12345)).unwrap()
    }

    #[inline(never)]
    fn instance_two() -> StackBox<TestTrait> {
        #[derive(Debug)]
        struct TwoStruct;
        impl TestTrait for TwoStruct {
            fn get_value(&self) -> u32 {
                54321
            }
        }
        StackBox::new(TwoStruct).unwrap()
    }

    #[inline(never)]
    fn instance_three() -> StackBox<[u8]> {
        StackBox::new([0; 8]).unwrap()
    }

    let i1 = instance_one();
    let i2 = instance_two();
    let i3: StackBox<[u8]> = instance_three();
    assert_eq!(i1.get_value(), 12345);
    assert_eq!(i2.get_value(), 54321);
    assert_eq!(i3.len(), 8);
}


#[test]
fn closure() {
    let v1 = 1234u64;
    let c: StackBox<Fn() -> String> = StackBox::new(|| format!("{}", v1)).ok().unwrap();
    assert_eq!(c(), "1234");
}


#[test]
fn oversize() {
    use std::any::Any;
    const MAX_SIZE: usize = 4;
    assert!(StackBox::<Any>::new([0usize; MAX_SIZE]).is_ok());
    assert_eq!(StackBox::<[usize]>::new([0usize; MAX_SIZE])
                   .unwrap()
                   .len(),
               MAX_SIZE);
    assert!(StackBox::<Any>::new([0usize; MAX_SIZE + 1]).is_err());
}
