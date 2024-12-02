#![feature(test)]

extern crate smallbox;
extern crate test;

use smallbox::SmallBox;
use smallbox::space::*;
use test::Bencher;
use test::black_box;

#[bench]
fn smallbox_small_item_small_space(b: &mut Bencher) {
    b.iter(|| {
        let small: SmallBox<_, S1> = black_box(SmallBox::new(black_box(true)));
        small
    })
}

#[bench]
fn smallbox_small_item_large_space(b: &mut Bencher) {
    b.iter(|| {
        let small: SmallBox<_, S64> = black_box(SmallBox::new(black_box(true)));
        small
    })
}

#[bench]
fn smallbox_large_item_small_space(b: &mut Bencher) {
    b.iter(|| {
        let large: SmallBox<_, S1> = black_box(SmallBox::new(black_box([0usize; 64])));
        large
    })
}

#[bench]
fn smallbox_large_item_large_space(b: &mut Bencher) {
    b.iter(|| {
        let large: SmallBox<_, S64> = black_box(SmallBox::new(black_box([0usize; 64])));
        large
    })
}

#[bench]
fn box_small_item(b: &mut Bencher) {
    b.iter(|| {
        let large: Box<_> = black_box(Box::new(black_box(true)));
        large
    })
}

#[bench]
fn box_large_item(b: &mut Bencher) {
    b.iter(|| {
        let large: Box<_> = black_box(Box::new(black_box([0usize; 64])));
        large
    })
}
