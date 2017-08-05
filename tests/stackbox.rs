extern crate smallbox;

use smallbox::StackBox;

#[test]
fn basic() {
    let stack = StackBox::<PartialEq<u32>>::new(1234u32).unwrap();
    assert!(*stack == 1234);
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
}

#[test]
fn test_nodrop_space() {
    use std::any::Any;
    use smallbox::space::S4;

    struct NoDrop(S4);
    impl Drop for NoDrop {
        fn drop(&mut self) {
            unreachable!();
        }
    }

    drop(StackBox::<Any, NoDrop>::new(true).unwrap());
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
fn test_oversize() {
    const DEFAULT_MAX_SIZE: usize = 2;

    let fit = StackBox::<[usize]>::new([0; DEFAULT_MAX_SIZE]);
    let oversize = StackBox::<[usize]>::new([0; DEFAULT_MAX_SIZE + 1]);
    assert!(fit.is_ok());
    assert!(oversize.is_err());
}

#[test]
fn test_downcast() {
    use std::any::Any;
    use smallbox::space::*;

    let num: StackBox<Any, S4> = StackBox::new(1234u32).unwrap();
    let string: StackBox<Any, S4> = StackBox::new("hello world".to_owned()).unwrap();

    if let Some(num) = num.downcast_ref::<u32>() {
        assert_eq!(*num, 1234);
    } else {
        unreachable!();
    }

    if let Some(string) = string.downcast_ref::<String>() {
        assert_eq!(string, "hello world");
    } else {
        unreachable!();
    }
}

#[test]
fn test_space_size() {
    use smallbox::space::*;

    assert!(StackBox::<[usize], S4>::new([0usize; 4]).is_ok());
    assert!(StackBox::<[usize], S4>::new([0usize; 4 + 1]).is_err());
    assert!(StackBox::<[usize], S8>::new([0usize; 8]).is_ok());
    assert!(StackBox::<[usize], S8>::new([0usize; 8 + 1]).is_err());
    assert!(StackBox::<[usize], [usize; 32]>::new([0usize; 32]).is_ok());
    assert!(StackBox::<[usize], [usize; 32]>::new([0usize; 32 + 1]).is_err());
    assert!(StackBox::<[u32], [u8; 32]>::new([0u32; 8]).is_ok());
    assert!(StackBox::<[u32], [u8; 32]>::new([0u32; 8 + 1]).is_err());
}

#[test]
fn test_resize() {
    use std::any::Any;
    use smallbox::space::*;

    let s = StackBox::<Any, S4>::new([0usize; 4]).unwrap();
    let m = s.resize::<S8>().ok().unwrap();

    if let Some(array) = m.downcast_ref::<[usize; 4]>() {
        assert_eq!(*array, [0usize; 4]);
    } else {
        unreachable!();
    }

    m.resize::<S4>().err().unwrap();
}

#[test]
fn test_zst() {
    use std::any::Any;

    let s = StackBox::<Any>::new([0usize; 0]).unwrap();

    if let Some(array) = s.downcast_ref::<[usize; 0]>() {
        assert_eq!(*array, [0usize; 0]);
    } else {
        unreachable!();
    }
}