# `smallbox`

[![Build Status](https://travis-ci.org/goandylok/smallbox.svg?branch=master)](https://travis-ci.org/goandylok/smallbox)
[![crates.io](https://img.shields.io/crates/v/smallbox.svg)](https://crates.io/crates/smallbox)
[![docs.rs](https://docs.rs/smallbox/badge.svg)](https://docs.rs/smallbox)

Box dynamically-sized types on stack. Requires nightly rust.

Store or return trait-object and closure without heap allocation, and fallback to heap when thing goes too large.

## [**Documentation**](https://docs.rs/smallbox/)

# Usage
First, add the following to your `Cargo.toml`:

```toml
[dependencies]
smallbox = "0.4.*"
```

Next, add this to your crate root:

```rust
extern crate smallbox;
```

Currently `smallbox` by default links to the standard library, but if you would
instead like to use this crate in a `#![no_std]` situation or crate, and want to 
opt out heap dependency and `SmallBox<T>` type, you can request this via:

```toml
[dependencies]
smallbox = { version = "0.4.*", default-features = false }
```

Enable `heap` feature for `#![no_std]` build to link to `alloc` crate
and bring `SmallBox<T>` back.

```toml
[dependencies.smallbox]
version = "0.4.*"
default-features = false
features = ["heap"]
```


# Feature Flags

The **stackbox** crate has the following cargo feature flags:

- `std`
  - Optional, enabled by default
  - Use libstd


- `heap`
  - Optional
  - Use heap fallback and include `SmallBox<T>` type, and link to `alloc` crate if `std`
    feature flag is opted out.


# Overview
This crate delivers two core type:

 `StackBox<T>`: Represents as a fixed-capacity allocation, and on stack stores dynamically-sized type. 
 The `new` method creates returns `Err(value)` if the instance larger then `Space`.
 Default capacity is two words (2 * `sizeof(usize)`), more details on custom capacity are at the following sections.

 
 `SmallBox<T>`: Takes `StackBox<T>` as an varience, and fallback to heap-alloc `Box<T>` when type `T` is larger then `Space`.


# Example
The simplest usage is trait object dynamic-dispatch
```rust
use smallbox::StackBox;
 
let val: StackBox<PartialEq<usize>> = StackBox::new(5usize).unwrap();
 
assert!(*val == 5)
```

`Any` downcasting is also quite a good use.

```rust
use std::any::Any;

let num: StackBox<Any> = StackBox::new(1234u32).unwrap();

if let Some(num) = num.downcast_ref::<u32>() {
    assert_eq!(*num, 1234);
} else {
    unreachable!();
}
```

Another use case is to allow returning capturing closures without having to box them.

```rust
use smallbox::StackBox;

fn make_closure(s: String) -> StackBox<Fn()->String> {
    StackBox::new(move || format!("Hello, {}", s)).ok().unwrap()
}

let closure = make_closure("world!".to_owned());
assert_eq!(closure(), "Hello, world!");
```

`SmallBox<T>` is to eliminate heap alloction for small things, except that
the object is large enough to allocte. 
In addition, the inner `StackBox<T>` or `Box<T>` can be moved out by explicit pattern match.

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


# Capacity
The custom capacity of `SmallBox<T, Space>` and `StackBox<T，Space>` is expressed by the size of type **`Space`**, 
which default to `space::S2` representing as 2 words space (2 * usize). 
There are some default options in module `smallbox::space` from `S2` to `S64`. 
Anyway, you can defind your own space type, or just use some array.

The `resize()` method on `StackBox<T, Space>` and `SmallBox<T, Space>` is used to transforms themselves to the one of larger capacity.

```rust
use smallbox::SmallBox;
use smallbox::space::*;

let s = SmallBox::<[usize], S8>::new([0usize; 8]);
assert!(s.resize::<S16>().is_ok());
```


# Roadmap
- Static size check (since `mem::size_of()` has become const fn)


# Contribution

All kinds of contribution are welcome.

- **Issue** Feel free to open an issue when you find typos, bugs, or have any question.
- **Pull requests**. Better implementation, more tests, more documents and typo fixes are all welcome.


# License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.