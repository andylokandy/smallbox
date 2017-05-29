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