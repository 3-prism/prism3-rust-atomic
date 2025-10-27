# Prism3 Atomic

[![CircleCI](https://circleci.com/gh/3-prism/rust-common.svg?style=shield)](https://circleci.com/gh/3-prism/rust-common)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/rust-common/badge.svg?branch=main)](https://coveralls.io/github/3-prism/rust-common?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-atomic.svg?color=blue)](https://crates.io/crates/prism3-atomic)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

为 Rust 提供类似 JDK 的用户友好原子操作封装。

## 概述

Prism3 Atomic 是一个全面的原子操作库，提供易于使用的原子类型和合理的默认内存序，类似于 Java 的 `java.util.concurrent.atomic` 包。它隐藏了内存序的复杂性，同时保持零成本抽象，并允许高级用户访问底层类型以进行细粒度控制。

## 设计目标

- **易用性**：通过合理的默认值隐藏内存序复杂性
- **完整性**：提供类似 JDK atomic 类的高级操作
- **安全性**：保证内存安全和线程安全
- **性能**：零成本抽象，无额外开销
- **灵活性**：通过 `inner()` 方法暴露底层类型供高级用户使用
- **简洁性**：最小化 API 表面积，不提供 `_with_ordering` 变体

## 特性

### 🔢 **原子整数类型**
- **有符号整数**：`AtomicI8`、`AtomicI16`、`AtomicI32`、`AtomicI64`、`AtomicIsize`
- **无符号整数**：`AtomicU8`、`AtomicU16`、`AtomicU32`、`AtomicU64`、`AtomicUsize`
- **丰富的操作**：自增、自减、加法、减法、乘法、除法、位运算、最大值/最小值
- **函数式更新**：`fetch_update`、`fetch_accumulate`

### 🔘 **原子布尔类型**
- **AtomicBool**：布尔原子操作
- **特殊操作**：设置、清除、取反、逻辑与/或/异或
- **条件 CAS**：`set_if_false`、`set_if_true`

### 🔢 **原子浮点数类型**
- **AtomicF32/AtomicF64**：32 位和 64 位浮点数原子操作
- **算术操作**：`fetch_add`、`fetch_sub`、`fetch_mul`、`fetch_div`（通过 CAS 循环实现）
- **函数式更新**：通过闭包进行自定义操作

### 🔗 **原子引用类型**
- **AtomicRef<T>**：使用 `Arc<T>` 的线程安全原子引用
- **引用更新**：原子交换和 CAS 操作
- **函数式更新**：原子地转换引用

### 🎯 **Trait 抽象**
- **Atomic**：通用原子操作 trait（包含 `fetch_update`）
- **AtomicNumber**：数字类型算术操作 trait（整数和浮点数通用）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
prism3-atomic = "0.1.0"
```

## 快速开始

### 基础计数器

```rust
use prism3_atomic::AtomicI32;
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    // 启动 10 个线程，每个线程递增计数器 1000 次
    for _ in 0..10 {
        let counter = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter.fetch_inc();
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证结果
    assert_eq!(counter.load(), 10000);
    println!("最终计数：{}", counter.load());
}
```

### CAS 循环

```rust
use prism3_atomic::AtomicI32;

fn increment_even_only(atomic: &AtomicI32) -> Result<i32, &'static str> {
    let mut current = atomic.load();
    loop {
        // 只对偶数值进行递增
        if current % 2 != 0 {
            return Err("值为奇数");
        }

        let new = current + 2;
        match atomic.compare_set(current, new) {
            Ok(_) => return Ok(new),
            Err(actual) => current = actual, // 重试
        }
    }
}

fn main() {
    let atomic = AtomicI32::new(10);
    match increment_even_only(&atomic) {
        Ok(new_value) => println!("成功递增到：{}", new_value),
        Err(e) => println!("失败：{}", e),
    }
    assert_eq!(atomic.load(), 12);
}
```

### 函数式更新

```rust
use prism3_atomic::AtomicI32;

fn main() {
    let atomic = AtomicI32::new(10);

    // 使用函数更新（返回旧值）
    let old_value = atomic.fetch_update(|x| {
        if x < 100 {
            x * 2
        } else {
            x
        }
    });

    assert_eq!(old_value, 10);
    assert_eq!(atomic.load(), 20);
    println!("更新后的值：{}", atomic.load());

    // 累积操作（返回旧值）
    let old_result = atomic.fetch_accumulate(5, |a, b| a + b);
    assert_eq!(old_result, 20);
    assert_eq!(atomic.load(), 25);
    println!("累积后的值：{}", atomic.load());
}
```

### 原子引用

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

    // 更新配置
    let new_config = Arc::new(Config {
        timeout: 2000,
        max_retries: 5,
    });

    let old_config = atomic_config.swap(new_config);
    println!("旧配置：{:?}", old_config);
    println!("新配置：{:?}", atomic_config.load());

    // 使用函数更新（返回旧值）
    let old = atomic_config.fetch_update(|current| {
        Arc::new(Config {
            timeout: current.timeout * 2,
            max_retries: current.max_retries + 1,
        })
    });

    println!("更新前的配置：{:?}", old);
    println!("更新后的配置：{:?}", atomic_config.load());
}
```

### 布尔标志

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
        // 只有当前未运行时才启动
        if self.running.set_if_false(true).is_ok() {
            println!("服务启动成功");
        } else {
            println!("服务已经在运行");
        }
    }

    fn stop(&self) {
        // 只有当前运行时才停止
        if self.running.set_if_true(false).is_ok() {
            println!("服务停止成功");
        } else {
            println!("服务已经停止");
        }
    }

    fn is_running(&self) -> bool {
        self.running.load()
    }
}

