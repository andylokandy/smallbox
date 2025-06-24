use divan;
use smallbox::SmallBox;
use smallbox::space::*;

fn main() {
    divan::main();
}

#[divan::bench]
fn smallbox_small_item_small_space() {
    divan::black_box({
        let small: SmallBox<_, S1> = SmallBox::new(divan::black_box(true));
        small
    });
}

#[divan::bench]
fn smallbox_small_item_large_space() {
    divan::black_box({
        let small: SmallBox<_, S64> = SmallBox::new(divan::black_box(true));
        small
    });
}

#[divan::bench]
fn smallbox_large_item_small_space() {
    divan::black_box({
        let large: SmallBox<_, S1> = SmallBox::new(divan::black_box([0usize; 64]));
        large
    });
}

#[divan::bench]
fn smallbox_large_item_large_space() {
    divan::black_box({
        let large: SmallBox<_, S64> = SmallBox::new(divan::black_box([0usize; 64]));
        large
    });
}

#[divan::bench]
fn box_small_item() {
    divan::black_box({
        let large: Box<_> = Box::new(divan::black_box(true));
        large
    });
}

#[divan::bench]
fn box_large_item() {
    divan::black_box({
        let large: Box<_> = Box::new(divan::black_box([0usize; 64]));
        large
    });
}
