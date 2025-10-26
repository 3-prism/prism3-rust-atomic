# Prism3 Atomic

[![CircleCI](https://circleci.com/gh/3-prism/rust-common.svg?style=shield)](https://circleci.com/gh/3-prism/rust-common)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/rust-common/badge.svg?branch=main)](https://coveralls.io/github/3-prism/rust-common?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-atomic.svg?color=blue)](https://crates.io/crates/prism3-atomic)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

User-friendly atomic operations wrapper providing JDK-like atomic API for Rust.

## Overview

Prism3 Atomic is a comprehensive atomic operations library that provides easy-to-use atomic types with reasonable default memory orderings, similar to Java's `java.util.concurrent.atomic` package. It hides the complexity of memory ordering while maintaining zero-cost abstraction and allowing advanced users to access underlying types for fine-grained control.

## Design Goals

- **Ease of Use**: Hides memory ordering complexity with reasonable defaults
- **Completeness**: Provides high-level operations similar to JDK atomic classes
- **Safety**: Guarantees memory safety and thread safety
- **Performance**: Zero-cost abstraction with no additional overhead
- **Flexibility**: Exposes underlying types via `inner()` for advanced users
- **Simplicity**: Minimal API surface without `_with_ordering` variants

## Features

### 🔢 **Atomic Integer Types**
- **Signed Integers**: `AtomicI8`, `AtomicI16`, `AtomicI32`, `AtomicI64`, `AtomicIsize`
- **Unsigned Integers**: `AtomicU8`, `AtomicU16`, `AtomicU32`, `AtomicU64`, `AtomicUsize`
- **Rich Operations**: increment, decrement, add, subtract, bitwise operations, max/min
- **Functional Updates**: `update_and_get`, `get_and_update`, `accumulate_and_get`

### 🔘 **Atomic Boolean Type**
- **AtomicBool**: Boolean atomic operations
- **Special Operations**: set, clear, negate, logical AND/OR/XOR
- **Conditional CAS**: `compare_and_set_if_false`, `compare_and_set_if_true`

### 🔢 **Atomic Floating-Point Types**
- **AtomicF32/AtomicF64**: 32-bit and 64-bit floating-point atomics
- **Arithmetic Operations**: add, subtract, multiply, divide (via CAS loop)
- **Functional Updates**: Custom operations via closures

### 🔗 **Atomic Reference Type**
- **AtomicRef<T>**: Thread-safe atomic reference using `Arc<T>`
- **Reference Updates**: Atomic swap and CAS operations
- **Functional Updates**: Transform references atomically

### 🎯 **Trait Abstractions**
- **Atomic**: Common atomic operations trait
- **UpdatableAtomic**: Functional update operations trait
- **AtomicInteger**: Integer-specific operations trait

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
prism3-atomic = "0.1.0"
```

## Quick Start

### Basic Counter

```rust
use prism3_atomic::AtomicI32;
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    // Spawn 10 threads, each increments counter 1000 times
    for _ in 0..10 {
        let counter = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter.increment_and_get();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify result
    assert_eq!(counter.get(), 10000);
    println!("Final count: {}", counter.get());
}
```

### CAS Loop

```rust
use prism3_atomic::AtomicI32;

fn increment_even_only(atomic: &AtomicI32) -> Result<i32, &'static str> {
    let mut current = atomic.get();
    loop {
        // Only increment even values
        if current % 2 != 0 {
            return Err("Value is odd");
        }

        let new = current + 2;
        match atomic.compare_and_set(current, new) {
            Ok(_) => return Ok(new),
            Err(actual) => current = actual, // Retry
        }
    }
}

fn main() {
    let atomic = AtomicI32::new(10);
    match increment_even_only(&atomic) {
        Ok(new_value) => println!("Successfully incremented to: {}", new_value),
        Err(e) => println!("Failed: {}", e),
    }
    assert_eq!(atomic.get(), 12);
}
```

### Functional Updates

```rust
use prism3_atomic::AtomicI32;

fn main() {
    let atomic = AtomicI32::new(10);

    // Update using a function
    let new_value = atomic.update_and_get(|x| {
        if x < 100 {
            x * 2
        } else {
            x
        }
    });

    assert_eq!(new_value, 20);
    println!("Updated value: {}", new_value);

    // Accumulate operation
    let result = atomic.accumulate_and_get(5, |a, b| a + b);
    assert_eq!(result, 25);
    println!("Accumulated value: {}", result);
}
```

### Atomic Reference

```rust
use prism3_atomic::AtomicRef;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct Config {
    timeout: u64,
    max_retries: u32,
}

fn main() {
    let config = Arc::new(Config {
        timeout: 1000,
        max_retries: 3,
    });

    let atomic_config = AtomicRef::new(config);

    // Update configuration
    let new_config = Arc::new(Config {
        timeout: 2000,
        max_retries: 5,
    });

    let old_config = atomic_config.swap(new_config);
    println!("Old config: {:?}", old_config);
    println!("New config: {:?}", atomic_config.get());

    // Update using a function
    atomic_config.update_and_get(|current| {
        Arc::new(Config {
            timeout: current.timeout * 2,
            max_retries: current.max_retries + 1,
        })
    });

    println!("Updated config: {:?}", atomic_config.get());
}
```

### Boolean Flag

```rust
use prism3_atomic::AtomicBool;
use std::sync::Arc;

struct Service {
    running: Arc<AtomicBool>,
}

impl Service {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    fn start(&self) {
        // Only start if not already running
        if self.running.compare_and_set_if_false(true).is_ok() {
            println!("Service started successfully");
        } else {
            println!("Service is already running");
        }
    }

    fn stop(&self) {
        // Only stop if currently running
        if self.running.compare_and_set_if_true(false).is_ok() {
            println!("Service stopped successfully");
        } else {
            println!("Service is already stopped");
        }
    }

    fn is_running(&self) -> bool {
        self.running.get()
    }
}

fn main() {
    let service = Service::new();

    service.start();
    assert!(service.is_running());

    service.start(); // Duplicate start will fail

    service.stop();
    assert!(!service.is_running());

    service.stop(); // Duplicate stop will fail
}
```

### Floating-Point Atomics

```rust
use prism3_atomic::AtomicF32;
use std::sync::Arc;
use std::thread;

fn main() {
    let sum = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

    // Spawn 10 threads, each adds 100 times
    for _ in 0..10 {
        let sum = sum.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                sum.add(0.01);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Note: Due to floating-point precision, result may not be exactly 10.0
    let result = sum.get();
    println!("Sum: {:.6}", result);
    println!("Error: {:.6}", (result - 10.0).abs());
}
```

## API Reference

### Common Operations (All Types)

| Method | Description | Memory Ordering |
|--------|-------------|-----------------|
| `new(value)` | Create new atomic | - |
| `get()` | Get current value | Acquire |
| `set(value)` | Set new value | Release |
| `swap(value)` | Swap value, return old | AcqRel |
| `compare_and_set(current, new)` | CAS operation, return Result | AcqRel/Acquire |
| `compare_and_exchange(current, new)` | CAS operation, return actual value | AcqRel/Acquire |
| `inner()` | Access underlying std type | - |

### Integer Operations

| Method | Description | Memory Ordering |
|--------|-------------|-----------------|
| `get_and_increment()` | Post-increment | Relaxed |
| `increment_and_get()` | Pre-increment | Relaxed |
| `get_and_decrement()` | Post-decrement | Relaxed |
| `decrement_and_get()` | Pre-decrement | Relaxed |
| `get_and_add(delta)` | Post-add | Relaxed |
| `add_and_get(delta)` | Pre-add | Relaxed |
| `get_and_sub(delta)` | Post-subtract | Relaxed |
| `sub_and_get(delta)` | Pre-subtract | Relaxed |
| `get_and_bitand(value)` | Bitwise AND | AcqRel |
| `get_and_bitor(value)` | Bitwise OR | AcqRel |
| `get_and_bitxor(value)` | Bitwise XOR | AcqRel |
| `get_and_max(value)` | Atomic max | AcqRel |
| `get_and_min(value)` | Atomic min | AcqRel |
| `update_and_get(f)` | Functional update | AcqRel |
| `get_and_update(f)` | Functional update | AcqRel |

### Boolean Operations

| Method | Description |
|--------|-------------|
| `get_and_set()` | Set to true, return old |
| `get_and_clear()` | Set to false, return old |
| `get_and_negate()` | Negate, return old |
| `get_and_logical_and(value)` | Logical AND |
| `get_and_logical_or(value)` | Logical OR |
| `get_and_logical_xor(value)` | Logical XOR |
| `compare_and_set_if_false(new)` | CAS if false |
| `compare_and_set_if_true(new)` | CAS if true |

### Floating-Point Operations

| Method | Description |
|--------|-------------|
| `add(delta)` | Atomic add (CAS loop) |
| `sub(delta)` | Atomic subtract (CAS loop) |
| `mul(factor)` | Atomic multiply (CAS loop) |
| `div(divisor)` | Atomic divide (CAS loop) |
| `update_and_get(f)` | Functional update |
| `get_and_update(f)` | Functional update |

## Memory Ordering Strategy

| Operation Type | Default Ordering | Reason |
|---------------|------------------|--------|
| **Pure Read** (`get()`) | `Acquire` | Ensure reading latest value |
| **Pure Write** (`set()`) | `Release` | Ensure write visibility |
| **Read-Modify-Write** (`swap()`, CAS) | `AcqRel` | Ensure both read and write correctness |
| **Counter Operations** (`increment_and_get()`) | `Relaxed` | Most scenarios only need count correctness |
| **Advanced API** (`update_and_get()`) | `AcqRel` | Ensure state consistency |

### Advanced Usage: Direct Access to Underlying Types

For scenarios requiring fine-grained memory ordering control (approximately 1% of use cases), use `inner()` to access the underlying standard library type:

```rust
use std::sync::atomic::Ordering;
use prism3_atomic::AtomicI32;

let atomic = AtomicI32::new(0);

// 99% of scenarios: use simple API
let value = atomic.get();

// 1% of scenarios: need fine-grained control
let value = atomic.inner().load(Ordering::Relaxed);
atomic.inner().store(42, Ordering::Release);
```

## Comparison with JDK

| Feature | JDK | Prism3 Atomic | Notes |
|---------|-----|---------------|-------|
| **Basic Types** | 3 types | 13 types | Rust supports more integer types |
| **Memory Ordering** | Implicit (volatile) | Default + `inner()` optional | Rust more flexible |
| **Weak CAS** | `weakCompareAndSet` | `compare_and_set_weak` | Equivalent |
| **Reference Type** | `AtomicReference<V>` | `AtomicRef<T>` | Rust uses `Arc<T>` |
| **Nullability** | Allows `null` | Use `Option<Arc<T>>` | Rust no null pointers |
| **Bitwise Operations** | Partial support | Full support | Rust more powerful |
| **Max/Min Operations** | Java 9+ support | Supported | Equivalent |
| **API Count** | ~20 methods/type | ~25 methods/type | Rust provides more convenience methods |

## Performance Considerations

### Zero-Cost Abstraction

All wrapper types use `#[repr(transparent)]` and `#[inline]` to ensure zero overhead after compilation:

```rust
// Our wrapper
let atomic = AtomicI32::new(0);
let value = atomic.get();

// Compiles to the same code as
let atomic = std::sync::atomic::AtomicI32::new(0);
let value = atomic.load(Ordering::Acquire);
```

### When to Use `inner()`

**99% of scenarios**: Use default API, which already provides optimal performance.

**1% of scenarios**: Use `inner()` only when:
- Extreme performance optimization (need `Relaxed` ordering)
- Complex lock-free algorithms (need precise memory ordering control)
- Interoperating with code that directly uses standard library types

**Golden Rule**: Default API first, `inner()` as last resort.

## Testing & Code Coverage

This project maintains comprehensive test coverage with detailed validation of all functionality.

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage report
./coverage.sh

# Generate text format report
./coverage.sh text

# Run CI checks (format, clippy, test, coverage)
./ci-check.sh
```

### Coverage Metrics

See [COVERAGE.md](COVERAGE.md) for detailed coverage statistics.

## Dependencies

This crate has **zero dependencies** for the core functionality, relying only on Rust's standard library.

## License

Copyright (c) 2025 3-Prism Co. Ltd. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

See [LICENSE](LICENSE) for the full license text.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

- Follow the Rust API guidelines
- Maintain comprehensive test coverage
- Document all public APIs with examples
- Ensure all tests pass before submitting PR

## Author

**Haixing Hu** - *3-Prism Co. Ltd.*

## Related Projects

- [prism3-rust-core](https://github.com/3-prism/rust-common/tree/main/prism3-rust-core) - Core utilities and data types
- [prism3-rust-concurrent](https://github.com/3-prism/rust-common/tree/main/prism3-rust-concurrent) - Concurrency utilities
- [prism3-rust-function](https://github.com/3-prism/rust-common/tree/main/prism3-rust-function) - Functional programming utilities

---

For more information about the Prism3 ecosystem, visit our [GitHub homepage](https://github.com/3-prism).

