# SmallBox

[![CI Status](https://github.com/andylokandy/smallbox/actions/workflows/ci.yml/badge.svg)](https://github.com/andylokandy/smallbox/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/smallbox.svg)](https://crates.io/crates/smallbox)
[![Documentation](https://docs.rs/smallbox/badge.svg)](https://docs.rs/smallbox)
[![License](https://img.shields.io/crates/l/smallbox.svg)](https://github.com/andylokandy/smallbox#license)

A space-efficient alternative to `Box<T>` that stores small values on the stack and falls back to heap allocation for larger values. This optimization can significantly reduce memory allocations and improve performance for applications working with many small objects.

## Quick Start

Add SmallBox to your `Cargo.toml`:

```toml
[dependencies]
smallbox = "0.8"
```

### Basic Usage

```rust
use smallbox::SmallBox;
use smallbox::space::S4;

// Small values are stored on the stack
let small: SmallBox<[u32; 2], S4> = SmallBox::new([1, 2]);
assert!(!small.is_heap());

// Large values automatically use heap allocation  
let large: SmallBox<[u32; 32], S4> = SmallBox::new([0; 32]);
assert!(large.is_heap());

// Use like a regular Box
println!("Small: {:?}, Large: {:?}", *small, large.len());
```

# Benchmark

The test platform is Ubuntu 2204 on AMD Ryzen 9 7950X3D 16-Core Processor.

```
compare                             fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ box_large_item                   13.96 ns      │ 14.44 ns      │ 14.2 ns       │ 14.16 ns      │ 100     │ 12800
├─ box_small_item                   7.313 ns      │ 7.512 ns      │ 7.391 ns      │ 7.392 ns      │ 100     │ 25600
├─ smallbox_large_item_large_space  14.13 ns      │ 49.42 ns      │ 14.9 ns       │ 15.07 ns      │ 100     │ 12800
├─ smallbox_large_item_small_space  23.91 ns      │ 26.09 ns      │ 25 ns         │ 24.94 ns      │ 100     │ 6400
├─ smallbox_small_item_large_space  0.995 ns      │ 1.025 ns      │ 1.005 ns      │ 1.003 ns      │ 100     │ 102400
╰─ smallbox_small_item_small_space  0.985 ns      │ 1.015 ns      │ 0.995 ns      │ 0.996 ns      │ 100     │ 102400
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
