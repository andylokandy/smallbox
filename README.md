Stack-allocated dynamically-sized types

# Overview
This crate provides a simple way of returnings DSTs up to a certain size without requiring a heap allocation.

# Basic usage
The core type is `StackDST<Trait>`, which represents a fixed-capacity allocation of an unsized type.
The `new` method on this type allows creating a instance from a concrete type, returning `Err(value)` if the instance is too large
for the allocated region.

# Example
One of the most obvious uses is to allow returning capturing closures without having to box them. In the example below, the closure
takes ownership of `value`, and is saved to a StackDST
```rust
use stack_dst::StackDST;

fn make_closure(value: u64) -> StackDST<Fn()->String> {
    StackDST::new(move || format!("Hello there! value={}", value)).ok().expect("Closure doesn't fit")
}
let closure = make_closure(12);
assert_eq!( closure(), "Hello there! value=12" );
```

# Status
- Works for most test cases
- Not rigourously tested
- No support for heap fallback


## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
