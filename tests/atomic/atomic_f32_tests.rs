/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_atomic::atomic::{
    Atomic,
    AtomicF32,
    UpdatableAtomic,
};
use std::sync::Arc;
use std::thread;

const EPSILON: f32 = 1e-6;

#[test]
fn test_new() {
    let atomic = AtomicF32::new(std::f32::consts::PI);
    assert!((atomic.get() - std::f32::consts::PI).abs() < EPSILON);
}

#[test]
fn test_default() {
    let atomic = AtomicF32::default();
    assert_eq!(atomic.get(), 0.0);
}

#[test]
fn test_from() {
    let atomic = AtomicF32::from(2.71);
    assert!((atomic.get() - 2.71).abs() < EPSILON);
}

#[test]
fn test_get_set() {
    let atomic = AtomicF32::new(0.0);
    atomic.set(std::f32::consts::PI);
    assert!((atomic.get() - std::f32::consts::PI).abs() < EPSILON);
    atomic.set(-2.5);
    assert!((atomic.get() - (-2.5)).abs() < EPSILON);
}

#[test]
fn test_swap() {
    let atomic = AtomicF32::new(1.0);
    let old = atomic.swap(2.0);
    assert!((old - 1.0).abs() < EPSILON);
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_set_success() {
    let atomic = AtomicF32::new(1.0);
    assert!(atomic.compare_and_set(1.0, 2.0).is_ok());
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_set_failure() {
    let atomic = AtomicF32::new(1.0);
    match atomic.compare_and_set(1.5, 2.0) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => assert!((actual - 1.0).abs() < EPSILON),
    }
    assert!((atomic.get() - 1.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_exchange() {
    let atomic = AtomicF32::new(1.0);
    let prev = atomic.compare_and_exchange(1.0, 2.0);
    assert!((prev - 1.0).abs() < EPSILON);
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_add() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.add(5.5);
    assert!((new - 15.5).abs() < EPSILON);
    assert!((atomic.get() - 15.5).abs() < EPSILON);
}

#[test]
fn test_sub() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.sub(3.5);
    assert!((new - 6.5).abs() < EPSILON);
    assert!((atomic.get() - 6.5).abs() < EPSILON);
}

#[test]
fn test_mul() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.mul(2.5);
    assert!((new - 25.0).abs() < EPSILON);
    assert!((atomic.get() - 25.0).abs() < EPSILON);
}

#[test]
fn test_div() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.div(2.0);
    assert!((new - 5.0).abs() < EPSILON);
    assert!((atomic.get() - 5.0).abs() < EPSILON);
}

#[test]
fn test_get_and_update() {
    let atomic = AtomicF32::new(10.0);
    let old = atomic.get_and_update(|x| x * 2.0);
    assert!((old - 10.0).abs() < EPSILON);
    assert!((atomic.get() - 20.0).abs() < EPSILON);
}

#[test]
fn test_update_and_get() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.update_and_get(|x| x * 2.0);
    assert!((new - 20.0).abs() < EPSILON);
    assert!((atomic.get() - 20.0).abs() < EPSILON);
}

#[test]
fn test_concurrent_add() {
    let sum = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

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

    // Due to floating point precision, result may not be exactly 10.0
    let result = sum.get();
    assert!((result - 10.0).abs() < 0.01);
}

#[test]
fn test_trait_atomic() {
    fn test_atomic<T: Atomic<Value = f32>>(atomic: &T) {
        atomic.set(std::f32::consts::PI);
        assert!((atomic.get() - std::f32::consts::PI).abs() < EPSILON);
        let old = atomic.swap(2.71);
        assert!((old - std::f32::consts::PI).abs() < EPSILON);
    }

    let atomic = AtomicF32::new(0.0);
    test_atomic(&atomic);
}

