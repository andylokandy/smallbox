# `smallbox`

[![CI Status](https://github.com/andylokandy/smallbox/actions/workflows/ci.yml/badge.svg)](https://github.com/andylokandy/smallbox/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/smallbox.svg)](https://crates.io/crates/smallbox)

**Small Box** optimization: store small items on the stack and fall back to heap allocation for large items. Requires Rust 1.56+.

## [**Documentation**](https://andylokandy.github.io/smallbox)

# Usage

First, add the following to your `Cargo.toml`:

```toml
[dependencies]
smallbox = "0.8"
```

Next, add this to your crate root:

```rust
extern crate smallbox;
```

If you want this crate to work with dynamically-sized types, you can enable it via:

```toml
[dependencies]
smallbox = { version = "0.8", features = ["coerce"] }
```

Currently, `smallbox` by default links to the standard library, but if you would
instead like to use this crate in a `#![no_std]` situation or crate, you can request this via:

```toml
[dependencies.smallbox]
version = "0.8"
features = ["coerce"]
default-features = false
```

# Feature Flags

This crate has the following cargo feature flags:

- `std`
  - Optional, enabled by default
  - Use libstd
  - If the `std` feature flag is disabled, the `alloc` crate
    will be linked, which requires nightly Rust.

- `coerce`
  - Optional
  - Requires nightly Rust
  - Allow automatic coercion from sized `SmallBox` to unsized `SmallBox`.

# Unsized Types

There are two ways to create an unsized `SmallBox`: using the `smallbox!()` macro or coercing from a sized `SmallBox` instance (requires nightly compiler).

Using the `smallbox!()` macro is the only option on stable Rust. This macro will check the type of the given value and
the target type `T`. For any invalid type coercions, this macro will trigger a compile-time error.

Once the `coerce` feature is enabled, sized `SmallBox<T>` will be automatically coerced into `SmallBox<T: ?Sized>` if necessary.

# Example

Eliminate heap allocation for small items with `SmallBox`:

```rust
use smallbox::SmallBox;
use smallbox::space::S4;

let small: SmallBox<_, S4> = SmallBox::new([0; 2]);
let large: SmallBox<_, S4> = SmallBox::new([0; 32]);

assert_eq!(small.len(), 2);
assert_eq!(large.len(), 32);

assert_eq!(*small, [0; 2]);
assert_eq!(*large, [0; 32]);

assert_eq!(small.is_heap(), false);
assert_eq!(large.is_heap(), true);
```

## Unsized Types

Construct with the `smallbox!()` macro:

```rust
#[macro_use]
extern crate smallbox;

use smallbox::SmallBox;
use smallbox::space::*;

let array: SmallBox<[usize], S2> = smallbox!([0usize, 1]);

assert_eq!(array.len(), 2);
assert_eq!(*array, [0, 1]);
```

With the `coerce` feature:

```rust
use smallbox::SmallBox;
use smallbox::space::*;
 
let array: SmallBox<[usize], S2> = SmallBox::new([0usize, 1]);

assert_eq!(array.len(), 2);
assert_eq!(*array, [0, 1]);
```

`Any` downcasting:

```rust
#[macro_use]
extern crate smallbox;

use std::any::Any;
use smallbox::SmallBox;
use smallbox::space::S2;

let num: SmallBox<dyn Any, S2> = smallbox!(1234u32);

if let Some(num) = num.downcast_ref::<u32>() {
    assert_eq!(*num, 1234);
} else {
    unreachable!();
}
```

# Capacity

The capacity is expressed by the size of the type parameter `Space`,
regardless of what the `Space` actually is.

The crate provides some space types in the `smallbox::space` module,
from `S1`, `S2`, `S4` to `S64`, representing `"n * usize"` spaces.

You can also define your own space type,
such as a byte array `[u8; 64]`.
Please note that space alignment is also important. If the alignment
of the space is smaller than the alignment of the value, the value
will be stored on the heap.

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

All kinds of contributions are welcome.

- **Issues** Feel free to open an issue when you find typos, bugs, or have any questions.
- **Pull requests** Better implementations, more tests, more documentation, and typo fixes are all welcome.

# License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