fn main() {
    let service = Service::new();

    service.start();
    assert!(service.is_running());

    service.start(); // 重复启动会失败

    service.stop();
    assert!(!service.is_running());

    service.stop(); // 重复停止会失败
}
```

### 浮点数原子操作

```rust
use prism3_atomic::AtomicF32;
use std::sync::Arc;
use std::thread;

fn main() {
    let sum = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

    // 启动 10 个线程，每个线程累加 100 次
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

    // 注意：由于浮点数精度问题，结果可能不是精确的 10.0
    let result = sum.load();
    println!("累加结果：{:.6}", result);
    println!("误差：{:.6}", (result - 10.0).abs());
}
```

## API 参考

### 通用操作（所有类型）

| 方法 | 描述 | 内存序 |
|-----|------|--------|
| `new(value)` | 创建新的原子值 | - |
| `load()` | 加载当前值 | Acquire |
| `store(value)` | 存储新值 | Release |
| `swap(value)` | 交换值，返回旧值 | AcqRel |
| `compare_set(current, new)` | CAS 操作，返回 Result | AcqRel/Acquire |
| `compare_set_weak(current, new)` | 弱 CAS，返回 Result | AcqRel/Acquire |
| `compare_and_exchange(current, new)` | CAS 操作，返回实际值 | AcqRel/Acquire |
| `compare_and_exchange_weak(current, new)` | 弱 CAS，返回实际值 | AcqRel/Acquire |
| `fetch_update(f)` | 函数式更新，返回旧值 | AcqRel/Acquire |
| `inner()` | 访问底层标准库类型 | - |

### 整数操作

| 方法 | 描述 | 内存序 |
|-----|------|--------|
| `fetch_inc()` | 后增，返回旧值 | Relaxed |
| `fetch_dec()` | 后减，返回旧值 | Relaxed |
| `fetch_add(delta)` | 后加，返回旧值 | Relaxed |
| `fetch_sub(delta)` | 后减，返回旧值 | Relaxed |
| `fetch_mul(factor)` | 后乘，返回旧值 | AcqRel（CAS 循环） |
| `fetch_div(divisor)` | 后除，返回旧值 | AcqRel（CAS 循环） |
| `fetch_and(value)` | 按位与，返回旧值 | AcqRel |
| `fetch_or(value)` | 按位或，返回旧值 | AcqRel |
| `fetch_xor(value)` | 按位异或，返回旧值 | AcqRel |
| `fetch_not()` | 按位取反，返回旧值 | AcqRel |
| `fetch_max(value)` | 原子取最大值，返回旧值 | AcqRel |
| `fetch_min(value)` | 原子取最小值，返回旧值 | AcqRel |
| `fetch_update(f)` | 函数式更新，返回旧值 | AcqRel/Acquire |
| `fetch_accumulate(x, f)` | 累积，返回旧值 | AcqRel/Acquire |

### 布尔操作

| 方法 | 描述 | 内存序 |
|-----|------|--------|
| `fetch_set()` | 设置为 true，返回旧值 | AcqRel |
| `fetch_clear()` | 设置为 false，返回旧值 | AcqRel |
| `fetch_not()` | 取反，返回旧值 | AcqRel |
| `fetch_and(value)` | 逻辑与，返回旧值 | AcqRel |
| `fetch_or(value)` | 逻辑或，返回旧值 | AcqRel |
| `fetch_xor(value)` | 逻辑异或，返回旧值 | AcqRel |
| `set_if_false(new)` | 如果为 false 则 CAS | AcqRel/Acquire |
| `set_if_true(new)` | 如果为 true 则 CAS | AcqRel/Acquire |

### 浮点数操作

| 方法 | 描述 | 内存序 |
|-----|------|--------|
| `fetch_add(delta)` | 原子加法，返回旧值 | AcqRel（CAS 循环） |
| `fetch_sub(delta)` | 原子减法，返回旧值 | AcqRel（CAS 循环） |
| `fetch_mul(factor)` | 原子乘法，返回旧值 | AcqRel（CAS 循环） |
| `fetch_div(divisor)` | 原子除法，返回旧值 | AcqRel（CAS 循环） |
| `fetch_update(f)` | 函数式更新，返回旧值 | AcqRel/Acquire |

## 内存序策略

| 操作类型 | 默认内存序 | 原因 |
|---------|-----------|------|
| **纯读操作** (`load()`) | `Acquire` | 保证读取最新值 |
| **纯写操作** (`store()`) | `Release` | 保证写入可见 |
| **读-改-写操作** (`swap()`、CAS) | `AcqRel` | 同时保证读和写的正确性 |
| **计数器操作** (`fetch_inc()`、`fetch_add()`) | `Relaxed` | 纯计数场景，无需同步其他数据 |
| **位运算操作** (`fetch_and()`、`fetch_or()`) | `AcqRel` | 通常用于标志位同步 |
| **最大/最小值操作** (`fetch_max()`、`fetch_min()`) | `AcqRel` | 常与阈值判断配合使用 |
| **函数式更新** (`fetch_update()`) | `AcqRel` / `Acquire` | CAS 循环标准语义 |

### 高级用法：直接访问底层类型

对于需要精细控制内存序的场景（约 1% 的使用情况），通过 `inner()` 方法访问底层标准库类型：

```rust
use std::sync::atomic::Ordering;
use prism3_atomic::AtomicI32;

