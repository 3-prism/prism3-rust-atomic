# Atomic 封装设计文档 v2.0

## 1. 背景与目标

### 1.1 背景

Rust 标准库的 `std::sync::atomic` 提供了底层的原子类型，但使用起来存在一些不便：

1. **显式 Ordering 要求**：每次操作都需要显式指定内存序（`Ordering::Relaxed`、`Ordering::Acquire`、`Ordering::Release` 等），增加了使用复杂度
2. **API 较为底层**：缺少常见的高级操作（如 `getAndIncrement`、`incrementAndGet` 等）
3. **易用性不足**：对于大多数场景，开发者只需要"正确"的原子操作，而不需要关心底层内存序细节

相比之下，JDK 的 atomic 包（`java.util.concurrent.atomic`）提供了更友好的 API：

```java
// Java 示例
AtomicInteger counter = new AtomicInteger(0);
int old = counter.getAndIncrement();  // 自动使用正确的内存序
int current = counter.incrementAndGet();
boolean success = counter.compareAndSet(expected, newValue);
```

### 1.2 目标

设计一套 Rust 的 atomic 封装，使其：

1. **易用性**：隐藏 `Ordering` 复杂性，提供合理的默认内存序
2. **完整性**：提供与 JDK atomic 类似的高级操作方法
3. **安全性**：保证内存安全和线程安全
4. **性能**：零成本抽象，不引入额外开销
5. **灵活性**：通过 `inner()` 方法暴露底层类型，高级用户可直接操作标准库类型
6. **简洁性**：API 表面积小，不提供 `_with_ordering` 变体以避免 API 膨胀

### 1.3 非目标

- 不改变 Rust 的内存模型
- 不引入新的同步原语
- 不提供跨进程的原子操作

## 2. 内存序策略

### 2.1 内存序概述

Rust 提供了五种内存序：

| 内存序 | 说明 | 适用场景 |
|-------|------|---------|
| `Relaxed` | 只保证原子性，不保证顺序 | 性能关键场景，如计数器 |
| `Acquire` | 读操作，防止后续读写被重排到此操作之前 | 读取共享状态 |
| `Release` | 写操作，防止之前读写被重排到此操作之后 | 更新共享状态 |
| `AcqRel` | 同时具有 Acquire 和 Release 语义 | 读-改-写操作 |
| `SeqCst` | 最强保证，全局顺序一致性 | 需要严格顺序的场景 |

### 2.2 默认策略

为平衡易用性、正确性和性能，我们采用以下默认策略：

| 操作类型 | 默认 Ordering | 原因 |
|---------|--------------|------|
| **纯读操作** | `Acquire` | 保证读取最新值，防止后续操作被重排 |
| **纯写操作** | `Release` | 保证写入可见，防止之前操作被重排 |
| **读-改-写操作** | `AcqRel` | 同时保证读和写的正确性 |
| **比较并交换** | `AcqRel`（成功）+ `Acquire`（失败）| 标准 CAS 语义 |

**特殊情况**：

- **计数器操作**（如 `increment`、`decrement`）：使用 `Relaxed`，因为大多数场景下只需要保证计数正确，不需要同步其他状态
- **高级 API**（如 `updateAndGet`）：使用 `AcqRel`，保证函数内的状态一致性

### 2.3 高级场景：直接访问底层类型

对于需要精细控制内存序的场景（约 1% 的使用情况），通过 `inner()` 方法访问底层标准库类型：

```rust
use std::sync::atomic::Ordering;

let atomic = AtomicI32::new(0);

// 99% 的场景：使用简单 API
let value = atomic.get();

// 1% 的场景：需要精细控制，直接操作底层类型
let value = atomic.inner().load(Ordering::Relaxed);
atomic.inner().store(42, Ordering::Release);
```

**设计理念**：我们不提供所有方法的 `_with_ordering` 变体，因为：
1. 避免 API 膨胀（否则方法数量翻倍）
2. 防止误用（用户可能不理解内存序）
3. 保持简洁性（符合"易用封装"的定位）
4. `inner()` 是完美的 escape hatch（高级用户清楚知道自己在做什么）

## 3. 类型设计

### 3.1 封装类型概览

| Rust 封装类型 | 底层类型 | JDK 对应类型 | 说明 |
|--------------|---------|-------------|------|
| `AtomicBool` | `std::sync::atomic::AtomicBool` | `AtomicBoolean` | 原子布尔值 |
| `AtomicI8` | `std::sync::atomic::AtomicI8` | - | 8位有符号整数 |
| `AtomicU8` | `std::sync::atomic::AtomicU8` | - | 8位无符号整数 |
| `AtomicI16` | `std::sync::atomic::AtomicI16` | - | 16位有符号整数 |
| `AtomicU16` | `std::sync::atomic::AtomicU16` | - | 16位无符号整数 |
| `AtomicI32` | `std::sync::atomic::AtomicI32` | `AtomicInteger` | 32位有符号整数 |
| `AtomicU32` | `std::sync::atomic::AtomicU32` | - | 32位无符号整数 |
| `AtomicI64` | `std::sync::atomic::AtomicI64` | `AtomicLong` | 64位有符号整数 |
| `AtomicU64` | `std::sync::atomic::AtomicU64` | - | 64位无符号整数 |
| `AtomicIsize` | `std::sync::atomic::AtomicIsize` | - | 指针大小的有符号整数 |
| `AtomicUsize` | `std::sync::atomic::AtomicUsize` | - | 指针大小的无符号整数 |
| `AtomicF32` | `std::sync::atomic::AtomicU32` + 位转换 | - | 32位浮点数（特殊实现） |
| `AtomicF64` | `std::sync::atomic::AtomicU64` + 位转换 | - | 64位浮点数（特殊实现） |
| `AtomicRef<T>` | `std::sync::atomic::AtomicPtr<T>` + `Arc<T>` | `AtomicReference<V>` | 原子引用 |

**注意**：我们直接使用 `std::sync::atomic` 的类型名，通过模块路径区分：

```rust
// 标准库类型
use std::sync::atomic::AtomicI32 as StdAtomicI32;

// 我们的封装类型
use prism3_rust_concurrent::atomic::AtomicI32;
```

### 3.2 核心结构

```rust
/// 原子整数封装（以 AtomicI32 为例）
///
/// 提供易用的原子操作 API，自动使用合理的内存序。
#[repr(transparent)]
pub struct AtomicI32 {
    inner: std::sync::atomic::AtomicI32,
}

// 自动实现的 trait
unsafe impl Send for AtomicI32 {}
unsafe impl Sync for AtomicI32 {}

impl Default for AtomicI32 {
    fn default() -> Self {
        Self::new(0)
    }
}

impl From<i32> for AtomicI32 {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for AtomicI32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicI32")
            .field("value", &self.get())
            .finish()
    }
}

impl fmt::Display for AtomicI32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}
```

### 3.3 Trait 实现

所有原子类型都应实现以下 trait：

| Trait | 说明 | JDK 对应 |
|-------|------|---------|
| `Send` | 可跨线程转移 | 自动满足 |
| `Sync` | 可跨线程共享 | 自动满足 |
| `Default` | 默认值构造 | - |
| `Debug` | 调试输出 | `toString()` |
| `Display` | 格式化输出 | `toString()` |
| `From<T>` | 类型转换 | 构造函数 |

**不实现的 trait**：
- `Clone`：原子类型不应该被克隆（但 `AtomicRef` 可以）
- `PartialEq`/`Eq`：比较原子类型的值需要读取，可能产生误解
- `PartialOrd`/`Ord`：同上
- `Hash`：同上

**原因**：实现这些 trait 会隐藏读取操作，用户应该显式调用 `get()` 或 `inner().load()`。

```rust
// ❌ 误导性的代码
if atomic1 == atomic2 {  // 这看起来像简单比较，但实际是两次原子读取
    // ...
}

// ✅ 明确的代码
if atomic1.get() == atomic2.get() {  // 清楚地表明这是两次独立的读取
    // ...
}
```

### 3.4 设计原则

1. **零成本抽象**：封装不引入额外开销，内联所有方法
2. **类型安全**：利用 Rust 类型系统防止误用
3. **所有权友好**：支持 `Send + Sync`，可安全跨线程共享
4. **trait 统一**：通过 trait 提供统一接口
5. **显式优于隐式**：不实现可能误导的 trait（如 `PartialEq`）

## 4. API 设计

### 4.1 基础操作

所有原子类型都提供以下基础操作：

```rust
impl AtomicI32 {
    /// 创建新的原子整数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(42);
    /// ```
    pub const fn new(value: i32) -> Self;

    /// 获取当前值（使用 Acquire ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(42);
    /// assert_eq!(atomic.get(), 42);
    /// ```
    pub fn get(&self) -> i32;

    /// 设置新值（使用 Release ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(0);
    /// atomic.set(42);
    /// assert_eq!(atomic.get(), 42);
    /// ```
    pub fn set(&self, value: i32);

    /// 交换值，返回旧值（使用 AcqRel ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.swap(20);
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 20);
    /// ```
    pub fn swap(&self, value: i32) -> i32;

    /// 比较并交换（CAS）
    ///
    /// 如果当前值等于 `current`，则设置为 `new`，返回 `Ok(())`；
    /// 否则返回 `Err(actual)`，其中 `actual` 是实际的当前值。
    ///
    /// # 参数
    ///
    /// * `current` - 期望的当前值
    /// * `new` - 要设置的新值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    ///
    /// // 成功的 CAS
    /// assert!(atomic.compare_and_set(10, 20).is_ok());
    /// assert_eq!(atomic.get(), 20);
    ///
    /// // 失败的 CAS
    /// match atomic.compare_and_set(10, 30) {
    ///     Ok(_) => panic!("Should fail"),
    ///     Err(actual) => assert_eq!(actual, 20),
    /// }
    /// ```
    pub fn compare_and_set(&self, current: i32, new: i32) -> Result<(), i32>;

    /// 弱版本的 CAS（允许虚假失败，但在某些平台上性能更好）
    ///
    /// 主要用于循环中的 CAS 操作。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    ///
    /// // 在循环中使用弱 CAS
    /// let mut current = atomic.get();
    /// loop {
    ///     let new = current + 1;
    ///     match atomic.compare_and_set_weak(current, new) {
    ///         Ok(_) => break,
    ///         Err(actual) => current = actual,
    ///     }
    /// }
    /// assert_eq!(atomic.get(), 11);
    /// ```
    pub fn compare_and_set_weak(&self, current: i32, new: i32) -> Result<(), i32>;

    /// 比较并交换，返回交换前的实际值
    ///
    /// 如果当前值等于 `current`，则设置为 `new`，返回 `current`（成功）；
    /// 否则返回实际的当前值（失败）。
    ///
    /// 与 `compare_and_set` 的区别：
    /// - `compare_and_set` 返回 `Result<(), i32>`，成功返回 `Ok(())`，失败返回 `Err(actual)`
    /// - `compare_and_exchange` 总是返回交换前的实际值，调用者通过比较返回值判断是否成功
    ///
    /// # 参数
    ///
    /// * `current` - 期望的当前值
    /// * `new` - 要设置的新值
    ///
    /// # 返回值
    ///
    /// 返回交换前的实际值。如果返回值等于 `current`，说明交换成功；否则交换失败。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    ///
    /// // 成功的交换
    /// let prev = atomic.compare_and_exchange(10, 20);
    /// assert_eq!(prev, 10); // 返回旧值，说明成功
    /// assert_eq!(atomic.get(), 20);
    ///
    /// // 失败的交换
    /// let prev = atomic.compare_and_exchange(10, 30);
    /// assert_eq!(prev, 20); // 返回实际值（不是期望的 10），说明失败
    /// assert_eq!(atomic.get(), 20); // 值未改变
    ///
    /// // 在 CAS 循环中使用（更简洁）
    /// let mut current = atomic.get();
    /// loop {
    ///     let new = current * 2;
    ///     let prev = atomic.compare_and_exchange(current, new);
    ///     if prev == current {
    ///         // 成功
    ///         break;
    ///     }
    ///     // 失败，prev 就是最新值，直接用于下次重试
    ///     current = prev;
    /// }
    /// assert_eq!(atomic.get(), 40);
    /// ```
    ///
    /// # 与 compare_and_set 的对比
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    ///
    /// // 使用 compare_and_set（需要处理 Result）
    /// let mut current = atomic.get();
    /// loop {
    ///     match atomic.compare_and_set(current, current + 1) {
    ///         Ok(_) => break,
    ///         Err(actual) => current = actual,
    ///     }
    /// }
    ///
    /// // 使用 compare_and_exchange（更直接）
    /// let mut current = atomic.get();
    /// loop {
    ///     let prev = atomic.compare_and_exchange(current, current + 1);
    ///     if prev == current {
    ///         break;
    ///     }
    ///     current = prev;
    /// }
    /// ```
    pub fn compare_and_exchange(&self, current: i32, new: i32) -> i32;

    /// 弱版本的 compare_and_exchange（允许虚假失败）
    ///
    /// 与 `compare_and_exchange` 类似，但允许虚假失败，在某些平台上性能更好。
    /// 主要用于循环中的 CAS 操作。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    ///
    /// // 在循环中使用弱版本
    /// let mut current = atomic.get();
    /// loop {
    ///     let new = current + 5;
    ///     let prev = atomic.compare_and_exchange_weak(current, new);
    ///     if prev == current {
    ///         break;
    ///     }
    ///     current = prev;
    /// }
    /// assert_eq!(atomic.get(), 15);
    /// ```
    pub fn compare_and_exchange_weak(&self, current: i32, new: i32) -> i32;

    /// 获取底层标准库类型的引用
    ///
    /// 用于需要精细控制内存序的高级场景。大多数情况下不需要使用此方法，
    /// 默认 API 已经提供了合理的内存序。
    ///
    /// # 使用场景
    ///
    /// - 极致性能优化（需要使用 `Relaxed` ordering）
    /// - 复杂的无锁算法（需要精确控制内存序）
    /// - 与直接使用标准库的代码互操作
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    /// use std::sync::atomic::Ordering;
    ///
    /// let atomic = AtomicI32::new(0);
    ///
    /// // 高性能场景：使用 Relaxed ordering
    /// for _ in 0..1_000_000 {
    ///     atomic.inner().fetch_add(1, Ordering::Relaxed);
    /// }
    ///
    /// // 最后用 Acquire 读取结果
    /// let result = atomic.inner().load(Ordering::Acquire);
    /// assert_eq!(result, 1_000_000);
    /// ```
    pub fn inner(&self) -> &std::sync::atomic::AtomicI32;
}
```

### 4.2 整数类型的高级操作

整数类型（`AtomicI32`、`AtomicI64`、`AtomicU32`、`AtomicU64`、`AtomicIsize`、`AtomicUsize`）额外提供：

```rust
impl AtomicI32 {
    // ==================== 自增/自减操作 ====================

