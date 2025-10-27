/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic Integer Macro
//!
//! Provides a macro to generate atomic integer types with consistent
//! implementations.
//!
//! # Author
//!
//! Haixing Hu

/// Macro to generate atomic integer types.
///
/// This macro generates a complete atomic integer type with all methods,
/// trait implementations, and documentation.
///
/// # Parameters
///
/// * `$name` - The name of the atomic type (e.g., `AtomicI32`)
/// * `$inner_type` - The underlying std atomic type (e.g.,
///   `std::sync::atomic::AtomicI32`)
/// * `$value_type` - The value type (e.g., `i32`)
/// * `$doc_type` - The type description for documentation (e.g., "32-bit
///   signed integer")
macro_rules! impl_atomic_integer {
    ($name:ident, $inner_type:ty, $value_type:ty, $doc_type:expr) => {
        #[doc = concat!("Atomic ", $doc_type, ".")]
        ///
        /// Provides easy-to-use atomic operations with automatic memory
        /// ordering selection. All methods are thread-safe and can be
        /// shared across threads.
        ///
        /// # Memory Ordering Strategy
        ///
        /// This type uses carefully chosen default memory orderings:
        ///
        /// - **Read operations** (`load`): Use `Acquire` ordering to
        ///   ensure visibility of writes from other threads.
        /// - **Write operations** (`store`): Use `Release` ordering to
        ///   ensure writes are visible to other threads.
        /// - **Read-Modify-Write** (`swap`, CAS): Use `AcqRel` ordering
        ///   for both read and write synchronization.
        /// - **Arithmetic operations** (`fetch_inc`, `fetch_add`, etc.):
        ///   Use `Relaxed` ordering for optimal performance in pure
        ///   counting scenarios where no other data needs synchronization.
        /// - **Bit operations** (`fetch_and`, `fetch_or`, etc.): Use
        ///   `AcqRel` ordering as they typically synchronize flag states.
        /// - **Max/Min operations**: Use `AcqRel` ordering as they often
        ///   coordinate with threshold-based logic.
        ///
        /// For advanced use cases requiring different memory orderings,
        /// use `inner()` to access the underlying standard library type.
        ///
        /// # Features
        ///
        /// - Automatic memory ordering selection
        /// - Rich set of integer operations (increment, decrement,
        ///   arithmetic, etc.)
        /// - Zero-cost abstraction with inline methods
        /// - Access to underlying type via `inner()` for advanced use
        ///   cases
        ///
        /// # Example
        ///
        /// ```rust
        #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
        /// use std::sync::Arc;
        /// use std::thread;
        ///
        #[doc = concat!("let counter = Arc::new(", stringify!($name), "::new(0));")]
        /// let mut handles = vec![];
        ///
        /// for _ in 0..10 {
        ///     let counter = counter.clone();
        ///     let handle = thread::spawn(move || {
        ///         for _ in 0..100 {
        ///             counter.fetch_inc();
        ///         }
        ///     });
        ///     handles.push(handle);
        /// }
        ///
        /// for handle in handles {
        ///     handle.join().unwrap();
        /// }
        ///
        /// assert_eq!(counter.load(), 1000);
        /// ```
        ///
        /// # Author
        ///
        /// Haixing Hu
        #[repr(transparent)]
        pub struct $name {
            inner: $inner_type,
        }

        impl $name {
            /// Creates a new atomic integer.
            ///
            /// # Parameters
            ///
            /// * `value` - The initial value.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(42);")]
            /// assert_eq!(atomic.load(), 42);
            /// ```
            #[inline]
            pub const fn new(value: $value_type) -> Self {
                Self {
                    inner: <$inner_type>::new(value),
                }
            }

            /// Loads the current value.
            ///
            /// # Memory Ordering
            ///
            /// Uses `Acquire` ordering to ensure that:
            /// - This load operation happens-before any subsequent memory
            ///   operations in the current thread.
            /// - If another thread performed a `Release` store, all writes
            ///   before that store are visible after this load.
            ///
            /// This is the standard choice for reading shared state that
            /// may have been modified by other threads.
            ///
            /// # Returns
            ///
            /// The current value.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(42);")]
            /// assert_eq!(atomic.load(), 42);
            /// ```
            #[inline]
            pub fn load(&self) -> $value_type {
                self.inner.load(Ordering::Acquire)
            }

            /// Stores a new value.
            ///
            /// # Memory Ordering
            ///
            /// Uses `Release` ordering to ensure that:
            /// - All memory operations before this store in the current
            ///   thread happen-before the store.
            /// - When another thread performs an `Acquire` load and sees
            ///   this value, all writes before this store become visible.
            ///
            /// This is the standard choice for publishing data to other
            /// threads.
            ///
            /// # Parameters
            ///
            /// * `value` - The new value to store.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0);")]
            /// atomic.store(42);
            /// assert_eq!(atomic.load(), 42);
            /// ```
            #[inline]
            pub fn store(&self, value: $value_type) {
                self.inner.store(value, Ordering::Release);
            }

            /// Swaps the current value with a new value, returning the old
            /// value.
            ///
            /// # Memory Ordering
            ///
            /// Uses `AcqRel` ordering to ensure that:
            /// - **Acquire**: Reads the current value and establishes
            ///   happens-before with prior `Release` operations.
            /// - **Release**: Writes the new value and makes all prior
            ///   writes visible to subsequent `Acquire` operations.
            ///
            /// This is the standard choice for atomic exchange operations
            /// that both read and write.
            ///
            /// # Parameters
            ///
            /// * `value` - The new value to swap in.
            ///
            /// # Returns
            ///
            /// The old value.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.swap(20);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.load(), 20);
            /// ```
            #[inline]
            pub fn swap(&self, value: $value_type) -> $value_type {
                self.inner.swap(value, Ordering::AcqRel)
            }

            /// Compares and sets the value atomically.
            ///
            /// If the current value equals `current`, sets it to `new` and
            /// returns `Ok(())`. Otherwise, returns `Err(actual)` where
            /// `actual` is the current value.
            ///
            /// # Memory Ordering
            ///
            /// - **Success**: Uses `AcqRel` ordering (both Acquire and
            ///   Release) to synchronize with other threads.
            /// - **Failure**: Uses `Acquire` ordering to see the latest
            ///   value written by other threads.
            ///
            /// This is the standard CAS pattern: on success, we need
            /// Release to publish our write; on failure, we need Acquire
            /// to see what value actually exists.
            ///
            /// # Parameters
            ///
            /// * `current` - The expected current value.
            /// * `new` - The new value to set if current matches.
            ///
            /// # Returns
            ///
            /// `Ok(())` on success, or `Err(actual)` on failure.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// assert!(atomic.compare_set(10, 20).is_ok());
            /// assert_eq!(atomic.load(), 20);
            /// ```
            #[inline]
            pub fn compare_set(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> Result<(), $value_type> {
                self.inner
                    .compare_exchange(
                        current,
                        new,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .map(|_| ())
            }

            /// Weak version of compare-and-set.
            ///
            /// May spuriously fail even when the comparison succeeds. Should
            /// be used in a loop.
            ///
            /// Uses `AcqRel` ordering on success and `Acquire` ordering on
            /// failure.
            ///
            /// # Parameters
            ///
            /// * `current` - The expected current value.
            /// * `new` - The new value to set if current matches.
            ///
            /// # Returns
            ///
            /// `Ok(())` on success, or `Err(actual)` on failure.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let mut current = atomic.load();
            /// loop {
            ///     match atomic.compare_set_weak(current, current + 1) {
            ///         Ok(_) => break,
            ///         Err(actual) => current = actual,
            ///     }
            /// }
            /// assert_eq!(atomic.load(), 11);
            /// ```
            #[inline]
            pub fn compare_set_weak(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> Result<(), $value_type> {
                self.inner
                    .compare_exchange_weak(
                        current,
                        new,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .map(|_| ())
            }

            /// Compares and exchanges the value atomically, returning the
            /// previous value.
            ///
            /// If the current value equals `current`, sets it to `new` and
            /// returns the old value. Otherwise, returns the actual current
            /// value.
            ///
            /// Uses `AcqRel` ordering on success and `Acquire` ordering on
            /// failure.
            ///
            /// # Parameters
            ///
            /// * `current` - The expected current value.
            /// * `new` - The new value to set if current matches.
            ///
            /// # Returns
            ///
            /// The value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let prev = atomic.compare_and_exchange(10, 20);
            /// assert_eq!(prev, 10);
            /// assert_eq!(atomic.load(), 20);
            /// ```
            #[inline]
            pub fn compare_and_exchange(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> $value_type {
                match self.inner.compare_exchange(
                    current,
                    new,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(prev) => prev,
                    Err(actual) => actual,
                }
            }

            /// Weak version of compare-and-exchange.
            ///
            /// May spuriously fail even when the comparison succeeds. Should
            /// be used in a loop.
            ///
            /// Uses `AcqRel` ordering on success and `Acquire` ordering on
            /// failure.
            ///
            /// # Parameters
            ///
            /// * `current` - The expected current value.
            /// * `new` - The new value to set if current matches.
            ///
            /// # Returns
            ///
            /// The value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let mut current = atomic.load();
            /// loop {
            ///     let prev =
            ///         atomic.compare_and_exchange_weak(current, current + 5);
            ///     if prev == current {
            ///         break;
            ///     }
            ///     current = prev;
            /// }
            /// assert_eq!(atomic.load(), 15);
            /// ```
            #[inline]
            pub fn compare_and_exchange_weak(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> $value_type {
                match self.inner.compare_exchange_weak(
                    current,
                    new,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(prev) => prev,
                    Err(actual) => actual,
                }
            }

            /// Increments the value by 1, returning the old value.
            ///
            /// # Memory Ordering
            ///
            /// Uses `Relaxed` ordering for optimal performance. This is
            /// appropriate for pure counters where:
            /// - Only the counter value itself needs to be atomic.
            /// - No other data needs to be synchronized.
            /// - The typical use case is statistics/counting.
            ///
            /// **Rationale**: Counters are the most common use of atomic
            /// integers. Using `Relaxed` provides maximum performance
            /// (especially on ARM) while maintaining correctness for the
            /// counter value itself. If you need to synchronize other
            /// data, use `load()`/`store()` with their stronger orderings.
            ///
            /// # Returns
            ///
            /// The old value before incrementing.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.fetch_inc();
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.load(), 11);
            /// ```
            #[inline]
            pub fn fetch_inc(&self) -> $value_type {
                self.inner.fetch_add(1, Ordering::Relaxed)
            }

            /// Decrements the value by 1, returning the old value.
            ///
            /// Uses `Relaxed` ordering.
            ///
            /// # Returns
            ///
            /// The old value before decrementing.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.fetch_dec();
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.load(), 9);
            /// ```
            #[inline]
            pub fn fetch_dec(&self) -> $value_type {
                self.inner.fetch_sub(1, Ordering::Relaxed)
            }

            /// Adds a delta to the value, returning the old value.
            ///
            /// # Memory Ordering
            ///
            /// Uses `Relaxed` ordering for the same reasons as
            /// `fetch_inc()`: optimal performance for pure counting
            /// scenarios. See `fetch_inc()` documentation for detailed
            /// rationale.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.fetch_add(5);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.load(), 15);
            /// ```
            #[inline]
            pub fn fetch_add(&self, delta: $value_type) -> $value_type {
                self.inner.fetch_add(delta, Ordering::Relaxed)
            }

            /// Subtracts a delta from the value, returning the old value.
            ///
            /// Uses `Relaxed` ordering.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.fetch_sub(3);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.load(), 7);
            /// ```
            #[inline]
            pub fn fetch_sub(&self, delta: $value_type) -> $value_type {
                self.inner.fetch_sub(delta, Ordering::Relaxed)
            }

            /// Multiplies the value by a factor, returning the old value.
            ///
            /// Uses `Relaxed` ordering for the operation.
            ///
            /// # Parameters
            ///
            /// * `factor` - The factor to multiply by.
            ///
            /// # Returns
            ///
            /// The old value before multiplication.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.fetch_mul(3);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.load(), 30);
            /// ```
            #[inline]
            pub fn fetch_mul(&self, factor: $value_type) -> $value_type {
                let mut current = self.load();
                loop {
                    let new = current.wrapping_mul(factor);
                    match self.compare_set(current, new) {
                        Ok(_) => return current,
                        Err(actual) => current = actual,
                    }
                }
            }

            /// Divides the value by a divisor, returning the old value.
            ///
            /// Uses `Relaxed` ordering for the operation.
            ///
            /// # Parameters
            ///
            /// * `divisor` - The divisor to divide by.
            ///
            /// # Returns
            ///
            /// The old value before division.
            ///
            /// # Panics
            ///
            /// Panics if `divisor` is zero.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(30);")]
            /// let old = atomic.fetch_div(3);
            /// assert_eq!(old, 30);
            /// assert_eq!(atomic.load(), 10);
            /// ```
            #[inline]
            pub fn fetch_div(&self, divisor: $value_type) -> $value_type {
                assert!(divisor != 0, "division by zero");
                let mut current = self.load();
                loop {
                    let new = current.wrapping_div(divisor);
                    match self.compare_set(current, new) {
                        Ok(_) => return current,
                        Err(actual) => current = actual,
                    }
                }
            }


            /// Performs bitwise AND, returning the old value.
            ///
            /// # Memory Ordering
            ///
            /// Uses `AcqRel` ordering because bit operations typically
            /// manipulate flag bits that coordinate access to other data.
            ///
            /// **Rationale**: Unlike pure counters, bit operations are
            /// commonly used for:
            /// - State flags (INITIALIZED, RUNNING, STOPPED, etc.)
            /// - Feature toggles
            /// - Permission masks
            ///
            /// These scenarios require synchronization with related data,
            /// so we use the stronger `AcqRel` ordering by default.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to AND with.
            ///
            /// # Returns
            ///
            /// The old value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0b1111);")]
            /// let old = atomic.fetch_and(0b1100);
            /// assert_eq!(old, 0b1111);
            /// assert_eq!(atomic.load(), 0b1100);
            /// ```
            #[inline]
            pub fn fetch_and(&self, value: $value_type) -> $value_type {
                self.inner.fetch_and(value, Ordering::AcqRel)
            }

            /// Performs bitwise OR, returning the old value.
            ///
            /// Uses `AcqRel` ordering.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to OR with.
            ///
            /// # Returns
            ///
            /// The old value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0b1100);")]
            /// let old = atomic.fetch_or(0b0011);
            /// assert_eq!(old, 0b1100);
            /// assert_eq!(atomic.load(), 0b1111);
            /// ```
            #[inline]
            pub fn fetch_or(&self, value: $value_type) -> $value_type {
                self.inner.fetch_or(value, Ordering::AcqRel)
            }

            /// Performs bitwise XOR, returning the old value.
            ///
            /// Uses `AcqRel` ordering.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to XOR with.
            ///
            /// # Returns
            ///
            /// The old value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0b1100);")]
            /// let old = atomic.fetch_xor(0b0110);
            /// assert_eq!(old, 0b1100);
            /// assert_eq!(atomic.load(), 0b1010);
            /// ```
            #[inline]
            pub fn fetch_xor(&self, value: $value_type) -> $value_type {
                self.inner.fetch_xor(value, Ordering::AcqRel)
            }

            /// Performs bitwise NOT, returning the old value.
            ///
            /// This is a convenience method equivalent to
            /// `fetch_xor(-1)`. Uses `AcqRel` ordering.
            ///
            /// # Returns
            ///
            /// The old value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0b1010_0101);")]
            /// let old = atomic.fetch_not();
            /// assert_eq!(old, 0b1010_0101);
            #[doc = concat!("assert_eq!(atomic.load(), !0b1010_0101_", stringify!($value_type), ");")]
            /// ```
            ///
            /// # Note
            ///
            /// This method is implemented using `fetch_xor(-1)` because
            /// hardware and LLVM do not provide a native atomic NOT
            /// instruction. The compiler will optimize this into
            /// efficient machine code.
            #[inline]
            pub fn fetch_not(&self) -> $value_type {
                self.inner.fetch_xor(!0, Ordering::AcqRel)
            }

            /// Updates the value using a function, returning the old value.
            ///
            /// Internally uses a CAS loop until the update succeeds.
            ///
            /// # Parameters
            ///
            /// * `f` - A function that takes the current value and returns
            ///   the new value.
            ///
            /// # Returns
            ///
            /// The old value before the update.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.fetch_update(|x| x * 2);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.load(), 20);
            /// ```
            #[inline]
            pub fn fetch_update<F>(&self, f: F) -> $value_type
            where
                F: Fn($value_type) -> $value_type,
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

            /// Accumulates a value using a binary function, returning the
            /// old value.
            ///
            /// Internally uses a CAS loop until the update succeeds.
            ///
            /// # Parameters
            ///
            /// * `x` - The value to accumulate with.
            /// * `f` - A binary function that takes the current value and
            ///   `x`, returning the new value.
            ///
            /// # Returns
            ///
            /// The old value before the accumulation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.fetch_accumulate(5, |a, b| a + b);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.load(), 15);
            /// ```
            #[inline]
            pub fn fetch_accumulate<F>(
                &self,
                x: $value_type,
                f: F,
            ) -> $value_type
            where
                F: Fn($value_type, $value_type) -> $value_type,
            {
                let mut current = self.load();
                loop {
                    let new = f(current, x);
                    match self.compare_set_weak(current, new) {
                        Ok(_) => return current,
                        Err(actual) => current = actual,
                    }
                }
            }

            /// Sets the value to the maximum of the current value and the
            /// given value, returning the old value.
            ///
            /// # Memory Ordering
            ///
            /// Uses `AcqRel` ordering because max/min operations often
            /// coordinate with threshold-based logic and related metadata.
            ///
            /// **Rationale**: Common use cases include:
            /// - Peak value tracking with timestamp recording
            /// - High-water marks that trigger alerts
            /// - Resource allocation thresholds
            ///
            /// These scenarios typically need to synchronize with other
            /// data (timestamps, alert states, etc.), so we use `AcqRel`
            /// for safety.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to compare with.
            ///
            /// # Returns
            ///
            /// The old value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// atomic.fetch_max(20);
            /// assert_eq!(atomic.load(), 20);
            ///
            /// atomic.fetch_max(15);
            /// assert_eq!(atomic.load(), 20);
            /// ```
            #[inline]
            pub fn fetch_max(&self, value: $value_type) -> $value_type {
                self.inner.fetch_max(value, Ordering::AcqRel)
            }

            /// Sets the value to the minimum of the current value and the
            /// given value, returning the old value.
            ///
            /// Uses `AcqRel` ordering.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to compare with.
            ///
            /// # Returns
            ///
            /// The old value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// atomic.fetch_min(5);
            /// assert_eq!(atomic.load(), 5);
            ///
            /// atomic.fetch_min(8);
            /// assert_eq!(atomic.load(), 5);
            /// ```
            #[inline]
            pub fn fetch_min(&self, value: $value_type) -> $value_type {
                self.inner.fetch_min(value, Ordering::AcqRel)
            }

            /// Gets a reference to the underlying standard library atomic
            /// type.
            ///
            /// This allows direct access to the standard library's atomic
            /// operations for advanced use cases that require fine-grained
            /// control over memory ordering.
            ///
            /// # Returns
            ///
            #[doc = concat!("A reference to the underlying `std::sync::atomic::", stringify!($inner_type), "`.")]
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            /// use std::sync::atomic::Ordering;
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0);")]
            /// atomic.inner().store(42, Ordering::Relaxed);
            /// assert_eq!(atomic.inner().load(Ordering::Relaxed), 42);
            /// ```
            #[inline]
            pub fn inner(&self) -> &$inner_type {
                &self.inner
            }
        }

        // Trait implementations: These methods forward to the struct's
        // direct implementations for API consistency. This design allows
        // both convenient direct method calls (atomic.load()) and generic
        // trait-based programming (fn foo<T: Atomic>(atomic: &T)).

        impl crate::atomic::traits::Atomic for $name {
            type Value = $value_type;

            #[inline]
            fn load(&self) -> $value_type {
                self.load()
            }

            #[inline]
            fn store(&self, value: $value_type) {
                self.store(value);
            }

            #[inline]
            fn swap(&self, value: $value_type) -> $value_type {
                self.swap(value)
            }

            #[inline]
            fn compare_set(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> Result<(), $value_type> {
                self.compare_set(current, new)
            }

            #[inline]
            fn compare_set_weak(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> Result<(), $value_type> {
                self.compare_set_weak(current, new)
            }

            #[inline]
            fn compare_exchange(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> $value_type {
                self.compare_and_exchange(current, new)
            }

            #[inline]
            fn compare_exchange_weak(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> $value_type {
                self.compare_and_exchange_weak(current, new)
            }

            #[inline]
            fn fetch_update<F>(&self, f: F) -> $value_type
            where
                F: Fn($value_type) -> $value_type,
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
        }

        impl crate::atomic::traits::AtomicNumber for $name {
            #[inline]
            fn fetch_add(&self, delta: $value_type) -> $value_type {
                self.fetch_add(delta)
            }

            #[inline]
            fn fetch_sub(&self, delta: $value_type) -> $value_type {
                self.fetch_sub(delta)
            }

            #[inline]
            fn fetch_mul(&self, factor: $value_type) -> $value_type {
                self.fetch_mul(factor)
            }

            #[inline]
            fn fetch_div(&self, divisor: $value_type) -> $value_type {
                self.fetch_div(divisor)
            }
        }

        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                Self::new(0)
            }
        }

        impl From<$value_type> for $name {
            #[inline]
            fn from(value: $value_type) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("value", &self.load())
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.load())
            }
        }
    };
}

pub(crate) use impl_atomic_integer;
