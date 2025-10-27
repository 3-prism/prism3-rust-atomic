/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

/// Macro to generate comprehensive tests for atomic integer types.
#[macro_export]
macro_rules! test_atomic_integer {
    ($atomic_type:ident, $value_type:ty, $test_mod:ident) => {
        mod $test_mod {
            use prism3_atomic::atomic::{
                $atomic_type,
                Atomic,
                AtomicNumber,
            };
            use std::sync::Arc;
            use std::thread;

            #[test]
            fn test_new() {
                let atomic = <$atomic_type>::new(42);
                assert_eq!(atomic.load(), 42);
            }

            #[test]
            fn test_default() {
                let atomic = <$atomic_type>::default();
                assert_eq!(atomic.load(), 0);
            }

            #[test]
            fn test_from() {
                let atomic = <$atomic_type>::from(100);
                assert_eq!(atomic.load(), 100);
            }

            #[test]
            fn test_get_set() {
                let atomic = <$atomic_type>::new(0);
                atomic.store(42);
                assert_eq!(atomic.load(), 42);
                atomic.store(10);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_swap() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.swap(20);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 20);
            }

            #[test]
            fn test_compare_and_set_success() {
                let atomic = <$atomic_type>::new(10);
                assert!(atomic.compare_set(10, 20).is_ok());
                assert_eq!(atomic.load(), 20);
            }

            #[test]
            fn test_compare_and_set_failure() {
                let atomic = <$atomic_type>::new(10);
                match atomic.compare_set(15, 20) {
                    Ok(_) => panic!("Should fail"),
                    Err(actual) => assert_eq!(actual, 10),
                }
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_compare_and_exchange() {
                let atomic = <$atomic_type>::new(10);
                let prev = atomic.compare_exchange(10, 20);
                assert_eq!(prev, 10);
                assert_eq!(atomic.load(), 20);

                let prev = atomic.compare_exchange(10, 30);
                assert_eq!(prev, 20);
                assert_eq!(atomic.load(), 20);
            }

            #[test]
            fn test_get_and_increment() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_inc();
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 11);
            }

            #[test]
            fn test_fetch_inc_and_get() {
                let atomic = <$atomic_type>::new(10);
                let new = atomic.fetch_inc_and_get();
                assert_eq!(new, 11);
                assert_eq!(atomic.load(), 11);
            }

            #[test]
            fn test_get_and_decrement() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_dec();
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 9);
            }

            #[test]
            fn test_fetch_dec_and_get() {
                let atomic = <$atomic_type>::new(10);
                let new = atomic.fetch_dec_and_get();
                assert_eq!(new, 9);
                assert_eq!(atomic.load(), 9);
            }

            #[test]
            fn test_get_and_add() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_add(5);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 15);
            }

            #[test]
            fn test_fetch_add_and_get() {
                let atomic = <$atomic_type>::new(10);
                let new = atomic.fetch_add_and_get(5);
                assert_eq!(new, 15);
                assert_eq!(atomic.load(), 15);
            }

            #[test]
            fn test_get_and_sub() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_sub(3);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 7);
            }

            #[test]
            fn test_fetch_sub_and_get() {
                let atomic = <$atomic_type>::new(10);
                let new = atomic.fetch_sub_and_get(3);
                assert_eq!(new, 7);
                assert_eq!(atomic.load(), 7);
            }

            #[test]
            fn test_fetch_mul() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_mul(3);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 30);
            }

            #[test]
            fn test_fetch_mul_zero() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_mul(0);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 0);
            }

            #[test]
            fn test_fetch_mul_one() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_mul(1);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_fetch_div() {
                let atomic = <$atomic_type>::new(30);
                let old = atomic.fetch_div(3);
                assert_eq!(old, 30);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_fetch_div_one() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_div(1);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            #[should_panic(expected = "division by zero")]
            fn test_fetch_div_by_zero() {
                let atomic = <$atomic_type>::new(10);
                atomic.fetch_div(0);
            }

            #[test]
            fn test_get_and_bit_and() {
                let atomic = <$atomic_type>::new(0b1111);
                let old = atomic.fetch_and(0b1100);
                assert_eq!(old, 0b1111);
                assert_eq!(atomic.load(), 0b1100);
            }

            #[test]
            fn test_get_and_bit_or() {
                let atomic = <$atomic_type>::new(0b1100);
                let old = atomic.fetch_or(0b0011);
                assert_eq!(old, 0b1100);
                assert_eq!(atomic.load(), 0b1111);
            }

            #[test]
            fn test_get_and_bit_xor() {
                let atomic = <$atomic_type>::new(0b1100);
                let old = atomic.fetch_xor(0b0110);
                assert_eq!(old, 0b1100);
                assert_eq!(atomic.load(), 0b1010);
            }

            #[test]
            fn test_get_and_bit_not() {
                let value: $value_type = 42;
                let atomic = <$atomic_type>::new(value);
                let old = atomic.fetch_not();
                assert_eq!(old, value);
                assert_eq!(atomic.load(), !value);
            }

            #[test]
            fn test_bit_not_twice() {
                let value: $value_type = 42;
                let atomic = <$atomic_type>::new(value);
                atomic.fetch_not();
                atomic.fetch_not();
                assert_eq!(atomic.load(), value);
            }

            #[test]
            fn test_bit_not_and_get() {
                let value: $value_type = 42;
                let atomic = <$atomic_type>::new(value);
                let new = atomic.bit_not_and_get();
                assert_eq!(new, !value);
                assert_eq!(atomic.load(), !value);
            }

            #[test]
            fn test_get_and_update() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_update(|x| x * 2);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 20);
            }

            #[test]
            fn test_get_and_accumulate() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_accumulate(5, |a, b| a + b);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 15);
            }

            #[test]
            fn test_get_and_max() {
                let atomic = <$atomic_type>::new(10);
                atomic.fetch_max(20);
                assert_eq!(atomic.load(), 20);
                atomic.fetch_max(15);
                assert_eq!(atomic.load(), 20);
            }

            #[test]
            fn test_fetch_max_and_get() {
                let atomic = <$atomic_type>::new(10);
                let new = atomic.fetch_max_and_get(20);
                assert_eq!(new, 20);
                assert_eq!(atomic.load(), 20);

                let new = atomic.fetch_max_and_get(15);
                assert_eq!(new, 20);
                assert_eq!(atomic.load(), 20);
            }

            #[test]
            fn test_get_and_min() {
                let atomic = <$atomic_type>::new(10);
                atomic.fetch_min(5);
                assert_eq!(atomic.load(), 5);
                atomic.fetch_min(8);
                assert_eq!(atomic.load(), 5);
            }

            #[test]
            fn test_fetch_min_and_get() {
                let atomic = <$atomic_type>::new(10);
                let new = atomic.fetch_min_and_get(5);
                assert_eq!(new, 5);
                assert_eq!(atomic.load(), 5);

                let new = atomic.fetch_min_and_get(8);
                assert_eq!(new, 5);
                assert_eq!(atomic.load(), 5);
            }

            #[test]
            fn test_concurrent_increment() {
                let counter = Arc::new(<$atomic_type>::new(0));
                let mut handles = vec![];

                for _ in 0..10 {
                    let counter = counter.clone();
                    let handle = thread::spawn(move || {
                        for _ in 0..10 {
                            counter.fetch_inc();
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                assert_eq!(counter.load(), 100 as $value_type);
            }

            #[test]
            fn test_concurrent_get_and_increment() {
                let counter = Arc::new(<$atomic_type>::new(0));
                let mut handles = vec![];

                for _ in 0..10 {
                    let counter = counter.clone();
                    let handle = thread::spawn(move || {
                        for _ in 0..10 {
                            counter.fetch_inc();
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                assert_eq!(counter.load(), 100 as $value_type);
            }

            #[test]
            fn test_concurrent_get_and_decrement() {
                let counter = Arc::new(<$atomic_type>::new(100 as $value_type));
                let mut handles = vec![];

                for _ in 0..10 {
                    let counter = counter.clone();
                    let handle = thread::spawn(move || {
                        for _ in 0..10 {
                            counter.fetch_dec();
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                assert_eq!(counter.load(), 0);
            }

            #[test]
            fn test_concurrent_get_and_add() {
                let counter = Arc::new(<$atomic_type>::new(0));
                let mut handles = vec![];

                for _ in 0..10 {
                    let counter = counter.clone();
                    let handle = thread::spawn(move || {
                        for _ in 0..10 {
                            counter.fetch_add(1);
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                assert_eq!(counter.load(), 100 as $value_type);
            }

            #[test]
            fn test_concurrent_get_and_accumulate_high_contention() {
                let counter = Arc::new(<$atomic_type>::new(0));
                let mut handles = vec![];

                // High contention: 10 threads, each doing 10 accumulations
                for _ in 0..10 {
                    let counter = counter.clone();
                    let handle = thread::spawn(move || {
                        for _ in 0..10 {
                            counter.fetch_accumulate(1, |a, b| a.wrapping_add(b));
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                assert_eq!(counter.load(), 100 as $value_type);
            }

            #[test]
            fn test_concurrent_get_and_update_high_contention() {
                let counter = Arc::new(<$atomic_type>::new(0));
                let mut handles = vec![];

                // Very high contention: 20 threads, each doing 5 updates
                for _ in 0..20 {
                    let counter = counter.clone();
                    let handle = thread::spawn(move || {
                        for _ in 0..5 {
                            counter.fetch_update(|x| x.wrapping_add(1));
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                assert_eq!(counter.load(), 100 as $value_type);
            }

            #[test]
            fn test_concurrent_cas() {
                let atomic = Arc::new(<$atomic_type>::new(0));
                let mut handles = vec![];

                for i in 0..10 {
                    let atomic = atomic.clone();
                    let handle = thread::spawn(move || {
                        let mut current = atomic.load();
                        loop {
                            match atomic.compare_set_weak(current, current + 1) {
                                Ok(_) => return i,
                                Err(actual) => current = actual,
                            }
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_concurrent_fetch_mul() {
                let atomic = Arc::new(<$atomic_type>::new(1));
                let mut handles = vec![];

                for _ in 0..5 {
                    let atomic = atomic.clone();
                    let handle = thread::spawn(move || {
                        atomic.fetch_mul(2);
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                // Result should be 2^5 = 32
                assert_eq!(atomic.load(), 32);
            }

            #[test]
            fn test_concurrent_fetch_div() {
                // Use 32 as starting value to avoid overflow on i8/u8
                // 32 / 2 / 2 / 2 / 2 / 2 = 1
                let atomic = Arc::new(<$atomic_type>::new(32));
                let mut handles = vec![];

                for _ in 0..5 {
                    let atomic = atomic.clone();
                    let handle = thread::spawn(move || {
                        atomic.fetch_div(2);
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                // Result should be 32 / 2^5 = 1
                assert_eq!(atomic.load(), 1);
            }

            #[test]
            fn test_concurrent_compare_exchange_weak() {
                let atomic = Arc::new(<$atomic_type>::new(0));
                let mut handles = vec![];

                for _ in 0..10 {
                    let atomic = atomic.clone();
                    let handle = thread::spawn(move || {
                        let mut current = atomic.load();
                        loop {
                            let prev = atomic.compare_and_exchange_weak(current, current + 1);
                            if prev == current {
                                break;
                            }
                            current = prev;
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_trait_atomic() {
                fn test_atomic<T: Atomic<Value = $value_type>>(atomic: &T) {
                    atomic.store(42);
                    assert_eq!(atomic.load(), 42);
                    let old = atomic.swap(100);
                    assert_eq!(old, 42);
                }

                let atomic = <$atomic_type>::new(0);
                test_atomic(&atomic);
            }

            #[test]
            fn test_trait_fetch_update() {
                let atomic = <$atomic_type>::new(0);
                let old = atomic.fetch_update(|x| x + 10);
                assert_eq!(old, 0);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_trait_atomic_number() {
                fn test_number<T: AtomicNumber<Value = $value_type>>(atomic: &T) {
                    // Note: fetch_inc/dec are not part of AtomicNumber trait
                    // They are integer-specific convenience methods
                    atomic.fetch_add(1);
                    atomic.fetch_add(5);
                    assert_eq!(atomic.load(), 6);
                }

                let atomic = <$atomic_type>::new(0);
                test_number(&atomic);
            }

            #[test]
            fn test_trait_atomic_compare_set_weak() {
                fn test_atomic<T: Atomic<Value = $value_type>>(atomic: &T) {
                    atomic.store(10);
                    assert!(atomic.compare_set_weak(10, 20).is_ok());
                    assert_eq!(atomic.load(), 20);
                }

                let atomic = <$atomic_type>::new(0);
                test_atomic(&atomic);
            }

            #[test]
            fn test_trait_atomic_compare_exchange_weak() {
                fn test_atomic<T: Atomic<Value = $value_type>>(atomic: &T) {
                    atomic.store(10);
                    let prev = atomic.compare_exchange_weak(10, 20);
                    assert_eq!(prev, 10);
                    assert_eq!(atomic.load(), 20);
                }

                let atomic = <$atomic_type>::new(0);
                test_atomic(&atomic);
            }

            #[test]
            fn test_trait_atomic_integer_fetch_sub() {
                fn test_integer<T: AtomicNumber<Value = $value_type>>(atomic: &T) {
                    atomic.store(100);
                    let old = atomic.fetch_sub(30);
                    assert_eq!(old, 100);
                    assert_eq!(atomic.load(), 70);
                }

                let atomic = <$atomic_type>::new(0);
                test_integer(&atomic);
            }

            #[test]
            fn test_trait_atomic_integer_fetch_mul() {
                fn test_integer<T: AtomicNumber<Value = $value_type>>(atomic: &T) {
                    atomic.store(5);
                    let old = atomic.fetch_mul(3);
                    assert_eq!(old, 5);
                    assert_eq!(atomic.load(), 15);
                }

                let atomic = <$atomic_type>::new(0);
                test_integer(&atomic);
            }

            #[test]
            fn test_trait_atomic_integer_fetch_div() {
                fn test_integer<T: AtomicNumber<Value = $value_type>>(atomic: &T) {
                    atomic.store(20);
                    let old = atomic.fetch_div(4);
                    assert_eq!(old, 20);
                    assert_eq!(atomic.load(), 5);
                }

                let atomic = <$atomic_type>::new(0);
                test_integer(&atomic);
            }

            #[test]
            fn test_debug_display() {
                let atomic = <$atomic_type>::new(42);
                let debug_str = format!("{:?}", atomic);
                assert!(debug_str.contains("42"));
                let display_str = format!("{}", atomic);
                assert_eq!(display_str, "42");
            }

            #[test]
            fn test_inner_operations() {
                use std::sync::atomic::Ordering;

                let atomic = <$atomic_type>::new(10);
                atomic.inner().store(100, Ordering::Relaxed);
                assert_eq!(atomic.inner().load(Ordering::Relaxed), 100);

                let old = atomic.inner().fetch_add(5, Ordering::Relaxed);
                assert_eq!(old, 100);
                assert_eq!(atomic.load(), 105);
            }

            #[test]
            fn test_compare_and_set_weak_success() {
                let atomic = <$atomic_type>::new(10);
                assert!(atomic.compare_set_weak(10, 20).is_ok());
                assert_eq!(atomic.load(), 20);
            }

            #[test]
            fn test_compare_and_exchange_weak_success() {
                let atomic = <$atomic_type>::new(10);
                let prev = atomic.compare_and_exchange_weak(10, 20);
                assert_eq!(prev, 10);
                assert_eq!(atomic.load(), 20);
            }

            #[test]
            fn test_get_and_accumulate_multiply() {
                let atomic = <$atomic_type>::new(2);
                let old = atomic.fetch_accumulate(3, |a, b| a.wrapping_mul(b));
                assert_eq!(old, 2);
                assert_eq!(atomic.load(), 6);
            }

            #[test]
            fn test_max_with_smaller_value() {
                let atomic = <$atomic_type>::new(50);
                atomic.fetch_max(30);
                assert_eq!(atomic.load(), 50);
            }

            #[test]
            fn test_min_with_larger_value() {
                let atomic = <$atomic_type>::new(50);
                atomic.fetch_min(80);
                assert_eq!(atomic.load(), 50);
            }

            #[test]
            fn test_bitwise_combinations() {
                let atomic = <$atomic_type>::new(0b1111);
                atomic.fetch_and(0b1100);
                assert_eq!(atomic.load(), 0b1100);

                atomic.fetch_or(0b0011);
                assert_eq!(atomic.load(), 0b1111);

                atomic.fetch_xor(0b0101);
                assert_eq!(atomic.load(), 0b1010);
            }

            #[test]
            fn test_wrapping_decrement_from_one() {
                let atomic = <$atomic_type>::new(1);
                atomic.fetch_dec();
                assert_eq!(atomic.load(), 0);

                // Test wrapping add
                atomic.fetch_add(1);
                assert_eq!(atomic.load(), 1);
            }

            #[test]
            fn test_get_and_update_complex() {
                let atomic = <$atomic_type>::new(10);
                let old = atomic.fetch_update(|x| x.wrapping_mul(2).wrapping_add(5));
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 25);
            }

            #[test]
            fn test_compare_and_set_failure_path() {
                let atomic = <$atomic_type>::new(10);
                match atomic.compare_set(5, 15) {
                    Ok(_) => panic!("Should have failed"),
                    Err(actual) => assert_eq!(actual, 10),
                }
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_compare_and_exchange_failure_path() {
                let atomic = <$atomic_type>::new(10);
                let prev = atomic.compare_exchange(5, 15);
                assert_eq!(prev, 10);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_compare_and_set_weak_failure_path() {
                let atomic = <$atomic_type>::new(10);
                match atomic.compare_set_weak(5, 15) {
                    Ok(_) => panic!("Should have failed"),
                    Err(actual) => assert_eq!(actual, 10),
                }
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_compare_and_exchange_weak_failure_path() {
                let atomic = <$atomic_type>::new(10);
                let prev = atomic.compare_and_exchange_weak(5, 15);
                assert_eq!(prev, 10);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_compare_set_weak_multiple_attempts() {
                let atomic = <$atomic_type>::new(0);
                for i in 0..100 {
                    let mut current = atomic.load();
                    loop {
                        match atomic.compare_set_weak(current, i) {
                            Ok(_) => break,
                            Err(actual) => current = actual,
                        }
                    }
                }
                assert_eq!(atomic.load(), 99);
            }

            #[test]
            fn test_compare_exchange_weak_multiple_attempts() {
                let atomic = <$atomic_type>::new(0);
                for i in 0..100 {
                    let mut current = atomic.load();
                    loop {
                        let prev = atomic.compare_and_exchange_weak(current, i);
                        if prev == current {
                            break;
                        }
                        current = prev;
                    }
                }
                assert_eq!(atomic.load(), 99);
            }

            #[test]
            fn test_compare_set_weak_with_zero() {
                let atomic = <$atomic_type>::new(0);
                assert!(atomic.compare_set_weak(0, 42).is_ok());
                assert_eq!(atomic.load(), 42);
            }

            #[test]
            fn test_compare_exchange_weak_with_zero() {
                let atomic = <$atomic_type>::new(0);
                let prev = atomic.compare_and_exchange_weak(0, 42);
                assert_eq!(prev, 0);
                assert_eq!(atomic.load(), 42);
            }

            #[test]
            fn test_inner_compare_exchange_failure() {
                use std::sync::atomic::Ordering;

                let atomic = <$atomic_type>::new(10);
                let result =
                    atomic
                        .inner()
                        .compare_exchange(5, 15, Ordering::AcqRel, Ordering::Acquire);
                assert!(result.is_err());
                assert_eq!(result.unwrap_err(), 10);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_inner_compare_exchange_weak_failure() {
                use std::sync::atomic::Ordering;

                let atomic = <$atomic_type>::new(10);
                let result = atomic.inner().compare_exchange_weak(
                    5,
                    15,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
                assert!(result.is_err());
                assert_eq!(result.unwrap_err(), 10);
                assert_eq!(atomic.load(), 10);
            }

            #[test]
            fn test_inner_fetch_operations_all() {
                use std::sync::atomic::Ordering;

                let atomic = <$atomic_type>::new(10);

                // Test fetch_sub
                let old = atomic.inner().fetch_sub(3, Ordering::Relaxed);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 7);

                // Test fetch_and
                atomic.store(0b1111);
                let old = atomic.inner().fetch_and(0b1100, Ordering::Relaxed);
                assert_eq!(old, 0b1111);
                assert_eq!(atomic.load(), 0b1100);

                // Test fetch_or
                let old = atomic.inner().fetch_or(0b0011, Ordering::Relaxed);
                assert_eq!(old, 0b1100);
                assert_eq!(atomic.load(), 0b1111);

                // Test fetch_xor
                let old = atomic.inner().fetch_xor(0b0101, Ordering::Relaxed);
                assert_eq!(old, 0b1111);
                assert_eq!(atomic.load(), 0b1010);
            }

            #[test]
            fn test_inner_fetch_max_min() {
                use std::sync::atomic::Ordering;

                let atomic = <$atomic_type>::new(10);

                // Test fetch_max with larger value
                let old = atomic.inner().fetch_max(20, Ordering::Relaxed);
                assert_eq!(old, 10);
                assert_eq!(atomic.load(), 20);

                // Test fetch_max with smaller value
                let old = atomic.inner().fetch_max(15, Ordering::Relaxed);
                assert_eq!(old, 20);
                assert_eq!(atomic.load(), 20);

                // Test fetch_min with smaller value
                let old = atomic.inner().fetch_min(5, Ordering::Relaxed);
                assert_eq!(old, 20);
                assert_eq!(atomic.load(), 5);

                // Test fetch_min with larger value
                let old = atomic.inner().fetch_min(10, Ordering::Relaxed);
                assert_eq!(old, 5);
                assert_eq!(atomic.load(), 5);
            }

            #[test]
            fn test_get_and_accumulate_divide() {
                let atomic = <$atomic_type>::new(20);
                let old = atomic.fetch_accumulate(4, |a, b| if b != 0 { a / b } else { a });
                assert_eq!(old, 20);
                assert_eq!(atomic.load(), 5);
            }

            #[test]
            fn test_compare_and_exchange_success_path() {
                let atomic = <$atomic_type>::new(10);
                let prev = atomic.compare_exchange(10, 15);
                assert_eq!(prev, 10);
                assert_eq!(atomic.load(), 15);
            }

            #[test]
            fn test_compare_and_exchange_weak_success_path() {
                let atomic = <$atomic_type>::new(10);
                let prev = atomic.compare_and_exchange_weak(10, 15);
                assert_eq!(prev, 10);
                assert_eq!(atomic.load(), 15);
            }

            #[test]
            fn test_inner_compare_exchange_success() {
                use std::sync::atomic::Ordering;

                let atomic = <$atomic_type>::new(10);
                let result =
                    atomic
                        .inner()
                        .compare_exchange(10, 15, Ordering::AcqRel, Ordering::Acquire);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), 10);
                assert_eq!(atomic.load(), 15);
            }

            #[test]
            fn test_inner_compare_exchange_weak_success() {
                use std::sync::atomic::Ordering;

                let atomic = <$atomic_type>::new(10);
                let result = atomic.inner().compare_exchange_weak(
                    10,
                    15,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), 10);
                assert_eq!(atomic.load(), 15);
            }
        }
    };
}