    /// 原子自增，返回旧值（使用 Relaxed ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.get_and_increment();
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 11);
    /// ```
    pub fn get_and_increment(&self) -> i32;

    /// 原子自增，返回新值（使用 Relaxed ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let new = atomic.increment_and_get();
    /// assert_eq!(new, 11);
    /// ```
    pub fn increment_and_get(&self) -> i32;

    /// 原子自减，返回旧值（使用 Relaxed ordering）
    pub fn get_and_decrement(&self) -> i32;

    /// 原子自减，返回新值（使用 Relaxed ordering）
    pub fn decrement_and_get(&self) -> i32;

    // ==================== 加法/减法操作 ====================

    /// 原子加法，返回旧值（使用 Relaxed ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.get_and_add(5);
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 15);
    /// ```
    pub fn get_and_add(&self, delta: i32) -> i32;

    /// 原子加法，返回新值（使用 Relaxed ordering）
    pub fn add_and_get(&self, delta: i32) -> i32;

    /// 原子减法，返回旧值（使用 Relaxed ordering）
    pub fn get_and_sub(&self, delta: i32) -> i32;

    /// 原子减法，返回新值（使用 Relaxed ordering）
    pub fn sub_and_get(&self, delta: i32) -> i32;

    // ==================== 位运算操作 ====================

    /// 原子按位与，返回旧值
    pub fn get_and_bitand(&self, value: i32) -> i32;

    /// 原子按位或，返回旧值
    pub fn get_and_bitor(&self, value: i32) -> i32;

    /// 原子按位异或，返回旧值
    pub fn get_and_bitxor(&self, value: i32) -> i32;

    // ==================== 函数式更新操作 ====================

    /// 使用给定函数原子更新值，返回旧值
    ///
    /// 内部使用 CAS 循环，直到更新成功。
    ///
    /// # 参数
    ///
    /// * `f` - 更新函数，接收当前值，返回新值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.get_and_update(|x| x * 2);
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 20);
    /// ```
    pub fn get_and_update<F>(&self, f: F) -> i32
    where
        F: Fn(i32) -> i32;

    /// 使用给定函数原子更新值，返回新值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let new = atomic.update_and_get(|x| x * 2);
    /// assert_eq!(new, 20);
    /// ```
    pub fn update_and_get<F>(&self, f: F) -> i32
    where
        F: Fn(i32) -> i32;

    /// 使用给定的二元函数原子累积值
    ///
    /// # 参数
    ///
    /// * `x` - 累积参数
    /// * `f` - 累积函数，接收当前值和参数，返回新值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.get_and_accumulate(5, |a, b| a + b);
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 15);
    /// ```
    pub fn get_and_accumulate<F>(&self, x: i32, f: F) -> i32
    where
        F: Fn(i32, i32) -> i32;

    /// 使用给定的二元函数原子累积值，返回新值
    pub fn accumulate_and_get<F>(&self, x: i32, f: F) -> i32
    where
        F: Fn(i32, i32) -> i32;

    // ==================== 最大值/最小值操作 ====================

    /// 原子取最大值，返回旧值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// atomic.get_and_max(20);
    /// assert_eq!(atomic.get(), 20);
    ///
    /// atomic.get_and_max(15);
    /// assert_eq!(atomic.get(), 20); // 保持较大值
    /// ```
    pub fn get_and_max(&self, value: i32) -> i32;

    /// 原子取最大值，返回新值
    pub fn max_and_get(&self, value: i32) -> i32;

    /// 原子取最小值，返回旧值
    pub fn get_and_min(&self, value: i32) -> i32;

    /// 原子取最小值，返回新值
    pub fn min_and_get(&self, value: i32) -> i32;
}
```

### 4.3 布尔类型的特殊操作

```rust
impl AtomicBool {
    /// 创建新的原子布尔值
    pub const fn new(value: bool) -> Self;

    /// 获取当前值
    pub fn get(&self) -> bool;

    /// 设置新值
    pub fn set(&self, value: bool);

    /// 交换值，返回旧值
    pub fn swap(&self, value: bool) -> bool;

    /// 比较并交换
    pub fn compare_and_set(&self, current: bool, new: bool) -> Result<(), bool>;

    /// 弱版本的 CAS
    pub fn compare_and_set_weak(&self, current: bool, new: bool) -> Result<(), bool>;

    /// 比较并交换，返回交换前的实际值
    ///
    /// 如果当前值等于 `current`，则设置为 `new`，返回 `current`（成功）；
    /// 否则返回实际的当前值（失败）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    ///
    /// // 成功的交换
    /// let prev = flag.compare_and_exchange(false, true);
    /// assert_eq!(prev, false); // 返回旧值，说明成功
    /// assert_eq!(flag.get(), true);
    ///
    /// // 失败的交换
    /// let prev = flag.compare_and_exchange(false, true);
    /// assert_eq!(prev, true); // 返回实际值（不是期望的 false），说明失败
    /// assert_eq!(flag.get(), true); // 值未改变
    /// ```
    pub fn compare_and_exchange(&self, current: bool, new: bool) -> bool;

    /// 弱版本的 compare_and_exchange
    pub fn compare_and_exchange_weak(&self, current: bool, new: bool) -> bool;

    // ==================== 布尔特殊操作 ====================

    /// 原子设置为 true，返回旧值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// let old = flag.get_and_set();
    /// assert_eq!(old, false);
    /// assert_eq!(flag.get(), true);
    /// ```
    pub fn get_and_set(&self) -> bool;

    /// 原子设置为 true，返回新值
    pub fn set_and_get(&self) -> bool;

    /// 原子设置为 false，返回旧值
    pub fn get_and_clear(&self) -> bool;

    /// 原子设置为 false，返回新值
    pub fn clear_and_get(&self) -> bool;

    /// 原子取反，返回旧值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert_eq!(flag.get_and_negate(), false);
    /// assert_eq!(flag.get(), true);
    /// assert_eq!(flag.get_and_negate(), true);
    /// assert_eq!(flag.get(), false);
    /// ```
    pub fn get_and_negate(&self) -> bool;

    /// 原子取反，返回新值
    pub fn negate_and_get(&self) -> bool;

    /// 原子逻辑与，返回旧值
    pub fn get_and_logical_and(&self, value: bool) -> bool;

    /// 原子逻辑或，返回旧值
    pub fn get_and_logical_or(&self, value: bool) -> bool;

    /// 原子逻辑异或，返回旧值
    pub fn get_and_logical_xor(&self, value: bool) -> bool;

    /// 使用 CAS 实现的条件设置
    ///
    /// 当当前值为 `false` 时设置为 `true`，返回是否成功。
    /// 常用于实现一次性标志或锁。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    ///
    /// // 第一次调用成功
    /// assert!(flag.compare_and_set_if_false(true).is_ok());
    /// assert_eq!(flag.get(), true);
    ///
    /// // 第二次调用失败（已经是 true）
    /// assert!(flag.compare_and_set_if_false(true).is_err());
    /// ```
    pub fn compare_and_set_if_false(&self, new: bool) -> Result<(), bool>;

    /// 当当前值为 `true` 时设置为 `false`，返回是否成功
    pub fn compare_and_set_if_true(&self, new: bool) -> Result<(), bool>;
}
```

### 4.4 引用类型的操作

```rust
/// 原子引用封装
///
/// 使用 `Arc<T>` 实现线程安全的引用共享。
///
/// # 泛型参数
///
/// * `T` - 引用的数据类型
pub struct AtomicRef<T> {
    inner: std::sync::atomic::AtomicPtr<Arc<T>>,
}

impl<T> AtomicRef<T> {
    /// 创建新的原子引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let data = Arc::new(42);
    /// let atomic = AtomicRef::new(data);
    /// ```
    pub fn new(value: Arc<T>) -> Self;

    /// 获取当前引用（使用 Acquire ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let atomic = AtomicRef::new(Arc::new(42));
    /// let value = atomic.get();
    /// assert_eq!(*value, 42);
    /// ```
    pub fn get(&self) -> Arc<T>;

    /// 设置新引用（使用 Release ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let atomic = AtomicRef::new(Arc::new(42));
    /// atomic.set(Arc::new(100));
    /// assert_eq!(*atomic.get(), 100);
    /// ```
    pub fn set(&self, value: Arc<T>);

    /// 交换引用，返回旧引用（使用 AcqRel ordering）
    pub fn swap(&self, value: Arc<T>) -> Arc<T>;

    /// 比较并交换引用
    ///
    /// 如果当前引用与 `current` 指向同一对象，则替换为 `new`。
    ///
    /// # 注意
    ///
    /// 比较使用指针相等性（`Arc::ptr_eq`），而非值相等性。
    pub fn compare_and_set(&self, current: &Arc<T>, new: Arc<T>) -> Result<(), Arc<T>>;

    /// 弱版本的 CAS
    pub fn compare_and_set_weak(&self, current: &Arc<T>, new: Arc<T>) -> Result<(), Arc<T>>;

    /// 比较并交换引用，返回交换前的实际引用
    ///
    /// 如果当前引用与 `current` 指向同一对象，则替换为 `new`，返回旧引用（成功）；
    /// 否则返回实际的当前引用（失败）。
    ///
    /// # 注意
    ///
    /// 比较使用指针相等性（`Arc::ptr_eq`），而非值相等性。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let atomic = AtomicRef::new(Arc::new(10));
    /// let current = atomic.get();
    ///
    /// // 成功的交换
    /// let prev = atomic.compare_and_exchange(&current, Arc::new(20));
    /// assert!(Arc::ptr_eq(&prev, &current)); // 返回旧引用，说明成功
    /// assert_eq!(*atomic.get(), 20);
    ///
    /// // 失败的交换
    /// let prev = atomic.compare_and_exchange(&current, Arc::new(30));
    /// assert!(!Arc::ptr_eq(&prev, &current)); // 返回实际引用（不是期望的），说明失败
    /// assert_eq!(*atomic.get(), 20); // 值未改变
    /// ```
    pub fn compare_and_exchange(&self, current: &Arc<T>, new: Arc<T>) -> Arc<T>;

