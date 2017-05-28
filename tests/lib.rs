extern crate smallbox;

use smallbox::StackBox;
use smallbox::SmallBox;

#[test]
fn basic() {
    let stack = StackBox::<PartialEq<u32>>::new(1234u32).unwrap();
    assert!(*stack == 1234);

    let small_stack = SmallBox::<PartialEq<u32>>::new(4321u32);
    assert!(*small_stack == 4321);
    match small_stack {
        SmallBox::Stack(_) => (),
        _ => unreachable!(),
    }

    let small_heap = SmallBox::<[usize]>::new([5; 1000]);
    assert!(small_heap.iter().eq([5; 1000].iter()));
    match small_heap {
        SmallBox::Box(_) => (),
        _ => unreachable!(),
    }
}

#[test]
fn test_drop() {
    use std::cell::Cell;
    use std::fmt::Debug;

    #[derive(Debug)]
    struct Struct<'a>(&'a Cell<bool>);
    impl<'a> Drop for Struct<'a> {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let flag = Cell::new(false);
    let val: StackBox<Debug> = StackBox::new(Struct(&flag)).unwrap();
    assert!(flag.get() == false);
    drop(val);
    assert!(flag.get() == true);

    let flag = Cell::new(false);
    let val: SmallBox<Debug> = SmallBox::new(Struct(&flag));
    assert!(flag.get() == false);
    drop(val);
    assert!(flag.get() == true);

    let flag = Cell::new(false);
    let val: SmallBox<Debug> = SmallBox::new(Struct(&flag));
    assert!(flag.get() == false);
    drop(val);
    assert!(flag.get() == true);
}

#[test]
fn many_instances() {
    trait TestTrait {
        fn get_value(&self) -> u32;
    }

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
fn test_closure() {
    let c: StackBox<Fn() -> String> = StackBox::new(|| format!("{}", 1234u64)).ok().unwrap();
    assert_eq!(c(), "1234");
}

#[test]
fn test_heap_fallback() {
    const MAX_SIZE: usize = 4;

    let fit = StackBox::<[usize]>::new([0; MAX_SIZE]);
    let oversize = StackBox::<[usize]>::new([0; MAX_SIZE + 1]);
    assert!(fit.is_ok());
    assert!(oversize.is_err());
    assert_eq!(fit.unwrap().len(), MAX_SIZE);

    let small = SmallBox::<[usize]>::new([8; MAX_SIZE]);
    let medium = SmallBox::<[usize]>::new([7; MAX_SIZE + 1]);
    let huge = SmallBox::<[usize]>::new([6; 10000]);
    assert!(small.iter().eq([8; MAX_SIZE].iter()));
    assert!(medium.iter().eq([7; MAX_SIZE + 1].iter()));
    assert!(huge.iter().eq([6; 10000].iter()));
}