/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic 64-bit Floating Point
//!
//! Provides an easy-to-use atomic 64-bit floating point type with sensible
//! default memory orderings. Implemented using bit conversion with AtomicU64.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::atomic::traits::Atomic;
use crate::atomic::traits::AtomicNumber;

/// Atomic 64-bit floating point number.
///
/// Provides easy-to-use atomic operations with automatic memory ordering
/// selection. Implemented using `AtomicU64` with bit conversion.
///
/// # Memory Ordering Strategy
///
/// This type uses the same memory ordering strategy as atomic integers:
///
/// - **Read operations** (`load`): Use `Acquire` ordering to ensure
///   visibility of prior writes from other threads.
///
/// - **Write operations** (`store`): Use `Release` ordering to ensure
///   visibility of prior writes to other threads.
///
/// - **Read-Modify-Write operations** (`swap`, `compare_set`): Use
///   `AcqRel` ordering for full synchronization.
///
/// - **CAS-based arithmetic** (`fetch_add`, `fetch_sub`, etc.): Use
///   `AcqRel` on success and `Acquire` on failure within the CAS loop.
///   The loop ensures eventual consistency.
///
/// # Implementation Details
///
/// Since hardware doesn't provide native atomic floating-point operations,
/// this type is implemented using `AtomicU64` with `f64::to_bits()` and
/// `f64::from_bits()` conversions. This preserves bit patterns exactly,
/// including special values like NaN and infinity.
///
/// # Features
///
/// - Automatic memory ordering selection
/// - Arithmetic operations via CAS loops
/// - Zero-cost abstraction with inline methods
/// - Access to underlying type via `inner()` for advanced use cases
///
/// # Limitations
///
/// - Arithmetic operations use CAS loops (slower than integer operations)
/// - NaN values may cause unexpected behavior in CAS operations
/// - No max/min operations (complex floating point semantics)
///
/// # Example
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicF64;
///
/// let atomic = AtomicF64::new(3.14159);
/// atomic.add(1.0);
/// assert_eq!(atomic.load(), 4.14159);
/// ```
///
/// # Author
///
/// Haixing Hu
#[repr(transparent)]
pub struct AtomicF64 {
    inner: AtomicU64,
}

impl AtomicF64 {
    /// Creates a new atomic floating point number.
    ///
    /// # Parameters
    ///
    /// * `value` - The initial value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF64;
    ///
    /// let atomic = AtomicF64::new(3.14159);
    /// assert_eq!(atomic.load(), 3.14159);
    /// ```
    #[inline]
    pub fn new(value: f64) -> Self {
        Self {
            inner: AtomicU64::new(value.to_bits()),
        }
    }

    /// Gets the current value.
    ///
    /// # Memory Ordering
    ///
    /// Uses `Acquire` ordering on the underlying `AtomicU64`. This ensures
    /// that all writes from other threads that happened before a `Release`
    /// store are visible after this load.
    ///
    /// # Returns
    ///
    /// The current value.
    #[inline]
    pub fn load(&self) -> f64 {
        f64::from_bits(self.inner.load(Ordering::Acquire))
    }

    /// Sets a new value.
    ///
    /// # Memory Ordering
    ///
    /// Uses `Release` ordering on the underlying `AtomicU64`. This ensures
    /// that all prior writes in this thread are visible to other threads
    /// that perform an `Acquire` load.
    ///
    /// # Parameters
    ///
    /// * `value` - The new value to set.
    #[inline]
    pub fn store(&self, value: f64) {
        self.inner.store(value.to_bits(), Ordering::Release);
    }

    /// Swaps the current value with a new value, returning the old value.
    ///
    /// # Memory Ordering
    ///
    /// Uses `AcqRel` ordering on the underlying `AtomicU64`. This provides
    /// full synchronization for this read-modify-write operation.
    ///
    /// # Parameters
    ///
    /// * `value` - The new value to swap in.
    ///
    /// # Returns
    ///
    /// The old value.
    #[inline]
    pub fn swap(&self, value: f64) -> f64 {
        f64::from_bits(self.inner.swap(value.to_bits(), Ordering::AcqRel))
    }