    /// 弱版本的 compare_and_exchange
    pub fn compare_and_exchange_weak(&self, current: &Arc<T>, new: Arc<T>) -> Arc<T>;

    /// 使用函数更新引用，返回旧引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let atomic = AtomicRef::new(Arc::new(10));
    /// let old = atomic.get_and_update(|x| Arc::new(*x * 2));
    /// assert_eq!(*old, 10);
    /// assert_eq!(*atomic.get(), 20);
    /// ```
    pub fn get_and_update<F>(&self, f: F) -> Arc<T>
    where
        F: Fn(&Arc<T>) -> Arc<T>;

    /// 使用函数更新引用，返回新引用
    pub fn update_and_get<F>(&self, f: F) -> Arc<T>
    where
        F: Fn(&Arc<T>) -> Arc<T>;
}

impl<T> Clone for AtomicRef<T> {
    /// 克隆原子引用
    ///
    /// 注意：这会创建一个新的 `AtomicRef`，它与原始引用指向同一底层数据，
    /// 但后续的原子操作是独立的。
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}
```

### 4.5 浮点数类型的操作

浮点数原子类型通过位转换实现，基于 `AtomicU32` 和 `AtomicU64`。

#### 4.5.1 AtomicF32 设计

```rust
/// 原子 32 位浮点数
///
/// 通过 `AtomicU32` 和位转换实现。由于硬件限制，浮点数没有原生的原子算术操作，
/// 因此算术操作需要通过 CAS 循环实现。
///
/// # 特性
///
/// - 支持基本的原子操作（load、store、swap、CAS）
/// - 算术操作通过 CAS 循环实现（性能略低于整数）
/// - NaN 值的处理遵循 IEEE 754 标准（NaN != NaN）
///
/// # 限制
///
/// - 不提供 `max`/`min` 操作（浮点数比较语义复杂）
/// - 算术操作在高竞争场景下性能可能不理想
/// - 需要注意浮点数精度和舍入问题
///
/// # 示例
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicF32;
/// use std::sync::Arc;
/// use std::thread;
///
/// let value = Arc::new(AtomicF32::new(0.0));
/// let mut handles = vec![];
///
/// for _ in 0..10 {
///     let value = value.clone();
///     let handle = thread::spawn(move || {
///         for _ in 0..1000 {
///             value.add(0.1);
///         }
///     });
///     handles.push(handle);
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
///
/// // 注意：由于浮点数精度问题，结果可能不是精确的 1000.0
/// println!("Result: {}", value.get());
/// ```
#[repr(transparent)]
pub struct AtomicF32 {
    inner: std::sync::atomic::AtomicU32,
}

impl AtomicF32 {
    /// 创建新的原子浮点数
    pub const fn new(value: f32) -> Self;

    /// 获取当前值（使用 Acquire ordering）
    pub fn get(&self) -> f32;

    /// 设置新值（使用 Release ordering）
    pub fn set(&self, value: f32);

    /// 交换值，返回旧值（使用 AcqRel ordering）
    pub fn swap(&self, value: f32) -> f32;

    /// 比较并交换
    ///
    /// # 注意
    ///
    /// 由于 NaN != NaN，如果当前值或期望值是 NaN，CAS 可能会有意外行为。
    /// 建议避免在原子浮点数中使用 NaN 值。
    pub fn compare_and_set(&self, current: f32, new: f32) -> Result<(), f32>;

    /// 弱版本的 CAS
    pub fn compare_and_set_weak(&self, current: f32, new: f32) -> Result<(), f32>;

    /// 比较并交换，返回交换前的实际值
    pub fn compare_and_exchange(&self, current: f32, new: f32) -> f32;

    /// 弱版本的 compare_and_exchange
    pub fn compare_and_exchange_weak(&self, current: f32, new: f32) -> f32;

    // ==================== 算术操作（通过 CAS 循环实现）====================

    /// 原子加法，返回新值
    ///
    /// 内部使用 CAS 循环实现，在高竞争场景下性能可能不理想。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(10.0);
    /// let new = atomic.add(5.5);
    /// assert_eq!(new, 15.5);
    /// ```
    pub fn add(&self, delta: f32) -> f32;

    /// 原子减法，返回新值
    pub fn sub(&self, delta: f32) -> f32;

    /// 原子乘法，返回新值
    pub fn mul(&self, factor: f32) -> f32;

    /// 原子除法，返回新值
    pub fn div(&self, divisor: f32) -> f32;

    /// 使用给定函数原子更新值，返回旧值
    pub fn get_and_update<F>(&self, f: F) -> f32
    where
        F: Fn(f32) -> f32;

    /// 使用给定函数原子更新值，返回新值
    pub fn update_and_get<F>(&self, f: F) -> f32
    where
        F: Fn(f32) -> f32;

    /// 获取底层标准库类型的引用
    pub fn inner(&self) -> &std::sync::atomic::AtomicU32;
}
```

#### 4.5.2 AtomicF64 设计

```rust
/// 原子 64 位浮点数
///
/// 通过 `AtomicU64` 和位转换实现。设计与 `AtomicF32` 类似。
///
/// # 示例
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicF64;
///
/// let atomic = AtomicF64::new(3.14159);
/// atomic.add(1.0);
/// assert_eq!(atomic.get(), 4.14159);
/// ```
#[repr(transparent)]
pub struct AtomicF64 {
    inner: std::sync::atomic::AtomicU64,
}

impl AtomicF64 {
    /// 创建新的原子浮点数
    pub const fn new(value: f64) -> Self;

    /// 获取当前值
    pub fn get(&self) -> f64;

    /// 设置新值
    pub fn set(&self, value: f64);

    /// 交换值，返回旧值
    pub fn swap(&self, value: f64) -> f64;

    /// 比较并交换
    pub fn compare_and_set(&self, current: f64, new: f64) -> Result<(), f64>;

    /// 弱版本的 CAS
    pub fn compare_and_set_weak(&self, current: f64, new: f64) -> Result<(), f64>;

    /// 比较并交换，返回交换前的实际值
    pub fn compare_and_exchange(&self, current: f64, new: f64) -> f64;

    /// 弱版本的 compare_and_exchange
    pub fn compare_and_exchange_weak(&self, current: f64, new: f64) -> f64;

    // ==================== 算术操作（通过 CAS 循环实现）====================

    /// 原子加法，返回新值
    pub fn add(&self, delta: f64) -> f64;

    /// 原子减法，返回新值
    pub fn sub(&self, delta: f64) -> f64;

    /// 原子乘法，返回新值
    pub fn mul(&self, factor: f64) -> f64;

    /// 原子除法，返回新值
    pub fn div(&self, divisor: f64) -> f64;

    /// 使用给定函数原子更新值，返回旧值
    pub fn get_and_update<F>(&self, f: F) -> f64
    where
        F: Fn(f64) -> f64;

    /// 使用给定函数原子更新值，返回新值
    pub fn update_and_get<F>(&self, f: F) -> f64
    where
        F: Fn(f64) -> f64;

    /// 获取底层标准库类型的引用
    pub fn inner(&self) -> &std::sync::atomic::AtomicU64;
}
```

#### 4.5.3 浮点数原子类型的注意事项

**1. NaN 值处理**

```rust
use prism3_rust_concurrent::atomic::AtomicF32;

let atomic = AtomicF32::new(f32::NAN);

// ⚠️ 警告：NaN != NaN，CAS 操作可能会失败
let result = atomic.compare_and_set(f32::NAN, 1.0);
// 结果不确定，因为 NaN 的比较总是返回 false

// ✅ 建议：避免在原子浮点数中使用 NaN
// 如果需要表示"无效值"，使用 Option<AtomicF32> 或特殊的哨兵值
```

**2. 浮点数精度**

```rust
let atomic = AtomicF32::new(0.0);

// 累加 0.1 十次
for _ in 0..10 {
    atomic.add(0.1);
}

// ⚠️ 由于浮点数精度问题，结果可能不是精确的 1.0
let result = atomic.get();
assert!((result - 1.0).abs() < 1e-6); // 使用容差比较
```

**3. 性能考虑**

```rust
// ❌ 不推荐：高竞争场景下的频繁算术操作
let counter = Arc::new(AtomicF32::new(0.0));
for _ in 0..1000 {
    counter.add(1.0); // 每次都需要 CAS 循环
}

// ✅ 推荐：使用整数原子类型，最后转换
let counter = Arc::new(AtomicI32::new(0));
for _ in 0..1000 {
    counter.increment_and_get(); // 原生原子操作，更快
}
let result = counter.get() as f32;
```

**4. 不提供的操作**

以下操作由于浮点数语义复杂，不提供：

- `max()` / `min()`：需要处理 NaN、+0.0/-0.0 等特殊情况
- `abs()`：符号位操作可能与用户期望不一致
- `increment()` / `decrement()`：对浮点数意义不明确

如果需要这些操作，请使用 `update_and_get` 自定义：

```rust
let atomic = AtomicF32::new(-5.0);

// 自定义 abs 操作
let result = atomic.update_and_get(|x| x.abs());
assert_eq!(result, 5.0);

// 自定义 max 操作（需要处理 NaN）
let result = atomic.update_and_get(|x| x.max(10.0));
```

## 5. Trait 抽象设计

### 5.1 Atomic Trait

提供统一的原子操作接口：

```rust
/// 原子操作的通用 trait
///
/// 定义了所有原子类型的基本操作。
pub trait Atomic {
    /// 值类型
    type Value;

    /// 获取当前值
    fn get(&self) -> Self::Value;

    /// 设置新值
    fn set(&self, value: Self::Value);

    /// 交换值，返回旧值
    fn swap(&self, value: Self::Value) -> Self::Value;

    /// 比较并交换
    fn compare_and_set(&self, current: Self::Value, new: Self::Value)
        -> Result<(), Self::Value>;

    /// 比较并交换，返回交换前的实际值
    ///
    /// 与 `compare_and_set` 的区别在于返回值：
    /// - `compare_and_set` 返回 `Result`，通过 `Ok`/`Err` 表示成功或失败
    /// - `compare_and_exchange` 直接返回交换前的实际值，调用者通过比较判断是否成功
    ///
    /// 在 CAS 循环中，`compare_and_exchange` 通常更简洁，因为失败时可以直接使用返回值。
    fn compare_and_exchange(&self, current: Self::Value, new: Self::Value) -> Self::Value;
}

/// 可更新的原子类型 trait
///
/// 提供函数式更新操作。
pub trait UpdatableAtomic: Atomic {
    /// 使用函数更新值，返回旧值
    fn get_and_update<F>(&self, f: F) -> Self::Value
    where
        F: Fn(Self::Value) -> Self::Value;

    /// 使用函数更新值，返回新值
    fn update_and_get<F>(&self, f: F) -> Self::Value
    where
        F: Fn(Self::Value) -> Self::Value;
}

/// 原子整数 trait
///
/// 提供整数特有的操作。
pub trait AtomicInteger: UpdatableAtomic {
    /// 自增，返回旧值
    fn get_and_increment(&self) -> Self::Value;

    /// 自增，返回新值
    fn increment_and_get(&self) -> Self::Value;

    /// 自减，返回旧值
    fn get_and_decrement(&self) -> Self::Value;

    /// 自减，返回新值
    fn decrement_and_get(&self) -> Self::Value;

    /// 加法，返回旧值
    fn get_and_add(&self, delta: Self::Value) -> Self::Value;

    /// 加法，返回新值
    fn add_and_get(&self, delta: Self::Value) -> Self::Value;
}
```

### 5.2 Trait 实现

```rust
// AtomicI32 实现 Atomic trait
impl Atomic for AtomicI32 {
    type Value = i32;

    fn get(&self) -> i32 {
        self.inner.load(Ordering::Acquire)
    }