#[test]
fn test_trait_updatable_atomic() {
    fn test_updatable<T: UpdatableAtomic<Value = f32>>(atomic: &T) {
        let new = atomic.update_and_get(|x| x + 10.0);
        assert!((new - 10.0).abs() < EPSILON);
    }

    let atomic = AtomicF32::new(0.0);
    test_updatable(&atomic);
}

#[test]
fn test_debug_display() {
    let atomic = AtomicF32::new(std::f32::consts::PI);
    let debug_str = format!("{:?}", atomic);
    assert!(debug_str.contains("3.14"));
    let display_str = format!("{}", atomic);
    assert!(display_str.contains("3.14"));
}

#[test]
fn test_negative_values() {
    let atomic = AtomicF32::new(-10.5);
    assert!((atomic.get() - (-10.5)).abs() < EPSILON);
    atomic.add(5.5);
    assert!((atomic.get() - (-5.0)).abs() < EPSILON);
}

#[test]
fn test_zero() {
    let atomic = AtomicF32::new(0.0);
    assert_eq!(atomic.get(), 0.0);
    atomic.add(1.0);
    assert!((atomic.get() - 1.0).abs() < EPSILON);
}

#[test]
fn test_infinity() {
    let atomic = AtomicF32::new(f32::INFINITY);
    assert_eq!(atomic.get(), f32::INFINITY);
    atomic.set(f32::NEG_INFINITY);
    assert_eq!(atomic.get(), f32::NEG_INFINITY);
}

#[test]
fn test_compare_and_set_weak() {
    let atomic = AtomicF32::new(1.0);
    assert!(atomic.compare_and_set_weak(1.0, 2.0).is_ok());
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_exchange_weak() {
    let atomic = AtomicF32::new(1.0);
    let prev = atomic.compare_and_exchange_weak(1.0, 2.0);
    assert!((prev - 1.0).abs() < EPSILON);
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_inner() {
    use std::sync::atomic::Ordering;

    let atomic = AtomicF32::new(1.0);
    let bits = atomic.inner().load(Ordering::Relaxed);
    assert_eq!(f32::from_bits(bits), 1.0);

    atomic.inner().store(2.0f32.to_bits(), Ordering::Release);
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_inner_cas() {
    use std::sync::atomic::Ordering;

    let atomic = AtomicF32::new(1.0);
    let current_bits = atomic.inner().load(Ordering::Relaxed);
    let new_bits = 2.0f32.to_bits();

    atomic
        .inner()
        .compare_exchange(current_bits, new_bits, Ordering::AcqRel, Ordering::Acquire)
        .unwrap();

    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_nan() {
    let atomic = AtomicF32::new(f32::NAN);
    assert!(atomic.get().is_nan());
}

#[test]
fn test_sub_negative() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.sub(-5.0);
    assert!((new - 15.0).abs() < EPSILON);
}

#[test]
fn test_mul_negative() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.mul(-2.0);
    assert!((new - (-20.0)).abs() < EPSILON);
}

#[test]
fn test_div_by_zero() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.div(0.0);
    assert!(new.is_infinite());
}

#[test]
fn test_compare_and_set_failure_returns_actual() {
    let atomic = AtomicF32::new(1.0);
    match atomic.compare_and_set(2.0, 3.0) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => assert!((actual - 1.0).abs() < EPSILON),
    }
}

