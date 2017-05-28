# `smallbox`

[![Build Status](https://travis-ci.org/goandylok/smallbox.svg?branch=master)](https://travis-ci.org/goandylok/smallbox)
[![crates.io](https://img.shields.io/crates/v/arraydeque.svg)](https://crates.io/crates/smallbox)
[![docs.rs](https://docs.rs/arraydeque/badge.svg)](https://docs.rs/smallbox)

Box dynamically-sized types on stack. Requires nightly rust.

Store or return trait-object and closure without heap allocation, and fallback to heap when thing goes too large.

## [**Documentation**](https://docs.rs/smallbox/)

# Usage
First, add the following to your `Cargo.toml`:

```toml
[dependencies]
smallbox = "0.2"
```

Next, add this to your crate root:

```rust
extern crate smallbox;
```


# Overview
This crate delivers two core type:

 `StackBox<T>`: Represents a fixed-capacity allocation, and on stack stores dynamically-sized type. 
 The `new` method on this type allows creating a instance from a concrete type, 
 returning `Err(value)` if the instance is too large for the allocated region. 
 So far, the fixed-capcity is about four words (4 * `sizeof(usize)`)
 
 `SmallBox<T>`: Takes `StackBox<T>` as an varience, and fallback to `Box<T>` when type `T` is too large for `StackBox<T>`.


# Example
One of the most obvious uses is to allow returning capturing closures without having to box them.

```rust
use smallbox::StackBox;

fn make_closure(s: String) -> StackBox<Fn()->String> {
    StackBox::new(move || format!("Hello, {}", s)).ok().unwrap()
}

let closure = make_closure("world!".to_owned());
assert_eq!(closure(), "Hello, world!");
```

The other uses is to eliminate heap alloction for small things, only when 
the object is large enough to allocte. 
In addition, the inner `StackBox<T>` or `Box<T>` can be moved out by explicitely pattern matching on `SmallBox<T>`.

```rust
use smallbox::SmallBox;

let tiny: SmallBox<[u64]> = SmallBox::new([0; 2]);
let big: SmallBox<[u64]> = SmallBox::new([1; 8]);

assert_eq!(tiny.len(), 2);
assert_eq!(big[7], 1);

match tiny {
    SmallBox::Stack(val) => assert_eq!(*val, [0; 2]),
    _ => unreachable!()
}

match big {
    SmallBox::Box(val) => assert_eq!(*val, [1; 8]),
    _ => unreachable!()
}
```

# Roadmap
- `no_std` support
- impl `Debug`, `Display`
- method that convert SmallBox<T> to Box<T>
- conveniently convert bewteen `SmallBox<T>` and `StackBox<T>`
- optional `SmallBox<T>` and heap dependency
- configurable `StackBox<T>` allocation size
- dowancasting for `StackBox<Any>` and `SmallBox<Any>`


## Contribution

All kinds of contribution are welcome.

- **Issue** Feel free to open an issue when you find typos, bugs, or have any question.
- **Pull requests**. Better implementation, more tests, more documents and typo fixes are all welcome.


## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.