    fn set(&self, value: i32) {
        self.inner.store(value, Ordering::Release);
    }

    fn swap(&self, value: i32) -> i32 {
        self.inner.swap(value, Ordering::AcqRel)
    }

    fn compare_and_set(&self, current: i32, new: i32) -> Result<(), i32> {
        self.inner
            .compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
    }

    fn compare_and_exchange(&self, current: i32, new: i32) -> i32 {
        // 使用标准库的 compare_exchange，成功返回旧值，失败返回当前值
        // 无论成功或失败，都返回交换前的实际值
        match self.inner.compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire) {
            Ok(prev) => prev,
            Err(actual) => actual,
        }
    }
}

// AtomicI32 实现 AtomicInteger trait
impl AtomicInteger for AtomicI32 {
    fn get_and_increment(&self) -> i32 {
        self.inner.fetch_add(1, Ordering::Relaxed)
    }

    fn increment_and_get(&self) -> i32 {
        self.inner.fetch_add(1, Ordering::Relaxed) + 1
    }

    // ... 其他方法
}
```

## 6. 使用示例

### 6.1 基础计数器

```rust
use prism3_rust_concurrent::atomic::AtomicI32;
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
                counter.increment_and_get();
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证结果
    assert_eq!(counter.get(), 10000);
    println!("最终计数：{}", counter.get());
}
```

### 6.2 CAS 循环

```rust
use prism3_rust_concurrent::atomic::AtomicI32;

