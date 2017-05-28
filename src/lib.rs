#![feature(unsize)]
#![feature(box_syntax)]

mod stackbox;
mod smallbox;

pub use stackbox::StackBox;
pub use smallbox::SmallBox;