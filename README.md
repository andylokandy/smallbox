# `smallbox`

[![Build Status](https://travis-ci.org/andylokandy/smallbox.svg?branch=master)](https://travis-ci.org/andylokandy/smallbox)
[![crates.io](https://img.shields.io/crates/v/smallbox.svg)](https://crates.io/crates/smallbox)
[![docs.rs](https://docs.rs/smallbox/badge.svg)](https://docs.rs/smallbox)


`Small Box` optimization: store small item on the stack and fallback to heap for large item.

## [**Documentation**](https://docs.rs/smallbox/)

 # Usage

 First, add the following to your `Cargo.toml`:

 ```toml
 [dependencies]
 smallbox = "0.5"
 ```

 Next, add this to your crate root:

 ```rust
 extern crate smallbox;
 ```

 If you want this crate to work with dynamic-sized type, you can request it via:

 ```toml
 [dependencies]
 smallbox = { version = "0.5", features = ["unsize"] }
 ```

 Currently `smallbox` by default links to the standard library, but if you would
 instead like to use this crate in a `#![no_std]` situation or crate, and want to
 opt out heap dependency and `SmallBox<T>` type, you can request this via:

 ```toml
 [dependencies.smallbox]
 version = "0.5"
 features = ["unsize"]
 default-features = false
 ```

 Enable `heap` feature for `#![no_std]` build to link to `alloc` crate
 and bring `SmallBox<T>` back.

 ```toml
 [dependencies.smallbox]
 version = "0.5"
 features = ["unsize", "heap"]
 default-features = false
 ```


 # Feature Flags

 This crate has the following cargo feature flags:

 - `std`
   - Optional, enabled by default
   - Use libstd


 - `heap`
   - Optional, enabled by default
   - Support heap fallback by including `SmallBox<T>`
   - If `std` feature flag is opted out, this will link
   `alloc` crate, and it need nightly rust for that.

 - `unsize`
   - Optional
   - Require nightly rust
   - Enable support for `DST` (dynamic-sized type).

 
 # Stable Rust

 The only possible way to use this crate on stable rust is to use the default feature flag, which means you can't use it in `no_std`
 environment or use it with `DST` (dynamic-sized type).

 # Unsized Type
 
 Once the feature `unsize` is enabled, the item type `T` of `SmallBox` and `StackBox` can
 and must be an unsized type, such as trait object or owned array slice. 

 # Overview
 This crate delivers two core type:

 - `SmallBox<T, Space>`: Stores `T` on heap or stack depending on the size of `T`. It takes `StackBox<T, Space>` as an varience to store small item, and then fallback to heap allocated `Box<T>` when type `T` is larger then the capacity of `Space`.

 - `StackBox<T, Space>`: Represents as a fixed-capacity allocation, and  stores item on stack.

 # Example

 Eliminate heap alloction for small items by `SmallBox`:

 ```rust
 use smallbox::SmallBox;
 use smallbox::space::S4;

 let small: SmallBox<_, S4> = SmallBox::new([0; 2]);
 let large: SmallBox<_, S4> = SmallBox::new([0; 32]);

 assert_eq!(small.len(), 2);
 assert_eq!(large.len(), 32);

 match small {
     SmallBox::Stack(val) => assert_eq!(*val, [0; 2]),
     _ => unreachable!()
 }

 match large {
     SmallBox::Box(val) => assert_eq!(*val, [0; 32]),
     _ => unreachable!()
 }
 ```

 ## DST

 The following examples requires `unsize` feature flag enabled.

 Trait object dynamic-dispatch:

 ```rust
 use smallbox::StackBox;
 use smallbox::space::S1;
  
 let val: StackBox<PartialEq<usize>, S1> = StackBox::new(5usize).unwrap();
  
 assert!(*val == 5)
 # }
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

# Benchmark

The test platform is Windows 10 on Intel E3 v1230 v3.

```
running 6 tests
test box_large_item                  ... bench:         102 ns/iter (+/- 15)
test box_small_item                  ... bench:          48 ns/iter (+/- 16)
test smallbox_large_item_large_space ... bench:          64 ns/iter (+/- 1)
test smallbox_large_item_small_space ... bench:         113 ns/iter (+/- 14)
test smallbox_small_item_large_space ... bench:          17 ns/iter (+/- 0)
test smallbox_small_item_small_space ... bench:           6 ns/iter (+/- 0)

test result: ok. 0 passed; 0 failed; 0 ignored; 6 measured; 0 filtered out
```

# Roadmap

- check size statically.
- provide `to_box()` for `SmallBox<T>` of `unsize` version.

# Contribution

All kinds of contribution are welcome.

- **Issue** Feel free to open an issue when you find typos, bugs, or have any question.
- **Pull requests**. Better implementation, more tests, more documents and typo fixes are all welcome.


# License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.