fn increment_even_only(atomic: &AtomicI32) -> Result<i32, &'static str> {
    let mut current = atomic.get();
    loop {
        // 只对偶数值进行递增
        if current % 2 != 0 {
            return Err("Value is odd");
        }

        let new = current + 2;
        match atomic.compare_and_set(current, new) {
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

    assert_eq!(atomic.get(), 12);
}
```

### 6.3 函数式更新

```rust
use prism3_rust_concurrent::atomic::AtomicI32;

fn main() {
    let atomic = AtomicI32::new(10);

    // 使用函数更新
    let new_value = atomic.update_and_get(|x| {
        if x < 100 {
            x * 2
        } else {
            x
        }
    });

    assert_eq!(new_value, 20);
    println!("更新后的值：{}", new_value);

    // 累积操作
    let result = atomic.accumulate_and_get(5, |a, b| a + b);
    assert_eq!(result, 25);
    println!("累积后的值：{}", result);
}
```

### 6.4 原子引用

```rust
use prism3_rust_concurrent::atomic::AtomicRef;
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
    println!("新配置：{:?}", atomic_config.get());

    // 使用函数更新
    atomic_config.update_and_get(|current| {
        Arc::new(Config {
            timeout: current.timeout * 2,
            max_retries: current.max_retries + 1,
        })
    });

    println!("更新后的配置：{:?}", atomic_config.get());
}
```

### 6.5 布尔标志

```rust
use prism3_rust_concurrent::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
        if self.running.compare_and_set_if_false(true).is_ok() {
            println!("服务启动成功");
        } else {
            println!("服务已经在运行");
        }
    }

    fn stop(&self) {
        // 只有当前运行时才停止
        if self.running.compare_and_set_if_true(false).is_ok() {
            println!("服务停止成功");
        } else {
            println!("服务已经停止");
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

    service.start(); // 重复启动会失败

    service.stop();
    assert!(!service.is_running());

    service.stop(); // 重复停止会失败
}
```

### 6.6 浮点数原子操作

```rust
use prism3_rust_concurrent::atomic::AtomicF32;
use std::sync::Arc;
use std::thread;

/// 示例：多线程累加浮点数
fn float_accumulator_example() {
    let sum = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

    // 启动 10 个线程，每个线程累加 100 次
    for i in 0..10 {
        let sum = sum.clone();
        let handle = thread::spawn(move || {
            for j in 0..100 {
                // 使用 add 方法（内部 CAS 循环）
                sum.add(0.01);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 注意：由于浮点数精度问题，结果可能不是精确的 10.0
    let result = sum.get();
    println!("累加结果: {:.6}", result);
    println!("误差: {:.6}", (result - 10.0).abs());
}

/// 示例：原子更新浮点数（自定义操作）
fn float_custom_update_example() {
    let temperature = AtomicF32::new(20.0);

    // 使用 update_and_get 实现自定义逻辑
    let new_temp = temperature.update_and_get(|current| {
        // 温度限制在 -50 到 50 之间
        (current + 5.0).clamp(-50.0, 50.0)
    });

    println!("更新后的温度: {}", new_temp);

    // 乘法操作
    temperature.mul(1.8); // 摄氏度转华氏度的一部分
    temperature.add(32.0); // 完成转换

    println!("华氏温度: {}", temperature.get());
}

/// 示例：性能对比 - 整数 vs 浮点数
fn performance_comparison() {
    use prism3_rust_concurrent::atomic::AtomicI32;
    use std::time::Instant;

    let iterations = 100_000;

    // 整数原子操作（使用原生 fetch_add）
    let int_counter = Arc::new(AtomicI32::new(0));
    let start = Instant::now();
    for _ in 0..iterations {
        int_counter.increment_and_get();
    }
    let int_duration = start.elapsed();

    // 浮点数原子操作（使用 CAS 循环）
    let float_counter = Arc::new(AtomicF32::new(0.0));
    let start = Instant::now();
    for _ in 0..iterations {
        float_counter.add(1.0);
    }
    let float_duration = start.elapsed();

    println!("整数操作耗时: {:?}", int_duration);
    println!("浮点数操作耗时: {:?}", float_duration);
    println!(
        "性能比: {:.2}x",
        float_duration.as_nanos() as f64 / int_duration.as_nanos() as f64
    );
}

fn main() {
    println!("=== 浮点数累加示例 ===");
    float_accumulator_example();

    println!("\n=== 自定义更新示例 ===");
    float_custom_update_example();

    println!("\n=== 性能对比 ===");
    performance_comparison();
}
```

**浮点数原子操作的最佳实践**：

1. **避免高竞争场景**：浮点数算术操作使用 CAS 循环，高竞争下性能下降明显
2. **注意精度问题**：累加小数时可能产生精度误差，使用容差比较
3. **避免 NaN**：不要在原子浮点数中使用 NaN 值
4. **优先使用整数**：如果可能，使用整数原子类型，最后转换为浮点数

### 6.7 使用 Trait 的泛型代码

```rust
use prism3_rust_concurrent::atomic::{Atomic, AtomicInteger, AtomicI32, AtomicI64};

/// 通用的原子计数器
fn increment_atomic<T>(atomic: &T) -> T::Value
where
    T: AtomicInteger<Value = i32>,
{
    atomic.increment_and_get()
}

fn main() {
    let counter32 = AtomicI32::new(0);
    let result = increment_atomic(&counter32);
    assert_eq!(result, 1);

    let counter64 = AtomicI64::new(0);
    // increment_atomic(&counter64); // 编译错误：类型不匹配
}
```

### 6.7 高性能场景：直接操作底层类型

```rust
use prism3_rust_concurrent::atomic::AtomicI32;
use std::sync::atomic::Ordering;

fn high_performance_counter() {
    let counter = AtomicI32::new(0);

    // 在只需要保证原子性、不需要同步其他状态的场景下，
    // 可以直接访问底层类型使用 Relaxed ordering 获得最佳性能
    for _ in 0..1_000_000 {
        counter.inner().fetch_add(1, Ordering::Relaxed);
    }

    // 最后使用 Acquire 读取最终值
    let final_count = counter.inner().load(Ordering::Acquire);
    println!("最终计数：{}", final_count);
}

fn mixed_usage() {
    let counter = AtomicI32::new(0);

    // 99% 的代码使用简单 API
    counter.increment_and_get();
    counter.add_and_get(5);

    // 1% 的关键路径使用精细控制
    unsafe {
        // 某些极端场景可能需要 unsafe 配合底层类型
    }

    // 继续使用简单 API
    let value = counter.get();
    println!("当前值：{}", value);
}
```

## 7. 实现细节

### 7.1 内存布局

所有封装类型都应该具有与底层标准库类型相同的内存布局：

```rust
#[repr(transparent)]
pub struct AtomicI32 {
    inner: std::sync::atomic::AtomicI32,
}
```

使用 `#[repr(transparent)]` 确保零成本抽象。

### 7.2 方法内联

所有方法都应该内联，避免函数调用开销：

```rust
impl AtomicI32 {
    #[inline]
    pub fn get(&self) -> i32 {
        self.inner.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set(&self, value: i32) {
        self.inner.store(value, Ordering::Release);
    }

    #[inline]
    pub fn inner(&self) -> &std::sync::atomic::AtomicI32 {
        &self.inner
    }

    // ... 其他方法
}
```

### 7.3 CAS 循环实现

函数式更新方法使用标准 CAS 循环模式，可以使用 `compare_and_set` 或 `compare_and_exchange`：

```rust
impl AtomicI32 {
    // 使用 compare_and_set（Result 风格）
    pub fn update_and_get<F>(&self, f: F) -> i32
    where
        F: Fn(i32) -> i32,
    {
        let mut current = self.get();
        loop {
            let new = f(current);
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    // 使用 compare_and_exchange（直接返回值风格，更简洁）
    pub fn get_and_update<F>(&self, f: F) -> i32
    where
        F: Fn(i32) -> i32,
    {
        let mut current = self.get();
        loop {
            let new = f(current);
            let prev = self.compare_and_exchange_weak(current, new);
            if prev == current {
                return current; // 成功，返回旧值
            }
            current = prev; // 失败，prev 就是最新值，直接使用
        }
    }
}
```

**两种风格的对比**：

```rust
// 风格 1：使用 compare_and_set（Result 风格）
let mut current = atomic.get();
loop {
    let new = current + 1;
    match atomic.compare_and_set(current, new) {
        Ok(_) => break,
                Err(actual) => current = actual,
            }
        }

// 风格 2：使用 compare_and_exchange（更简洁）
let mut current = atomic.get();
loop {
    let new = current + 1;
    let prev = atomic.compare_and_exchange(current, new);
    if prev == current {
        break;
    }
    current = prev;
}
```

两种风格功能等价，`compare_and_exchange` 在 CAS 循环中通常更简洁直观。

### 7.4 AtomicRef 实现细节

`AtomicRef` 需要正确管理 `Arc` 的引用计数：

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::ptr;

pub struct AtomicRef<T> {
    inner: AtomicPtr<T>,
}

impl<T> AtomicRef<T> {
    pub fn new(value: Arc<T>) -> Self {
        let ptr = Arc::into_raw(value) as *mut T;
        Self {
            inner: AtomicPtr::new(ptr),
        }
    }

    pub fn get(&self) -> Arc<T> {
        let ptr = self.inner.load(Ordering::Acquire);
        unsafe {
            // 增加引用计数但不释放原指针
            let arc = Arc::from_raw(ptr);
            let cloned = arc.clone();
            Arc::into_raw(arc); // 防止释放
            cloned
        }
    }

    pub fn set(&self, value: Arc<T>) {
        let new_ptr = Arc::into_raw(value) as *mut T;
        let old_ptr = self.inner.swap(new_ptr, Ordering::AcqRel);
        unsafe {
            if !old_ptr.is_null() {
                // 释放旧值
                Arc::from_raw(old_ptr);
            }
        }
    }

    // ... 其他方法
}

impl<T> Drop for AtomicRef<T> {
    fn drop(&mut self) {
        let ptr = self.inner.load(Ordering::Acquire);
        unsafe {
            if !ptr.is_null() {
                Arc::from_raw(ptr);
            }
        }
    }
}

unsafe impl<T: Send + Sync> Send for AtomicRef<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicRef<T> {}
```

### 7.5 浮点数原子类型实现细节

浮点数原子类型通过位转换实现，核心思想是利用 `f32`/`f64` 与 `u32`/`u64` 之间的位级等价性。

#### 7.5.1 基本实现

```rust
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

#[repr(transparent)]
pub struct AtomicF32 {
    inner: AtomicU32,
}

impl AtomicF32 {
    #[inline]
    pub const fn new(value: f32) -> Self {
        Self {
            inner: AtomicU32::new(value.to_bits()),
        }
    }

    #[inline]
    pub fn get(&self) -> f32 {
        f32::from_bits(self.inner.load(Ordering::Acquire))
    }

    #[inline]
    pub fn set(&self, value: f32) {
        self.inner.store(value.to_bits(), Ordering::Release);
    }

    #[inline]
    pub fn swap(&self, value: f32) -> f32 {
        f32::from_bits(self.inner.swap(value.to_bits(), Ordering::AcqRel))
    }

    #[inline]
    pub fn compare_and_set(&self, current: f32, new: f32) -> Result<(), f32> {
        self.inner
            .compare_exchange(
                current.to_bits(),
                new.to_bits(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|bits| f32::from_bits(bits))
    }

    #[inline]
    pub fn compare_and_exchange(&self, current: f32, new: f32) -> f32 {
        match self.inner.compare_exchange(
            current.to_bits(),
            new.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(prev_bits) => f32::from_bits(prev_bits),
            Err(actual_bits) => f32::from_bits(actual_bits),
        }
    }

    #[inline]
    pub fn inner(&self) -> &AtomicU32 {
        &self.inner
    }
}

// AtomicF64 实现类似，使用 AtomicU64
#[repr(transparent)]
pub struct AtomicF64 {
    inner: AtomicU64,
}

impl AtomicF64 {
    #[inline]
    pub const fn new(value: f64) -> Self {
        Self {
            inner: AtomicU64::new(value.to_bits()),
        }
    }

    #[inline]
    pub fn get(&self) -> f64 {
        f64::from_bits(self.inner.load(Ordering::Acquire))
    }

    // ... 其他方法类似 AtomicF32
}
```

#### 7.5.2 算术操作实现（CAS 循环）

由于硬件不支持浮点数的原子算术操作，需要通过 CAS 循环实现：

```rust
impl AtomicF32 {
    pub fn add(&self, delta: f32) -> f32 {
        let mut current = self.get();
        loop {
            let new = current + delta;
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn sub(&self, delta: f32) -> f32 {
        let mut current = self.get();
        loop {
            let new = current - delta;
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn mul(&self, factor: f32) -> f32 {
        let mut current = self.get();
        loop {
            let new = current * factor;
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn div(&self, divisor: f32) -> f32 {
        let mut current = self.get();
        loop {
            let new = current / divisor;
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn update_and_get<F>(&self, f: F) -> f32
    where
        F: Fn(f32) -> f32,
    {
        let mut current = self.get();
        loop {
            let new = f(current);
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn get_and_update<F>(&self, f: F) -> f32
    where
        F: Fn(f32) -> f32,
    {
        let mut current = self.get();
        loop {
            let new = f(current);
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }
}
```

#### 7.5.3 NaN 处理的特殊考虑

NaN 值的比较总是返回 `false`，这会导致 CAS 操作的特殊行为：

```rust
// 问题示例
let atomic = AtomicF32::new(f32::NAN);
let current = atomic.get(); // 得到 NaN

// ⚠️ 这个 CAS 会失败，因为 NaN != NaN（位级比较）
// 即使当前值确实是 NaN，但 NaN 的位模式可能不同
atomic.compare_and_set(current, 1.0); // 可能失败

// 解决方案：使用位级比较
let current_bits = atomic.inner().load(Ordering::Acquire);
atomic.inner().compare_exchange(
    current_bits,
    1.0_f32.to_bits(),
    Ordering::AcqRel,
    Ordering::Acquire,
);
```

**设计建议**：
1. 在文档中明确警告 NaN 的特殊行为
2. 建议用户避免在原子浮点数中使用 NaN
3. 如果需要表示"无效值"，使用 `Option<f32>` 或特殊哨兵值（如 `-1.0`）

#### 7.5.4 性能特性

| 操作类型 | 性能 | 说明 |
|---------|------|------|
| `get()` / `set()` | 与整数相同 | 只是位转换，无额外开销 |
| `swap()` | 与整数相同 | 原子交换 + 位转换 |
| `compare_and_set()` | 与整数相同 | 单次 CAS + 位转换 |
| `add()` / `sub()` 等 | 较慢 | CAS 循环，高竞争下性能下降 |

**性能对比**（相对于 `AtomicI32::fetch_add`）：

```rust
// 基准测试结果（参考）
AtomicI32::fetch_add()      // 1.0x  (基线)
AtomicF32::add()            // 3-5x  (低竞争)
AtomicF32::add()            // 10-20x (高竞争)
```

#### 7.5.5 Trait 实现

浮点数类型实现 `Atomic` trait，但不实现 `AtomicInteger` trait：

```rust
impl Atomic for AtomicF32 {
    type Value = f32;

    fn get(&self) -> f32 {
        f32::from_bits(self.inner.load(Ordering::Acquire))
    }

    fn set(&self, value: f32) {
        self.inner.store(value.to_bits(), Ordering::Release);
    }

    fn swap(&self, value: f32) -> f32 {
        f32::from_bits(self.inner.swap(value.to_bits(), Ordering::AcqRel))
    }

    fn compare_and_set(&self, current: f32, new: f32) -> Result<(), f32> {
        self.inner
            .compare_exchange(
                current.to_bits(),
                new.to_bits(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|bits| f32::from_bits(bits))
    }

    fn compare_and_exchange(&self, current: f32, new: f32) -> f32 {
        match self.inner.compare_exchange(
            current.to_bits(),
            new.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(prev) => f32::from_bits(prev),
            Err(actual) => f32::from_bits(actual),
        }
    }
}

// 注意：不实现 AtomicInteger trait
// 因为浮点数没有 increment/decrement 等整数特有操作
```

### 7.6 模块结构

```
prism3-rust-concurrent/
├── src/
│   ├── lib.rs
│   ├── atomic/                      # 新增：原子类型模块
│   │   ├── mod.rs                   # 模块导出
│   │   ├── atomic_bool.rs           # AtomicBool 实现
│   │   ├── atomic_i8.rs             # AtomicI8 实现
│   │   ├── atomic_u8.rs             # AtomicU8 实现
│   │   ├── atomic_i16.rs            # AtomicI16 实现
│   │   ├── atomic_u16.rs            # AtomicU16 实现
│   │   ├── atomic_i32.rs            # AtomicI32 实现
│   │   ├── atomic_u32.rs            # AtomicU32 实现
│   │   ├── atomic_i64.rs            # AtomicI64 实现
│   │   ├── atomic_u64.rs            # AtomicU64 实现
│   │   ├── atomic_isize.rs          # AtomicIsize 实现
│   │   ├── atomic_usize.rs          # AtomicUsize 实现
│   │   ├── atomic_f32.rs            # AtomicF32 实现（位转换）
│   │   ├── atomic_f64.rs            # AtomicF64 实现（位转换）
│   │   ├── atomic_ref.rs            # AtomicRef<T> 实现
│   │   └── traits.rs                # Atomic trait 定义
│   ├── double_checked/
│   ├── executor.rs
│   └── lock/
├── tests/
│   ├── atomic/                      # 新增：原子类型测试
│   │   ├── mod.rs
│   │   ├── atomic_bool_tests.rs
│   │   ├── atomic_i8_tests.rs
│   │   ├── atomic_u8_tests.rs
│   │   ├── atomic_i16_tests.rs
│   │   ├── atomic_u16_tests.rs
│   │   ├── atomic_i32_tests.rs
│   │   ├── atomic_u32_tests.rs
│   │   ├── atomic_i64_tests.rs
│   │   ├── atomic_u64_tests.rs
│   │   ├── atomic_isize_tests.rs
│   │   ├── atomic_usize_tests.rs
│   │   ├── atomic_f32_tests.rs
│   │   ├── atomic_f64_tests.rs
│   │   ├── atomic_ref_tests.rs
│   │   ├── trait_tests.rs           # Trait 测试
│   │   ├── concurrent_tests.rs      # 并发测试
│   │   └── performance_tests.rs     # 性能测试
│   ├── double_checked/
│   └── lock/
├── examples/
│   ├── atomic_counter_demo.rs       # 新增：计数器示例
│   ├── atomic_cas_demo.rs           # 新增：CAS 示例
│   ├── atomic_ref_demo.rs           # 新增：引用示例
│   ├── atomic_bool_demo.rs          # 新增：布尔标志示例
│   ├── atomic_float_demo.rs         # 新增：浮点数示例
│   └── atomic_performance_demo.rs   # 新增：性能对比示例
├── benches/
│   └── atomic_bench.rs              # 新增：性能基准测试
└── doc/
    └── atomic_design_zh_CN_v1.0.claude.md  # 本文档
```

### 7.6 文档注释规范

遵循项目的 Rust 文档注释规范：

```rust
/// 原子 32 位有符号整数
///
/// 提供易用的原子操作 API，自动使用合理的内存序。
/// 所有方法都是线程安全的，可以在多个线程间共享使用。
///
/// # 特性
///
/// - 自动选择合适的内存序，简化使用
/// - 提供丰富的高级操作（自增、自减、函数式更新等）
/// - 零成本抽象，性能与直接使用标准库相同
/// - 通过 `inner()` 方法可访问底层类型（高级用法）
///
/// # 使用场景
///
/// - 多线程计数器
/// - 状态标志
/// - 统计数据收集
/// - 无锁算法
///
/// # 基础示例
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicI32;
/// use std::sync::Arc;
/// use std::thread;
///
/// let counter = Arc::new(AtomicI32::new(0));
/// let mut handles = vec![];
///
/// for _ in 0..10 {
///     let counter = counter.clone();
///     let handle = thread::spawn(move || {
///         for _ in 0..1000 {
///             counter.increment_and_get();
///         }
///     });
///     handles.push(handle);
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
///
/// assert_eq!(counter.get(), 10000);
/// ```
///
/// # 高级用法：直接访问底层类型
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicI32;
/// use std::sync::atomic::Ordering;
///
/// let atomic = AtomicI32::new(0);
///
/// // 99% 的场景：使用简单 API
/// atomic.increment_and_get();
///
/// // 1% 的场景：需要精细控制内存序
/// atomic.inner().store(42, Ordering::Relaxed);
/// let value = atomic.inner().load(Ordering::SeqCst);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct AtomicI32 {
    inner: std::sync::atomic::AtomicI32,
}
```

## 8. 性能考虑

### 8.1 零成本抽象验证

使用 `#[repr(transparent)]` 和 `#[inline]` 确保编译器优化后的代码与直接使用标准库类型相同：

```rust
// 我们的封装
let atomic = AtomicI32::new(0);
let value = atomic.get();

// 编译后应该等价于
let atomic = std::sync::atomic::AtomicI32::new(0);
let value = atomic.load(Ordering::Acquire);
```

可以通过以下方式验证：

```bash
# 查看生成的汇编代码
cargo rustc --release -- --emit=asm

# 或使用 cargo-show-asm
cargo install cargo-show-asm
cargo asm --release prism3_rust_concurrent::atomic::AtomicI32::get
```

### 8.2 内存序性能对比

不同内存序的性能开销（从小到大）：

1. **Relaxed** - 几乎无开销，只保证原子性
2. **Acquire/Release** - 轻微开销，防止指令重排
3. **AcqRel** - 中等开销，结合 Acquire 和 Release
4. **SeqCst** - 最大开销，保证全局顺序一致性

### 8.3 性能优化建议

1. **纯计数场景**：如果性能关键，可以直接使用 `inner()` 配合 `Relaxed` ordering
   ```rust
   use std::sync::atomic::Ordering;

   // 性能关键路径
   counter.inner().fetch_add(1, Ordering::Relaxed);

   // 或者使用默认 API（已经使用 Relaxed）
   counter.get_and_increment();  // 内部也是 Relaxed
   ```

2. **状态同步场景**：使用默认 API（自动使用 `Acquire/Release`）
   ```rust
   if atomic.get() {
       // 读取到 true 时，之前的写入一定可见
   }
   ```

3. **CAS 循环**：使用 `compare_and_set_weak`
   ```rust
   // 弱 CAS 在某些平台上性能更好
   loop {
       match atomic.compare_and_set_weak(current, new) {
           Ok(_) => break,
           Err(actual) => current = actual,
       }
   }
   ```

4. **何时使用 `inner()`**：
   - **不需要**：大多数场景，默认 API 已经足够好
   - **需要**：极致性能优化、复杂无锁算法、需要 `SeqCst` 等特殊内存序

## 9. 与 JDK 对比

### 9.1 完整 API 对照表

#### 9.1.1 AtomicInteger (JDK) vs AtomicI32 (Rust)

| 分类 | JDK API | Rust 封装 API | 实现状态 | 说明 |
|------|---------|--------------|---------|------|
| **构造** | `new(int value)` | `new(value: i32)` | ✅ | 构造函数 |
| **基础操作** | `get()` | `get()` | ✅ | 读取当前值 |
| | `set(int newValue)` | `set(value: i32)` | ✅ | 设置新值 |
| | `lazySet(int newValue)` | `inner().store(value, Relaxed)` | ✅ | 延迟写入（通过 inner）|
| | `getAndSet(int newValue)` | `swap(value: i32)` | ✅ | 交换值（Rust 习惯命名）|
| **自增/自减** | `getAndIncrement()` | `get_and_increment()` | ✅ | 后增 |
| | `incrementAndGet()` | `increment_and_get()` | ✅ | 前增 |
| | `getAndDecrement()` | `get_and_decrement()` | ✅ | 后减 |
| | `decrementAndGet()` | `decrement_and_get()` | ✅ | 前减 |
| **算术操作** | `getAndAdd(int delta)` | `get_and_add(delta: i32)` | ✅ | 后加 |
| | `addAndGet(int delta)` | `add_and_get(delta: i32)` | ✅ | 前加 |
| | - | `get_and_sub(delta: i32)` | ✅ | 后减（Rust 特有）|
| | - | `sub_and_get(delta: i32)` | ✅ | 前减（Rust 特有）|
| **CAS 操作** | `compareAndSet(int expect, int update)` | `compare_and_set(current, new)` | ✅ | CAS，返回 Result |
| | `weakCompareAndSet(int expect, int update)` | `compare_and_set_weak(current, new)` | ✅ | 弱 CAS，返回 Result |
| | `compareAndExchange(int expect, int update)` (Java 9+) | `compare_and_exchange(current, new)` | ✅ | CAS，返回实际值 |
| | `weakCompareAndExchange(int expect, int update)` (Java 9+) | `compare_and_exchange_weak(current, new)` | ✅ | 弱 CAS，返回实际值 |
| **函数式更新** | `getAndUpdate(IntUnaryOperator f)` (Java 8+) | `get_and_update(f)` | ✅ | 函数更新，返回旧值 |
| | `updateAndGet(IntUnaryOperator f)` (Java 8+) | `update_and_get(f)` | ✅ | 函数更新，返回新值 |
| | `getAndAccumulate(int x, IntBinaryOperator f)` (Java 8+) | `get_and_accumulate(x, f)` | ✅ | 累积，返回旧值 |
| | `accumulateAndGet(int x, IntBinaryOperator f)` (Java 8+) | `accumulate_and_get(x, f)` | ✅ | 累积，返回新值 |
| **位运算** | - | `get_and_bitand(value)` | ✅ | 按位与（Rust 特有）|
| | - | `get_and_bitor(value)` | ✅ | 按位或（Rust 特有）|
| | - | `get_and_bitxor(value)` | ✅ | 按位异或（Rust 特有）|
| **最大/最小值** | - | `get_and_max(value)` | ✅ | 取最大值（Rust 特有）|
| | - | `max_and_get(value)` | ✅ | 取最大值，返回新值 |
| | - | `get_and_min(value)` | ✅ | 取最小值（Rust 特有）|
| | - | `min_and_get(value)` | ✅ | 取最小值，返回新值 |
| **类型转换** | `intValue()` | `get()` | ✅ | 直接用 get() |
| | `longValue()` | `get() as i64` | ✅ | 通过 as 转换 |
| | `floatValue()` | `get() as f32` | ✅ | 通过 as 转换 |
| | `doubleValue()` | `get() as f64` | ✅ | 通过 as 转换 |
| **其他** | `toString()` | `Display` trait | ✅ | 实现 Display |
| | - | `Debug` trait | ✅ | 实现 Debug |
| | - | `inner()` | ✅ | 访问底层类型（Rust 特有）|
| | - | `into_inner()` | ✅ | 转换为底层类型 |
| | - | `from_std(std_atomic)` | ✅ | 从标准库类型创建 |

#### 9.1.2 AtomicBoolean (JDK) vs AtomicBool (Rust)

| 分类 | JDK API | Rust 封装 API | 实现状态 | 说明 |
|------|---------|--------------|---------|------|
| **构造** | `new(boolean value)` | `new(value: bool)` | ✅ | 构造函数 |
| **基础操作** | `get()` | `get()` | ✅ | 读取当前值 |
| | `set(boolean newValue)` | `set(value: bool)` | ✅ | 设置新值 |
| | `lazySet(boolean newValue)` | `inner().store(value, Relaxed)` | ✅ | 延迟写入（通过 inner）|
| | `getAndSet(boolean newValue)` | `swap(value: bool)` | ✅ | 交换值 |
| **CAS 操作** | `compareAndSet(boolean expect, boolean update)` | `compare_and_set(current, new)` | ✅ | CAS，返回 Result |
| | `weakCompareAndSet(boolean expect, boolean update)` | `compare_and_set_weak(current, new)` | ✅ | 弱 CAS，返回 Result |
| | `compareAndExchange(boolean expect, boolean update)` (Java 9+) | `compare_and_exchange(current, new)` | ✅ | CAS，返回实际值 |
| | `weakCompareAndExchange(boolean expect, boolean update)` (Java 9+) | `compare_and_exchange_weak(current, new)` | ✅ | 弱 CAS，返回实际值 |
| **布尔特有** | - | `get_and_set()` | ✅ | 设置为 true，返回旧值（Rust 特有）|
| | - | `set_and_get()` | ✅ | 设置为 true，返回新值 |
| | - | `get_and_clear()` | ✅ | 设置为 false，返回旧值 |
| | - | `clear_and_get()` | ✅ | 设置为 false，返回新值 |
| | - | `get_and_negate()` | ✅ | 取反，返回旧值（Rust 特有）|
| | - | `negate_and_get()` | ✅ | 取反，返回新值 |
| | - | `get_and_logical_and(bool)` | ✅ | 逻辑与（Rust 特有）|
| | - | `get_and_logical_or(bool)` | ✅ | 逻辑或（Rust 特有）|
| | - | `get_and_logical_xor(bool)` | ✅ | 逻辑异或（Rust 特有）|
| | - | `compare_and_set_if_false(new)` | ✅ | 条件 CAS（Rust 特有）|
| | - | `compare_and_set_if_true(new)` | ✅ | 条件 CAS（Rust 特有）|
| **其他** | `toString()` | `Display` trait | ✅ | 实现 Display |
| | - | `inner()` | ✅ | 访问底层类型 |

#### 9.1.3 AtomicReference (JDK) vs AtomicRef (Rust)

| 分类 | JDK API | Rust 封装 API | 实现状态 | 说明 |
|------|---------|--------------|---------|------|
| **构造** | `new(V value)` | `new(value: Arc<T>)` | ✅ | 构造函数（使用 Arc）|
| **基础操作** | `get()` | `get()` | ✅ | 获取当前引用 |
| | `set(V newValue)` | `set(value: Arc<T>)` | ✅ | 设置新引用 |
| | `lazySet(V newValue)` | `inner().store(ptr, Relaxed)` | ✅ | 延迟写入（通过 inner）|
| | `getAndSet(V newValue)` | `swap(value: Arc<T>)` | ✅ | 交换引用 |
| **CAS 操作** | `compareAndSet(V expect, V update)` | `compare_and_set(&current, new)` | ✅ | CAS（指针相等性），返回 Result |
| | `weakCompareAndSet(V expect, V update)` | `compare_and_set_weak(&current, new)` | ✅ | 弱 CAS，返回 Result |
| | `compareAndExchange(V expect, V update)` (Java 9+) | `compare_and_exchange(&current, new)` | ✅ | CAS，返回实际引用 |
| | `weakCompareAndExchange(V expect, V update)` (Java 9+) | `compare_and_exchange_weak(&current, new)` | ✅ | 弱 CAS，返回实际引用 |
| **函数式更新** | `getAndUpdate(UnaryOperator<V> f)` (Java 8+) | `get_and_update(f)` | ✅ | 函数更新，返回旧引用 |
| | `updateAndGet(UnaryOperator<V> f)` (Java 8+) | `update_and_get(f)` | ✅ | 函数更新，返回新引用 |
| | `getAndAccumulate(V x, BinaryOperator<V> f)` (Java 8+) | `get_and_accumulate(x, f)` | ✅ | 累积，返回旧引用 |
| | `accumulateAndGet(V x, BinaryOperator<V> f)` (Java 8+) | `accumulate_and_get(x, f)` | ✅ | 累积，返回新引用 |
| **其他** | `toString()` | `Display` trait (如果 T: Display) | ✅ | 实现 Display |
| | - | `inner()` | ✅ | 访问底层类型 |
| | - | `Clone` trait | ✅ | 克隆原子引用 |

#### 9.1.4 JDK 没有但 Rust 提供的类型

| Rust 类型 | 说明 | 对应 JDK 类型 |
|----------|------|--------------|
| `AtomicU32` | 32位无符号整数 | - |
| `AtomicU64` | 64位无符号整数 | - |
| `AtomicIsize` | 指针大小的有符号整数 | - |
| `AtomicUsize` | 指针大小的无符号整数 | - |

#### 9.1.5 API 总结

| 特性 | JDK | Rust 封装 | 说明 |
|-----|-----|----------|------|
| **基础方法数** | ~15 个/类型 | ~25 个/类型 | Rust 提供更多便利方法 |
| **函数式方法** | Java 8+ 支持 | ✅ 支持 | 两者等价 |
| **位运算** | ❌ 不支持 | ✅ 支持 | Rust 特有（更强大）|
| **最大/最小值** | ❌ 不支持 | ✅ 支持 | Rust 特有 |
| **内存序控制** | 隐式（volatile） | 默认 + `inner()` 可选 | Rust 更灵活 |
| **类型数量** | 3 种基础类型 | 8 种基础类型 | Rust 支持更多整数类型 |

### 9.2 关键差异

| 特性 | JDK | Rust 封装 | 说明 |
|-----|-----|----------|------|
| **内存序** | 隐式（使用 volatile 语义） | 默认自动 + `inner()` 可选 | 99% 场景无需关心，1% 场景通过 `inner()` 控制 |
| **弱 CAS** | `weakCompareAndSet` | `compare_and_set_weak` | 两者等价 |
| **引用类型** | `AtomicReference<V>` | `AtomicRef<T>` | Rust 使用 `Arc<T>` |
| **可空性** | 允许 `null` | 使用 `Option<Arc<T>>` | Rust 不允许空指针 |
| **位运算** | 部分支持 | 完整支持 | Rust 支持所有位运算 |
| **最大/最小值** | Java 9+ 支持 | 支持 | 两者等价 |
| **API 数量** | ~20 个方法/类型 | ~25 个方法/类型 | Rust 不提供 `_with_ordering` 变体，API 更简洁 |

### 9.3 Rust 特有优势

1. **编译期内存安全**：完全避免数据竞争
2. **零成本抽象**：内联后无性能开销
3. **精细的内存序控制**：可根据需求选择最优内存序
4. **类型安全**：通过 trait 系统保证正确使用
5. **无垃圾回收开销**：`Arc` 使用引用计数，可预测的性能

### 9.4 compare_and_exchange 设计说明

#### 9.4.1 为什么需要 compare_and_exchange

JDK 在 Java 9 中引入了 `compareAndExchange` 方法，与 `compareAndSet` 的主要区别：

| 方法 | 返回值 | 使用场景 |
|-----|--------|---------|
| `compareAndSet` | `boolean`（成功/失败） | 只关心操作是否成功 |
| `compareAndExchange` | 实际的当前值 | 需要在失败时获取当前值继续重试 |

**优势**：在 CAS 循环中，`compareAndExchange` 可以减少一次读取操作：

```rust
// 使用 compare_and_set（需要在失败时从 Err 中提取值）
let mut current = atomic.get();
loop {
    match atomic.compare_and_set(current, new_value) {
        Ok(_) => break,
        Err(actual) => current = actual, // 从 Err 中提取
    }
}

// 使用 compare_and_exchange（更直接）
let mut current = atomic.get();
loop {
    let prev = atomic.compare_and_exchange(current, new_value);
    if prev == current {
        break; // 成功
    }
    current = prev; // 失败，prev 就是最新值
}
```

#### 9.4.2 在 Trait 中的定义

`compare_and_exchange` 被定义在 `Atomic` trait 中，所有原子类型都必须实现：

```rust
pub trait Atomic {
    type Value;

    // 返回 Result 风格
    fn compare_and_set(&self, current: Self::Value, new: Self::Value)
        -> Result<(), Self::Value>;

    // 返回实际值风格（Java 9+）
    fn compare_and_exchange(&self, current: Self::Value, new: Self::Value)
        -> Self::Value;
}
```

#### 9.4.3 实现细节

对于整数和布尔类型，实现非常直接：

```rust
impl AtomicI32 {
    pub fn compare_and_exchange(&self, current: i32, new: i32) -> i32 {
        match self.inner.compare_exchange(
            current,
            new,
            Ordering::AcqRel,    // 成功时的内存序
            Ordering::Acquire     // 失败时的内存序
        ) {
            Ok(prev) => prev,     // 成功，返回旧值（等于 current）
            Err(actual) => actual, // 失败，返回实际值
        }
    }
}
```

对于 `AtomicRef<T>`，需要处理 `Arc` 的引用计数：

```rust
impl<T> AtomicRef<T> {
    pub fn compare_and_exchange(&self, current: &Arc<T>, new: Arc<T>) -> Arc<T> {
        // 实现时需要正确管理 Arc 的引用计数
        // 详见 7.4 节 AtomicRef 实现细节
    }
}
```

#### 9.4.4 使用建议

**何时使用 `compare_and_set`**：
- 只需要知道操作是否成功
- 喜欢 Rust 的 `Result` 风格错误处理
- 代码中已经习惯使用 `match` 处理 `Result`

**何时使用 `compare_and_exchange`**：
- CAS 循环中，需要在失败时立即获取当前值
- 移植 Java 9+ 代码，保持 API 一致性
- 希望代码更简洁（少一层 `Result` 包装）

**性能对比**：
- 两者性能完全相同（编译后生成相同的代码）
- 选择哪个纯粹是 API 风格偏好

#### 9.4.5 与标准库的关系

Rust 标准库的 `std::sync::atomic` 只提供了 `compare_exchange` 方法（返回 `Result`）：

```rust
// 标准库 API
pub fn compare_exchange(
    &self,
    current: i32,
    new: i32,
    success: Ordering,
    failure: Ordering,
) -> Result<i32, i32>
```

我们的封装提供了两种风格：

1. **`compare_and_set`**：封装标准库的 `compare_exchange`，返回 `Result<(), T>`
2. **`compare_and_exchange`**：封装标准库的 `compare_exchange`，直接返回 `T`

两者都是对标准库 API 的薄封装，性能无差异。




## 10. 性能优化指南：何时使用 `inner()`

### 10.1 总体原则

**99% 的场景**：使用默认 API 就足够了，不需要调用 `inner()`。

**1% 的场景**：在性能极其关键的热点代码路径上，经过性能分析确认存在瓶颈后，才考虑使用 `inner()` 进行微调。

### 10.2 默认内存序的性能特点

我们的默认内存序策略已经过仔细设计，平衡了正确性和性能：

| 操作类型 | 默认 Ordering | 性能特点 | 典型场景 |
|---------|--------------|---------|---------|
| **读取** (`get()`) | `Acquire` | 轻量级，读屏障 | 读取共享状态 |
| **写入** (`set()`) | `Release` | 轻量级，写屏障 | 更新共享状态 |
| **RMW** (`swap()`, CAS) | `AcqRel` | 中等，读写屏障 | 原子交换 |
| **计数器** (`increment_and_get()`) | `Relaxed` | 最快，无屏障 | 纯计数统计 |

**关键点**：我们的默认策略在大多数架构上性能已经很好，不需要手动优化。

### 10.3 何时应该使用 `inner()`

#### 场景 1：高频计数器，不需要同步其他状态

```rust
use std::sync::atomic::Ordering;

// ❌ 过度使用：默认 API 已经使用 Relaxed
let counter = AtomicI32::new(0);
for _ in 0..1_000_000 {
    counter.increment_and_get();  // 内部已经是 Relaxed
}

// ✅ 默认 API 就够了
let counter = AtomicI32::new(0);
for _ in 0..1_000_000 {
    counter.increment_and_get();  // 性能最优
}

// ⚠️ 只有当你需要与默认不同的语义时才用 inner()
// 例如：需要 SeqCst 保证严格全局顺序
for _ in 0..1_000_000 {
    counter.inner().fetch_add(1, Ordering::SeqCst);  // 显式需要最强保证
}
```

#### 场景 2：延迟写入（Lazy Set）

```rust
use std::sync::atomic::Ordering;

struct Cache {
    dirty: AtomicBool,
    data: Vec<u8>,
}

impl Cache {
    fn mark_dirty(&self) {
        // ✅ 使用 Relaxed：标记为脏不需要立即对其他线程可见
        // 因为实际数据的写入会有更强的同步
        self.dirty.inner().store(true, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        // ✅ 读取时使用 Acquire 确保看到数据的变更
        self.dirty.get()  // 默认 Acquire
    }
}
```

**原因**：这是 JDK 的 `lazySet()` 模式，写入可以延迟，但读取需要同步。

#### 场景 3：自旋锁中的 Relaxed 读取

```rust
use std::sync::atomic::Ordering;

struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    fn lock(&self) {
        // 自旋等待锁释放
        while self.locked.inner().load(Ordering::Relaxed) {
            // ✅ 使用 Relaxed：频繁读取，不需要同步其他状态
            std::hint::spin_loop();
        }

        // 真正获取锁时使用 CAS（默认 AcqRel）
        while self.locked.compare_and_set(false, true).is_err() {
            while self.locked.inner().load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
    }

    fn unlock(&self) {
        // ❌ 错误：不能使用 Relaxed
        // self.locked.inner().store(false, Ordering::Relaxed);

        // ✅ 正确：释放锁必须用 Release
        self.locked.set(false);  // 默认 Release
    }
}
```

**关键点**：
- 自旋等待时的读取可以 `Relaxed`（性能关键）
- 但获取和释放锁必须用正确的内存序（默认 API 已提供）

#### 场景 4：SeqCst 保证严格全局顺序

```rust
use std::sync::atomic::Ordering;

// 某些算法需要严格的全局顺序（少见）
struct SequentialConsistencyRequired {
    flag1: AtomicBool,
    flag2: AtomicBool,
}

impl SequentialConsistencyRequired {
    fn operation(&self) {
        // ✅ 需要 SeqCst 保证全局顺序
        self.flag1.inner().store(true, Ordering::SeqCst);

        if self.flag2.inner().load(Ordering::SeqCst) {
            // 保证看到全局一致的顺序
        }
    }
}
```

**注意**：这种场景非常罕见，大多数算法用 Acquire/Release 就够了。

#### 场景 5：性能基准测试

```rust
use std::sync::atomic::Ordering;

fn benchmark_compare() {
    let counter = AtomicI32::new(0);

    // 测试默认 API（Relaxed for increment）
    let start = Instant::now();
    for _ in 0..10_000_000 {
        counter.increment_and_get();
    }
    println!("Default API: {:?}", start.elapsed());

    // 测试显式 Relaxed（应该相同）
    counter.set(0);
    let start = Instant::now();
    for _ in 0..10_000_000 {
        counter.inner().fetch_add(1, Ordering::Relaxed);
    }
    println!("Explicit Relaxed: {:?}", start.elapsed());

    // 测试 SeqCst（应该更慢）
    counter.set(0);
    let start = Instant::now();
    for _ in 0..10_000_000 {
        counter.inner().fetch_add(1, Ordering::SeqCst);
    }
    println!("SeqCst: {:?}", start.elapsed());
}
```

### 10.4 何时不应该使用 `inner()`

#### 反模式 1：没有性能瓶颈就优化

```rust
// ❌ 错误：过早优化
fn process_data() {
    let counter = AtomicI32::new(0);
    for item in items {
        // 没有证据表明这里是性能瓶颈
        counter.inner().fetch_add(1, Ordering::Relaxed);
    }
}

// ✅ 正确：使用默认 API
fn process_data() {
    let counter = AtomicI32::new(0);
    for item in items {
        counter.increment_and_get();  // 清晰且性能已经很好
    }
}
```

#### 反模式 2：误用 Relaxed 破坏同步

```rust
// ❌ 错误：使用 Relaxed 破坏了同步
let flag = AtomicBool::new(false);
let mut data = 42;

// 线程 1
data = 100;
flag.inner().store(true, Ordering::Relaxed);  // 错误！

// 线程 2
if flag.inner().load(Ordering::Relaxed) {  // 错误！
    println!("{}", data);  // 可能看到旧值 42
}

// ✅ 正确：使用默认 API
// 线程 1
data = 100;
flag.set(true);  // Release - 保证 data 的写入可见

// 线程 2
if flag.get() {  // Acquire - 保证看到 data 的更新
    println!("{}", data);  // 一定看到 100
}
```

#### 反模式 3：为了"看起来专业"而使用

```rust
// ❌ 错误：炫技
fn update_stats(&self) {
    self.counter.inner().fetch_add(1, Ordering::Relaxed);
    self.timestamp.inner().store(now(), Ordering::Release);
}

// ✅ 正确：清晰明了
fn update_stats(&self) {
    self.counter.increment_and_get();  // 已经是 Relaxed
    self.timestamp.set(now());         // 已经是 Release
}
```

### 10.5 性能优化决策树

```
是否有性能问题？
├─ 否 → 使用默认 API
└─ 是
    ├─ 已经用性能分析工具确认是瓶颈？
    │   ├─ 否 → 使用默认 API（不要猜测）
    │   └─ 是
    │       ├─ 是纯计数场景？
    │       │   ├─ 是 → 默认 API 已经是 Relaxed
    │       │   └─ 否 → 继续
    │       ├─ 需要特殊的内存序语义？
    │       │   ├─ 是 → 使用 inner()
    │       │   └─ 否 → 使用默认 API
    │       └─ 在自旋循环中频繁读取？
    │           ├─ 是 → 考虑 inner().load(Relaxed)
    │           └─ 否 → 使用默认 API
```

### 10.6 性能对比数据（参考）

以下是不同内存序在典型架构上的相对性能（数字越小越快）：

| 操作 | x86-64 | ARM64 | 说明 |
|-----|--------|-------|------|
| `Relaxed` | 1.0x | 1.0x | 基线 |
| `Acquire` (读) | 1.0x | 1.1x | x86 免费，ARM 需要屏障 |
| `Release` (写) | 1.0x | 1.1x | x86 免费，ARM 需要屏障 |
| `AcqRel` (RMW) | 1.0x | 1.2x | x86 免费，ARM 需要双屏障 |
| `SeqCst` (读) | 2.0x | 2.0x | 需要 mfence/dmb |
| `SeqCst` (写) | 2.0x | 2.0x | 需要 mfence/dmb |
| `SeqCst` (RMW) | 2.0x | 2.5x | 最重的同步 |

**结论**：
- 在 x86-64 上，`Acquire/Release/AcqRel` 几乎是免费的
- 在 ARM 上，有轻微开销，但通常可以接受
- `SeqCst` 在所有架构上都明显更慢
- 我们的默认策略（Acquire/Release/AcqRel）在各架构上都是最佳平衡

### 10.7 使用 `inner()` 的检查清单

在使用 `inner()` 之前，问自己这些问题：

- [ ] 我已经用性能分析工具（如 `cargo flamegraph`）确认这是瓶颈吗？
- [ ] 我理解不同内存序的语义和后果吗？
- [ ] 默认 API 真的不够用吗？
- [ ] 我的使用会破坏内存同步吗？
- [ ] 我在代码注释中解释了为什么需要特殊内存序吗？
- [ ] 我写了测试验证正确性吗（尤其是并发测试）？

**如果有任何一个答案是"否"，请不要使用 `inner()`。**

### 10.8 总结：黄金法则

> **默认 API 优先，`inner()` 是最后的手段。**

- 🟢 **总是先用默认 API**：99% 的情况下性能已经足够好
- 🟡 **测量再优化**：只有确认是瓶颈才考虑 `inner()`
- 🔴 **理解再使用**：使用 `inner()` 前确保理解内存序语义
- 📝 **记录原因**：如果使用了 `inner()`，在代码注释中解释为什么

**记住**：过早优化是万恶之源。清晰的代码比微小的性能提升更有价值。

## 11. 最佳实践

### 11.1 选择合适的原子类型

| 场景 | 推荐类型 | 原因 |
|-----|---------|------|
| 简单计数器 | `AtomicI32`/`AtomicU32` | 最常见，性能好 |
| 大范围计数 | `AtomicI64`/`AtomicU64` | 避免溢出 |
| 布尔标志 | `AtomicBool` | 语义清晰 |
| 指针大小的值 | `AtomicIsize`/`AtomicUsize` | 平台相关 |
| 共享配置 | `AtomicRef<Config>` | 支持复杂类型 |

### 11.2 内存序选择指南

| 场景 | 推荐内存序 | 说明 |
|-----|----------|------|
| 纯计数，无其他状态 | `Relaxed` | 最佳性能 |
| 读取共享状态 | `Acquire`（默认） | 保证读到最新值 |
| 更新共享状态 | `Release`（默认） | 保证写入可见 |
| CAS 操作 | `AcqRel`（默认） | 标准 CAS 语义 |
| 需要严格顺序 | `SeqCst` | 牺牲性能换取正确性 |

### 11.3 常见陷阱

#### 陷阱 1：不必要地使用 `inner()`

```rust
// ❌ 不推荐：不必要的显式 ordering
counter.inner().fetch_add(1, Ordering::Relaxed);

// ✅ 推荐：使用默认 API（已经是 Relaxed）
counter.get_and_increment();
```

#### 陷阱 2：通过 `inner()` 误用 `Relaxed`

```rust
use std::sync::atomic::Ordering;

// ❌ 错误：使用 Relaxed 同步标志
let flag = AtomicBool::new(false);
let mut data = 42;

// 线程 1
data = 100;
flag.inner().store(true, Ordering::Relaxed); // 错误！data 可能不可见

// 线程 2
if flag.inner().load(Ordering::Relaxed) {  // 错误！
    println!("{}", data); // 可能读到旧值 42
}

// ✅ 正确：使用默认 API（自动使用 Acquire/Release）
flag.set(true); // Release - 保证之前的写入可见
if flag.get() { // Acquire - 保证读取到最新值
    println!("{}", data); // 保证读到 100
}
```

**教训**：默认 API 已经为你选择了正确的内存序，不要画蛇添足！

#### 陷阱 3：忘记处理 CAS 失败

```rust
// ❌ 错误：忽略 CAS 失败
atomic.compare_and_set(expected, new);

// ✅ 正确：处理 CAS 结果
match atomic.compare_and_set(expected, new) {
    Ok(_) => println!("成功"),
    Err(actual) => println!("失败，当前值: {}", actual),
}
```

### 11.4 性能优化技巧

#### 技巧 1：批量操作

```rust
// ❌ 效率低：多次原子操作
for _ in 0..1000 {
    counter.increment_and_get();
}

// ✅ 效率高：一次原子操作
counter.add_and_get(1000);
```

#### 技巧 2：使用弱 CAS

```rust
// ✅ 在循环中使用弱 CAS
loop {
    match atomic.compare_and_set_weak(current, new) {
        Ok(_) => break,
        Err(actual) => current = actual,
    }
}
```

#### 技巧 3：避免不必要的读取

```rust
// ❌ 不必要的读取
let old = atomic.get();
let new = old + 1;
atomic.set(new);

// ✅ 直接使用自增
atomic.increment_and_get();
```

## 12. 与现有生态集成

### 12.1 与标准库的互操作

```rust
use std::sync::atomic::AtomicI32 as StdAtomicI32;
use std::sync::atomic::Ordering;
use prism3_rust_concurrent::atomic::AtomicI32;

impl From<StdAtomicI32> for AtomicI32 {
    fn from(std_atomic: StdAtomicI32) -> Self {
        Self::new(std_atomic.load(Ordering::Acquire))
    }
}

impl AtomicI32 {
    /// 获取底层标准库类型的引用
    ///
    /// 这是与标准库互操作的主要方法。
    #[inline]
    pub fn inner(&self) -> &StdAtomicI32 {
        &self.inner
    }

    /// 转换为标准库类型（消耗 self）
    pub fn into_inner(self) -> StdAtomicI32 {
        self.inner
    }

    /// 从标准库类型创建（零成本）
    pub const fn from_std(std_atomic: StdAtomicI32) -> Self {
        Self { inner: std_atomic }
    }
}

// 使用示例
fn interop_example() {
    // 封装类型 -> 标准库类型
    let atomic = AtomicI32::new(42);
    let std_atomic = atomic.inner();
    std_atomic.store(100, Ordering::Release);

    // 标准库类型 -> 封装类型
    let std_atomic = StdAtomicI32::new(42);
    let atomic = AtomicI32::from_std(std_atomic);
}
```

### 12.2 与 crossbeam 集成

保持与 `crossbeam-utils` 的 `AtomicCell` 兼容性：

```rust
// 可以根据需要在两者之间转换
use crossbeam_utils::atomic::AtomicCell;
use prism3_rust_concurrent::atomic::AtomicI32;

let atomic = AtomicI32::new(42);
let cell = AtomicCell::new(atomic.get());
```

### 12.3 与 parking_lot 集成

如果需要，可以提供与 `parking_lot` 的集成：

```rust
use parking_lot::Mutex;
use prism3_rust_concurrent::atomic::AtomicBool;

struct Resource {
    data: Mutex<Vec<u8>>,
    initialized: AtomicBool,
}
```



遵循项目的 Rust 文档注释规范：

```rust
/// 原子 32 位有符号整数
///
/// 提供易用的原子操作 API，自动使用合理的内存序。
/// 所有方法都是线程安全的，可以在多个线程间共享使用。
///
/// # 特性
///
/// - 自动选择合适的内存序，简化使用
/// - 提供丰富的高级操作（自增、自减、函数式更新等）
/// - 零成本抽象，性能与直接使用标准库相同
/// - 通过 `inner()` 方法可访问底层类型（高级用法）
///
/// # 使用场景
///
/// - 多线程计数器
/// - 状态标志
/// - 统计数据收集
/// - 无锁算法
///
/// # 基础示例
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicI32;
/// use std::sync::Arc;
/// use std::thread;
///
/// let counter = Arc::new(AtomicI32::new(0));
/// let mut handles = vec![];
///
/// for _ in 0..10 {
///     let counter = counter.clone();
///     let handle = thread::spawn(move || {
///         for _ in 0..1000 {
///             counter.increment_and_get();
///         }
///     });
///     handles.push(handle);
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
///
/// assert_eq!(counter.get(), 10000);
/// ```
///
/// # 高级用法：直接访问底层类型
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicI32;
/// use std::sync::atomic::Ordering;
///
/// let atomic = AtomicI32::new(0);
///
/// // 99% 的场景：使用简单 API
/// atomic.increment_and_get();
///
/// // 1% 的场景：需要精细控制内存序
/// atomic.inner().store(42, Ordering::Relaxed);
/// let value = atomic.inner().load(Ordering::SeqCst);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct AtomicI32 {
    inner: std::sync::atomic::AtomicI32,
}
```

## 13. 迁移指南

### 13.1 从标准库迁移

```rust
// 迁移前：使用标准库
use std::sync::atomic::{AtomicI32 as StdAtomicI32, Ordering};

let atomic = StdAtomicI32::new(0);
let value = atomic.load(Ordering::Acquire);
atomic.store(42, Ordering::Release);
let old = atomic.fetch_add(1, Ordering::Relaxed);

// 迁移后：使用封装（大多数情况）
use prism3_rust_concurrent::atomic::AtomicI32;

let atomic = AtomicI32::new(0);
let value = atomic.get();                // 自动 Acquire
atomic.set(42);                          // 自动 Release
let old = atomic.get_and_increment();   // 自动 Relaxed（计数器场景）

// 如果需要特殊的内存序（少数情况）
use std::sync::atomic::Ordering;
let value = atomic.inner().load(Ordering::SeqCst);
atomic.inner().store(100, Ordering::Relaxed);
```

### 13.2 分阶段迁移策略

**阶段 1：新代码使用封装**
```rust
// 新写的代码直接使用封装类型
let counter = AtomicI32::new(0);
counter.increment_and_get();
```

**阶段 2：逐步替换旧代码**
```rust
// 旧代码保持不变
let old_counter = std::sync::atomic::AtomicI32::new(0);

// 通过 from_std 桥接
let new_counter = AtomicI32::from_std(old_counter);
```

**阶段 3：性能关键路径评估**
```rust
// 如果默认内存序不满足性能需求，使用 inner()
for _ in 0..1_000_000 {
    // 性能关键：直接使用 Relaxed
    counter.inner().fetch_add(1, Ordering::Relaxed);
}
```

### 13.3 从 JDK 迁移

```rust
// Java 代码
AtomicInteger counter = new AtomicInteger(0);
int old = counter.getAndIncrement();
int current = counter.incrementAndGet();
boolean success = counter.compareAndSet(10, 20);

// Rust 等价代码
use prism3_rust_concurrent::atomic::AtomicI32;

let counter = AtomicI32::new(0);
let old = counter.get_and_increment();
let current = counter.increment_and_get();
let success = counter.compare_and_set(10, 20).is_ok();
```

## 14. 未来扩展

### 14.1 已实现的类型

本设计文档已包含以下所有类型的完整设计：

**基础整数类型**：
- ✅ `AtomicI8`, `AtomicU8` - 8位整数（直接封装标准库）
- ✅ `AtomicI16`, `AtomicU16` - 16位整数（直接封装标准库）
- ✅ `AtomicI32`, `AtomicU32` - 32位整数（直接封装标准库）
- ✅ `AtomicI64`, `AtomicU64` - 64位整数（直接封装标准库）
- ✅ `AtomicIsize`, `AtomicUsize` - 指针大小整数（直接封装标准库）

**浮点数类型**：
- ✅ `AtomicF32` - 32位浮点数（通过 `AtomicU32` + 位转换实现）
- ✅ `AtomicF64` - 64位浮点数（通过 `AtomicU64` + 位转换实现）

**其他类型**：
- ✅ `AtomicBool` - 布尔值
- ✅ `AtomicRef<T>` - 原子引用

### 14.2 可能的未来扩展

1. **原子数组**
   - `AtomicArray<T, N>`

2. **原子指针**
   - 更安全的 `AtomicPtr` 封装

3. **无锁数据结构**
   - 基于原子操作的栈、队列等

4. **统计功能**
   - 内置计数、统计功能

5. **128位整数支持**（平台依赖）
   - `AtomicI128`, `AtomicU128`
   - 需要条件编译支持
   - 建议依赖 `portable_atomic` crate
   - 在不支持的平台上降级为基于锁的实现

### 14.3 兼容性考虑

- **Rust 版本**：最低支持 Rust 1.70+
- **no_std 支持**：核心功能应支持 `no_std` 环境
- **WASM 支持**：确保在 WebAssembly 环境中正常工作

## 15. 相关资料

### 15.1 Rust 文档

- [std::sync::atomic 文档](https://doc.rust-lang.org/std/sync/atomic/)
- [Rust Atomics and Locks 书籍](https://marabos.nl/atomics/)
- [Rust 内存模型](https://doc.rust-lang.org/nomicon/atomics.html)

### 15.2 JDK 文档

- [java.util.concurrent.atomic 文档](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/util/concurrent/atomic/package-summary.html)
- [AtomicInteger Javadoc](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/util/concurrent/atomic/AtomicInteger.html)

### 15.3 论文和文章

- [C++ Memory Model](https://en.cppreference.com/w/cpp/atomic/memory_order)
- [Linux Kernel Memory Barriers](https://www.kernel.org/doc/Documentation/memory-barriers.txt)