#[test]
fn test_concurrent_mul() {
    let value = Arc::new(AtomicF32::new(1.0));
    let mut handles = vec![];

    for _ in 0..5 {
        let value = value.clone();
        let handle = thread::spawn(move || {
            value.mul(2.0);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be 2^5 = 32
    let result = value.get();
    assert!((result - 32.0).abs() < 0.01);
}

#[test]
fn test_concurrent_div() {
    let value = Arc::new(AtomicF32::new(1024.0));
    let mut handles = vec![];

    for _ in 0..5 {
        let value = value.clone();
        let handle = thread::spawn(move || {
            value.div(2.0);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be 1024 / 2^5 = 32
    let result = value.get();
    assert!((result - 32.0).abs() < 0.01);
}

#[test]
fn test_compare_and_set_weak_in_loop() {
    let atomic = AtomicF32::new(0.0);
    let mut current = atomic.get();
    for i in 0..10 {
        loop {
            match atomic.compare_and_set_weak(current, (i + 1) as f32) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
        current = (i + 1) as f32;
    }
    assert!((atomic.get() - 10.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_exchange_weak_in_loop() {
    let atomic = AtomicF32::new(0.0);
    let mut current = atomic.get();
    for i in 0..10 {
        loop {
            let prev = atomic.compare_and_exchange_weak(current, (i + 1) as f32);
            if (prev - current).abs() < EPSILON {
                break;
            }
            current = prev;
        }
        current = (i + 1) as f32;
    }
    assert!((atomic.get() - 10.0).abs() < EPSILON);
}

#[test]
fn test_add_zero() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.add(0.0);
    assert!((new - 10.0).abs() < EPSILON);
}

#[test]
fn test_sub_zero() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.sub(0.0);
    assert!((new - 10.0).abs() < EPSILON);
}

#[test]
fn test_mul_one() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.mul(1.0);
    assert!((new - 10.0).abs() < EPSILON);
}

#[test]
fn test_div_one() {
    let atomic = AtomicF32::new(10.0);
    let new = atomic.div(1.0);
    assert!((new - 10.0).abs() < EPSILON);
}

#[test]
fn test_display() {
    let atomic = AtomicF32::new(std::f32::consts::PI);
    let display_str = format!("{}", atomic);
    assert!(display_str.contains("3.14"));
}

#[test]
fn test_debug_false() {
    let atomic = AtomicF32::new(0.0);
    let debug_str = format!("{:?}", atomic);
    assert!(debug_str.contains("0"));
}

#[test]
fn test_trait_updatable_atomic_comprehensive() {
    fn test_updatable<T: UpdatableAtomic<Value = f32>>(atomic: &T) {
        let old = atomic.get_and_update(|x| x + 1.0);
        assert!((old - 0.0).abs() < EPSILON);
        assert!((atomic.get() - 1.0).abs() < EPSILON);

        let new = atomic.update_and_get(|x| x * 2.0);
        assert!((new - 2.0).abs() < EPSILON);
    }

    let atomic = AtomicF32::new(0.0);
    test_updatable(&atomic);
}

#[test]
fn test_trait_atomic_comprehensive() {
    fn test_atomic<T: Atomic<Value = f32>>(atomic: &T) {
        atomic.set(5.0);
        assert!((atomic.get() - 5.0).abs() < EPSILON);

        let old = atomic.swap(10.0);
        assert!((old - 5.0).abs() < EPSILON);

        assert!(atomic.compare_and_set(10.0, 15.0).is_ok());
        assert_eq!(atomic.compare_and_exchange(15.0, 20.0), 15.0);
    }

    let atomic = AtomicF32::new(0.0);
    test_atomic(&atomic);
}

#[test]
fn test_get_and_update_identity() {
    let atomic = AtomicF32::new(42.0);
    let old = atomic.get_and_update(|x| x);
    assert!((old - 42.0).abs() < EPSILON);
    assert!((atomic.get() - 42.0).abs() < EPSILON);
}

#[test]
fn test_update_and_get_identity() {
    let atomic = AtomicF32::new(42.0);
    let new = atomic.update_and_get(|x| x);
    assert!((new - 42.0).abs() < EPSILON);
    assert!((atomic.get() - 42.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_set_failure_path() {
    let atomic = AtomicF32::new(10.0);
    // Try to CAS with wrong current value
    match atomic.compare_and_set(5.0, 15.0) {
        Ok(_) => panic!("Should have failed"),
        Err(actual) => assert!((actual - 10.0).abs() < EPSILON),
    }
    assert!((atomic.get() - 10.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_exchange_failure_path() {
    let atomic = AtomicF32::new(10.0);
    let prev = atomic.compare_and_exchange(5.0, 15.0);
    assert!((prev - 10.0).abs() < EPSILON);
    assert!((atomic.get() - 10.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_set_weak_failure_path() {
    let atomic = AtomicF32::new(10.0);
    match atomic.compare_and_set_weak(5.0, 15.0) {
        Ok(_) => panic!("Should have failed"),
        Err(actual) => assert!((actual - 10.0).abs() < EPSILON),
    }
    assert!((atomic.get() - 10.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_exchange_weak_failure_path() {
    let atomic = AtomicF32::new(10.0);
    let prev = atomic.compare_and_exchange_weak(5.0, 15.0);
    assert!((prev - 10.0).abs() < EPSILON);
    assert!((atomic.get() - 10.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_exchange_success_path() {
    let atomic = AtomicF32::new(10.0);
    let prev = atomic.compare_and_exchange(10.0, 15.0);
    assert!((prev - 10.0).abs() < EPSILON);
    assert!((atomic.get() - 15.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_exchange_weak_success_path() {
    let atomic = AtomicF32::new(10.0);
    let prev = atomic.compare_and_exchange_weak(10.0, 15.0);
    assert!((prev - 10.0).abs() < EPSILON);
    assert!((atomic.get() - 15.0).abs() < EPSILON);
}

#[test]
fn test_concurrent_add_high_contention() {
    let atomic = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

    // High contention: many threads adding simultaneously
    for _ in 0..20 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            for _ in 0..50 {
                atomic.add(0.1);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be around 100.0 (20 * 50 * 0.1)
    let result = atomic.get();
    assert!((result - 100.0).abs() < 0.5);
}

#[test]
fn test_concurrent_sub_high_contention() {
    let atomic = Arc::new(AtomicF32::new(1000.0));
    let mut handles = vec![];

    // High contention: many threads subtracting simultaneously
    for _ in 0..20 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                atomic.sub(1.0);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be around 800.0 (1000 - 20 * 10)
    let result = atomic.get();
    assert!((result - 800.0).abs() < 1.0);
}

#[test]
fn test_concurrent_mul_and_div() {
    let atomic = Arc::new(AtomicF32::new(100.0));
    let mut handles = vec![];

    // Some threads multiply, some divide
    for i in 0..10 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            if i % 2 == 0 {
                atomic.mul(1.1);
            } else {
                atomic.div(1.1);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be close to original (5 muls and 5 divs)
    let result = atomic.get();
    assert!(result > 50.0 && result < 200.0);
}

#[test]
fn test_concurrent_mul_extreme_contention() {
    let atomic = Arc::new(AtomicF32::new(1.0));
    let mut handles = vec![];

    // Very high contention: 30 threads, each doing 20 multiplications
    for _ in 0..30 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            for _ in 0..20 {
                atomic.mul(1.001);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be greater than 1.0
    let result = atomic.get();
    assert!(result > 1.0);
}

#[test]
fn test_concurrent_div_extreme_contention() {
    let atomic = Arc::new(AtomicF32::new(1000000.0));
    let mut handles = vec![];

    // Very high contention: 30 threads, each doing 20 divisions
    for _ in 0..30 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            for _ in 0..20 {
                atomic.div(1.001);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be less than original
    let result = atomic.get();
    assert!(result < 1000000.0);
}

#[test]
fn test_concurrent_get_and_update_contention() {
    let atomic = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

    for _ in 0..10 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            for _ in 0..20 {
                atomic.get_and_update(|x| x + 0.5);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be 10 * 20 * 0.5 = 100.0
    let result = atomic.get();
    assert!((result - 100.0).abs() < 0.5);
}

#[test]
fn test_concurrent_update_and_get_contention() {
    let atomic = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

    for _ in 0..10 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            for _ in 0..20 {
                atomic.update_and_get(|x| x + 0.5);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Result should be 10 * 20 * 0.5 = 100.0
    let result = atomic.get();
    assert!((result - 100.0).abs() < 0.5);
}
