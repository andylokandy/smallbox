#![cfg(feature = "heap")]

extern crate smallbox;

use smallbox::SmallBox;

#[test]
fn basic() {
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
    struct Struct<'a, T>(&'a Cell<bool>, T);
    impl<'a, T> Drop for Struct<'a, T> {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let flag = Cell::new(false);
    let val: SmallBox<Debug> = SmallBox::new(Struct(&flag, ()));
    assert!(flag.get() == false);
    drop(val);
    assert!(flag.get() == true);

    let flag = Cell::new(false);
    let val: SmallBox<Debug> = SmallBox::new(Struct(&flag, [0usize; 16]));
    assert!(flag.get() == false);
    drop(val);
    assert!(flag.get() == true);
}

#[test]
fn test_heap_fallback() {
    const MAX_SIZE: usize = 4;

    let small = SmallBox::<[usize]>::new([8; MAX_SIZE]);
    let medium = SmallBox::<[usize]>::new([7; MAX_SIZE + 1]);
    let huge = SmallBox::<[usize]>::new([6; 10000]);
    assert!(small.iter().eq([8; MAX_SIZE].iter()));
    assert!(medium.iter().eq([7; MAX_SIZE + 1].iter()));
    assert!(huge.iter().eq([6; 10000].iter()));
}

#[test]
fn test_downcast() {
    use std::any::Any;

    let num: SmallBox<Any> = SmallBox::new(1234u32);
    let string: SmallBox<Any> = SmallBox::new("hello world".to_owned());

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
fn test_resize() {
    use std::any::Any;
    use smallbox::space::*;

    let s = SmallBox::<Any, U4>::new([0usize; 4]);
    let m = s.resize::<U8>().ok().unwrap();

    if let Some(array) = m.downcast_ref::<[usize; 4]>() {
        assert_eq!(*array, [0usize; 4]);
    } else {
        unreachable!();
    }

    m.resize::<U4>().err().unwrap();

    let s = SmallBox::<Any, U4>::new([0usize; 8]);
    let m = s.resize::<U8>().ok().unwrap();

    if let Some(array) = m.downcast_ref::<[usize; 8]>() {
        assert_eq!(*array, [0usize; 8]);
    } else {
        unreachable!();
    }

    m.resize::<U4>().unwrap();
}

#[test]
fn test_zst() {
    use std::any::Any;

    let s = SmallBox::<Any>::new([0usize; 0]);

    if let Some(array) = s.downcast_ref::<[usize; 0]>() {
        assert_eq!(*array, [0usize; 0]);
    } else {
        unreachable!();
    }
}