    /// Compares and sets the value atomically.
    ///
    /// If the current value equals `current`, sets it to `new` and returns
    /// `Ok(())`. Otherwise, returns `Err(actual)` where `actual` is the
    /// current value.
    ///
    /// # Memory Ordering
    ///
    /// - **Success**: Uses `AcqRel` ordering on the underlying `AtomicU64`
    ///   to ensure full synchronization when the exchange succeeds.
    /// - **Failure**: Uses `Acquire` ordering to observe the actual value
    ///   written by another thread.
    ///
    /// # Parameters
    ///
    /// * `current` - The expected current value.
    /// * `new` - The new value to set if current matches.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(actual)` on failure.
    #[inline]
    pub fn compare_set(&self, current: f64, new: f64) -> Result<(), f64> {
        self.inner
            .compare_exchange(
                current.to_bits(),
                new.to_bits(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(f64::from_bits)
    }

    /// Weak version of compare-and-set.
    ///
    /// May spuriously fail even when the comparison succeeds. Should be used
    /// in a loop.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `current` - The expected current value.
    /// * `new` - The new value to set if current matches.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(actual)` on failure.
    #[inline]
    pub fn compare_set_weak(&self, current: f64, new: f64) -> Result<(), f64> {
        self.inner
            .compare_exchange_weak(
                current.to_bits(),
                new.to_bits(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(f64::from_bits)
    }

    /// Compares and exchanges the value atomically, returning the previous
    /// value.
    ///
    /// If the current value equals `current`, sets it to `new` and returns
    /// the old value. Otherwise, returns the actual current value.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `current` - The expected current value.
    /// * `new` - The new value to set if current matches.
    ///
    /// # Returns
    ///
    /// The value before the operation.
    #[inline]
    pub fn compare_and_exchange(&self, current: f64, new: f64) -> f64 {
        match self.inner.compare_exchange(
            current.to_bits(),
            new.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(prev_bits) => f64::from_bits(prev_bits),
            Err(actual_bits) => f64::from_bits(actual_bits),
        }
    }

    /// Weak version of compare-and-exchange.
    ///
    /// May spuriously fail even when the comparison succeeds. Should be used
    /// in a loop.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `current` - The expected current value.
    /// * `new` - The new value to set if current matches.
    ///
    /// # Returns
    ///
    /// The value before the operation.
    #[inline]
    pub fn compare_and_exchange_weak(&self, current: f64, new: f64) -> f64 {
        match self.inner.compare_exchange_weak(
            current.to_bits(),
            new.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(prev_bits) => f64::from_bits(prev_bits),
            Err(actual_bits) => f64::from_bits(actual_bits),
        }
    }

    /// Atomically adds a value, returning the old value.
    ///
    /// # Memory Ordering
    ///
    /// Internally uses a CAS loop with `compare_set_weak`, which uses
    /// `AcqRel` on success and `Acquire` on failure. The loop ensures
    /// eventual consistency even under high contention.
    ///
    /// # Performance
    ///
    /// May be slow in high-contention scenarios due to the CAS loop.
    /// Consider using atomic integers if performance is critical.
    ///
    /// # Parameters
    ///
    /// * `delta` - The value to add.
    ///
    /// # Returns
    ///
    /// The old value before adding.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF64;
    ///
    /// let atomic = AtomicF64::new(10.0);
    /// let old = atomic.fetch_add(5.5);
    /// assert_eq!(old, 10.0);
    /// assert_eq!(atomic.load(), 15.5);
    /// ```
    #[inline]
    pub fn fetch_add(&self, delta: f64) -> f64 {
        let mut current = self.load();
        loop {
            let new = current + delta;
            match self.compare_set_weak(current, new) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }

    /// Atomically subtracts a value, returning the old value.
    ///
    /// # Memory Ordering
    ///
    /// Internally uses a CAS loop with `compare_set_weak`, which uses
    /// `AcqRel` on success and `Acquire` on failure. The loop ensures
    /// eventual consistency even under high contention.
    ///
    /// # Parameters
    ///
    /// * `delta` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The old value before subtracting.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF64;
    ///
    /// let atomic = AtomicF64::new(10.0);
    /// let old = atomic.fetch_sub(3.5);
    /// assert_eq!(old, 10.0);
    /// assert_eq!(atomic.load(), 6.5);
    /// ```
    #[inline]
    pub fn fetch_sub(&self, delta: f64) -> f64 {
        let mut current = self.load();
        loop {
            let new = current - delta;
            match self.compare_set_weak(current, new) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }

    /// Atomically multiplies by a factor, returning the old value.
    ///
    /// # Memory Ordering
    ///
    /// Internally uses a CAS loop with `compare_set_weak`, which uses
    /// `AcqRel` on success and `Acquire` on failure. The loop ensures
    /// eventual consistency even under high contention.
    ///
    /// # Parameters
    ///
    /// * `factor` - The factor to multiply by.
    ///
    /// # Returns
    ///
    /// The old value before multiplying.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF64;
    ///
    /// let atomic = AtomicF64::new(10.0);
    /// let old = atomic.fetch_mul(2.5);
    /// assert_eq!(old, 10.0);
    /// assert_eq!(atomic.load(), 25.0);
    /// ```
    #[inline]
    pub fn fetch_mul(&self, factor: f64) -> f64 {
        let mut current = self.load();
        loop {
            let new = current * factor;
            match self.compare_set_weak(current, new) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }

    /// Atomically divides by a divisor, returning the old value.
    ///
    /// # Memory Ordering
    ///
    /// Internally uses a CAS loop with `compare_set_weak`, which uses
    /// `AcqRel` on success and `Acquire` on failure. The loop ensures
    /// eventual consistency even under high contention.
    ///
    /// # Parameters
    ///
    /// * `divisor` - The divisor to divide by.
    ///
    /// # Returns
    ///
    /// The old value before dividing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF64;
    ///
    /// let atomic = AtomicF64::new(10.0);
    /// let old = atomic.fetch_div(2.0);
    /// assert_eq!(old, 10.0);
    /// assert_eq!(atomic.load(), 5.0);
    /// ```
    #[inline]
    pub fn fetch_div(&self, divisor: f64) -> f64 {
        let mut current = self.load();
        loop {
            let new = current / divisor;
            match self.compare_set_weak(current, new) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }

    /// Updates the value using a function, returning the old value.
    ///
    /// # Memory Ordering
    ///
    /// Internally uses a CAS loop with `compare_set_weak`, which uses
    /// `AcqRel` on success and `Acquire` on failure. The loop ensures
    /// eventual consistency even under high contention.
    ///
    /// # Parameters
    ///
    /// * `f` - A function that takes the current value and returns the new
    ///   value.
    ///
    /// # Returns
    ///
    /// The old value before the update.
    #[inline]
    pub fn fetch_update<F>(&self, f: F) -> f64
    where
        F: Fn(f64) -> f64,
    {
        let mut current = self.load();
        loop {
            let new = f(current);
            match self.compare_set_weak(current, new) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }

    /// Gets a reference to the underlying standard library atomic type.
    ///
    /// This allows direct access to the standard library's atomic operations
    /// for advanced use cases that require fine-grained control over memory
    /// ordering.
    ///
    /// # Memory Ordering
    ///
    /// When using the returned reference, you have full control over memory
    /// ordering. Remember to use `f64::to_bits()` and `f64::from_bits()` for
    /// conversions.
    ///
    /// # Returns
    ///
    /// A reference to the underlying `std::sync::atomic::AtomicU64`.
    #[inline]
    pub fn inner(&self) -> &AtomicU64 {
        &self.inner
    }
}

impl Atomic for AtomicF64 {
    type Value = f64;

    #[inline]
    fn load(&self) -> f64 {
        self.load()
    }

    #[inline]
    fn store(&self, value: f64) {
        self.store(value);
    }

    #[inline]
    fn swap(&self, value: f64) -> f64 {
        self.swap(value)
    }

    #[inline]
    fn compare_set(&self, current: f64, new: f64) -> Result<(), f64> {
        self.compare_set(current, new)
    }

    #[inline]
    fn compare_set_weak(&self, current: f64, new: f64) -> Result<(), f64> {
        self.compare_set_weak(current, new)
    }

    #[inline]
    fn compare_exchange(&self, current: f64, new: f64) -> f64 {
        self.compare_and_exchange(current, new)
    }

    #[inline]
    fn compare_exchange_weak(&self, current: f64, new: f64) -> f64 {
        self.compare_and_exchange_weak(current, new)
    }

    #[inline]
    fn fetch_update<F>(&self, f: F) -> f64
    where
        F: Fn(f64) -> f64,
    {
        self.fetch_update(f)
    }
}

impl AtomicNumber for AtomicF64 {
    #[inline]
    fn fetch_add(&self, delta: f64) -> f64 {
        self.fetch_add(delta)
    }

    #[inline]
    fn fetch_sub(&self, delta: f64) -> f64 {
        self.fetch_sub(delta)
    }

    #[inline]
    fn fetch_mul(&self, factor: f64) -> f64 {
        self.fetch_mul(factor)
    }

    #[inline]
    fn fetch_div(&self, divisor: f64) -> f64 {
        self.fetch_div(divisor)
    }
}

unsafe impl Send for AtomicF64 {}
unsafe impl Sync for AtomicF64 {}

impl Default for AtomicF64 {
    #[inline]
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl From<f64> for AtomicF64 {
    #[inline]
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for AtomicF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicF64")
            .field("value", &self.load())
            .finish()
    }
}

impl fmt::Display for AtomicF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.load())
    }
}
