# qht-rs
[![Latest version](https://img.shields.io/crates/v/qht-rs.svg)](https://crates.io/crates/qht-rs)
[![Documentation](https://docs.rs/qht-rs/badge.svg)](https://docs.rs/qht-rs)
[![Build Status](https://travis-ci.org/ovheurdrive/qht-rs.svg?branch=master)](https://travis-ci.org/ovheurdrive/qht-rs)
![Long time support rustc version](https://img.shields.io/badge/rustc-1.31%2B-green.svg)
![License](https://img.shields.io/badge/License-MIT-blue.svg)


A Rust implementation of Quotient Hash Tables, a reasonably efficient approximate duplicate detection algorithm. [See paper here](https://arxiv.org/abs/1901.04358).

## How to use

```rust
extern crate qht;
extern crate rand;

use qht::*;
use rand::rngs::StdRng;
use rand::{FromEntropy, Rng};

fn main() {
    // Creates a new quotient hash table with 1 bucket and a fingerpint size of 3
    let mut f = QuotientHashTable::new(1024, 8, 3);
    // Initialize PRNG
    let mut rng = StdRng::from_entropy();

    let mut measured_collisions = 0;
    let mut actual_collisions = 0;
    let mut elements : Vec<u64> = Vec::with_capacity(1000);

    for _ in 0..10000 {
        // We generate a random element
        let element = rng.gen_range(0, 1000);

        // Check if it's in the Hash Table
        if f.lookup(element) {
            measured_collisions+=1;
        } else {
            // Insert it if it's not
            f.insert(element);
        }

        // Check if the element has been generated
        if elements.contains(&element) {
            actual_collisions+=1;
        } else {
            // Record it if it's not
            elements.push(element);
        }
    }

    println!("Measured Collisions {}, Actual Collisions {} ", measured_collisions, actual_collisions);
}

```

## Running the tests

Public functions are tested in their documentation.
Other miscellaneous tests are written in `lib.rs`.
Run the tests with

```
cargo test
```

## Running the benchmarks

The `Criterion` dependency is used to provide precise benchmarkings. Benchmarks can be run with
```
cargo bench
```

## Documentation

Generate the documentation with

```
cargo doc --no-deps
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details