let atomic = AtomicI32::new(0);

// 99% 的场景：使用简单 API
let value = atomic.load();

// 1% 的场景：需要精细控制
let value = atomic.inner().load(Ordering::Relaxed);
atomic.inner().store(42, Ordering::Release);
```

## 与 JDK 对比

| 特性 | JDK | Prism3 Atomic | 说明 |
|-----|-----|---------------|------|
| **基础类型** | 3 种类型 | 13 种类型 | Rust 支持更多整数类型 |
| **内存序** | 隐式（volatile 语义） | 默认 + `inner()` 可选 | Rust 更灵活 |
| **弱 CAS** | `weakCompareAndSet` | `compare_and_set_weak` | 等价 |
| **引用类型** | `AtomicReference<V>` | `AtomicRef<T>` | Rust 使用 `Arc<T>` |
| **可空性** | 允许 `null` | 使用 `Option<Arc<T>>` | Rust 不允许空指针 |
| **位运算** | 部分支持 | 完整支持 | Rust 更强大 |
| **最大/最小值** | Java 9+ 支持 | 支持 | 等价 |
| **API 数量** | 约 20 个方法/类型 | 约 25 个方法/类型 | Rust 提供更多便利方法 |

## 性能考虑

### 零成本抽象

所有封装类型都使用 `#[repr(transparent)]` 和 `#[inline]` 确保编译后零开销：

```rust
// 我们的封装
let atomic = AtomicI32::new(0);
let value = atomic.load();

// 编译后与以下代码生成相同的机器码
let atomic = std::sync::atomic::AtomicI32::new(0);
let value = atomic.load(Ordering::Acquire);
```

### 何时使用 `inner()`

**99% 的场景**：使用默认 API，已经提供最优性能。

**1% 的场景**：只有在以下情况才使用 `inner()`：
- 极致性能优化（需要使用 `Relaxed` 内存序）
- 复杂的无锁算法（需要精确控制内存序）
- 与直接使用标准库的代码互操作

**黄金法则**：默认 API 优先，`inner()` 是最后的手段。

## 测试与代码覆盖率

本项目保持全面的测试覆盖，对所有功能进行详细验证。

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行覆盖率报告
./coverage.sh

# 生成文本格式报告
./coverage.sh text

# 运行 CI 检查（格式化、clippy、测试、覆盖率）
./ci-check.sh
```

### 覆盖率指标

详细的覆盖率统计请参见 [COVERAGE.zh_CN.md](COVERAGE.zh_CN.md)。

## 依赖项

此 crate 的核心功能**零依赖**，仅依赖 Rust 标准库。

## 许可证

Copyright (c) 2025 3-Prism Co. Ltd. All rights reserved.

根据 Apache 许可证 2.0 版（"许可证"）授权；
除非遵守许可证，否则您不得使用此文件。
您可以在以下位置获取许可证副本：

    http://www.apache.org/licenses/LICENSE-2.0

除非适用法律要求或书面同意，否则根据许可证分发的软件
按"原样"分发，不附带任何明示或暗示的担保或条件。
有关许可证下的特定语言管理权限和限制，请参阅许可证。

完整的许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发指南

- 遵循 Rust API 指南
- 保持全面的测试覆盖
- 为所有公共 API 编写文档和示例
- 提交 PR 前确保所有测试通过

## 作者

**胡海星** - *棱芯科技有限公司*

## 相关项目

- [prism3-rust-core](https://github.com/3-prism/rust-common/tree/main/prism3-rust-core) - 核心工具和数据类型
- [prism3-rust-concurrent](https://github.com/3-prism/rust-common/tree/main/prism3-rust-concurrent) - 并发工具
- [prism3-rust-function](https://github.com/3-prism/rust-common/tree/main/prism3-rust-function) - 函数式编程工具

---

有关 Prism3 生态系统的更多信息，请访问我们的 [GitHub 主页](https://github.com/3-prism)。

