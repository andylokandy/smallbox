# `smallbox`

[![Build Status](https://travis-ci.org/andylokandy/smallbox.svg?branch=master)](https://travis-ci.org/andylokandy/smallbox)
[![crates.io](https://img.shields.io/crates/v/smallbox.svg)](https://crates.io/crates/smallbox)
[![docs.rs](https://docs.rs/smallbox/badge.svg)](https://docs.rs/smallbox)


`Small Box` optimization: store small item on the stack or fallback to heap for large item.

## [**Documentation**](https://docs.rs/smallbox/)

 # Usage

First, add the following to your `Cargo.toml`:

```toml
[dependencies]
smallbox = "0.6"
```

Next, add this to your crate root:

```rust
extern crate smallbox;
```

If you want this crate to work with dynamic-sized type, you can request it via:

```toml
[dependencies]
smallbox = { version = "0.6", features = ["unsize"] }
```

Currently `smallbox` by default links to the standard library, but if you would
instead like to use this crate in a `#![no_std]` situation or crate,
you can request this via, this would link `alloc` crate and requires nightly rust:

```toml
[dependencies.smallbox]
version = "0.6"
features = ["unsize"]
default-features = false
```


# Feature Flags

This crate has the following cargo feature flags:

- `std`
  - Optional, enabled by default
  - Use libstd
  - If `std` feature flag is opted out, `alloc` crate
    will be linked, which would require nightly rust.

- `unsize`
  - Optional
  - Require nightly rust
  - Enable support for `DST` (dynamic-sized type).


# Stable Rust

The only possible way to use this crate on stable rust is to use the default feature flag, which means you can't use it in `no_std`
environment or use it with `DST` (dynamic-sized type).

# Unsized Type

Once the feature `unsize` is enabled, sized `SmallBox<T>` can be coerced into `SmallBox<T: ?Sized>` if necessary.

# Example

Eliminate heap alloction for small items by `SmallBox`:

```rust
use smallbox::SmallBox;
use smallbox::space::S4;

let small: SmallBox<_, S4> = SmallBox::new([0; 2]);
let large: SmallBox<_, S4> = SmallBox::new([0; 32]);

assert_eq!(small.len(), 2);
assert_eq!(large.len(), 32);

assert_eq!(*small, [0; 2]);
assert_eq!(*large, [0; 32]);

assert!(small.heaped() == false);
assert!(large.heaped() == true);
```

## DST

The following examples requires `unsize` feature flag enabled.

Trait object dynamic-dispatch:

```rust
use smallbox::SmallBox;
use smallbox::space::S1;
 
let val: SmallBox<PartialEq<usize>, S1> = SmallBox::new(5usize);
 
assert!(*val == 5)
```

`Any` downcasting:

```rust
use std::any::Any;
use smallbox::SmallBox;
use smallbox::space::S2;

let num: SmallBox<Any, S2> = SmallBox::new(1234u32);

if let Some(num) = num.downcast_ref::<u32>() {
    assert_eq!(*num, 1234);
} else {
    unreachable!();
}
```


# Capacity

The capacity of `SmallBox<T, Space>` is expressed by the size of type parameter **`Space`**, 
regardless of what the `Space` actually is.

This crate provides some spaces in module `smallbox::space`, 
from `S1`, `S2`, `S4` to `S64`, representing `"n * usize"` spaces.

Anyway, you can defind your own space type, 
such as a byte array `[u8; 64]`.

The `resize()` method on `SmallBox` is used to change its capacity.

```rust
use smallbox::SmallBox;
use smallbox::space::{S8, S16};

let s: SmallBox::<_, S8> = SmallBox::new([0usize; 8]);
let m: SmallBox<_, S16> = s.resize();
```

# Benchmark

The test platform is Windows 10 on Intel E3 v1230 v3.

```
running 6 tests
test box_large_item                  ... bench:         104 ns/iter (+/- 14)
test box_small_item                  ... bench:          49 ns/iter (+/- 5)
test smallbox_large_item_large_space ... bench:          52 ns/iter (+/- 6)
test smallbox_large_item_small_space ... bench:         106 ns/iter (+/- 25)
test smallbox_small_item_large_space ... bench:          18 ns/iter (+/- 1)
test smallbox_small_item_small_space ... bench:           2 ns/iter (+/- 0)

test result: ok. 0 passed; 0 failed; 0 ignored; 6 measured; 0 filtered out
```


# Contribution

All kinds of contribution are welcome.

- **Issue** Feel free to open an issue when you find typos, bugs, or have any question.
- **Pull requests**. Better implementation, more tests, more documents and typo fixes are all welcome.


# License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.