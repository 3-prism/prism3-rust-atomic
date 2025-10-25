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
    AtomicBool,
};
use std::sync::Arc;
use std::thread;

#[test]
fn test_new() {
    let atomic = AtomicBool::new(true);
    assert!(atomic.get());
    let atomic = AtomicBool::new(false);
    assert!(!atomic.get());
}

#[test]
fn test_default() {
    let atomic = AtomicBool::default();
    assert!(!atomic.get());
}

#[test]
fn test_from() {
    let atomic = AtomicBool::from(true);
    assert!(atomic.get());
}

#[test]
fn test_get_set() {
    let atomic = AtomicBool::new(false);
    atomic.set(true);
    assert!(atomic.get());
    atomic.set(false);
    assert!(!atomic.get());
}

#[test]
fn test_swap() {
    let atomic = AtomicBool::new(false);
    let old = atomic.swap(true);
    assert!(!old);
    assert!(atomic.get());
}

#[test]
fn test_compare_and_set_success() {
    let atomic = AtomicBool::new(false);
    assert!(atomic.compare_and_set(false, true).is_ok());
    assert!(atomic.get());
}

#[test]
fn test_compare_and_set_failure() {
    let atomic = AtomicBool::new(false);
    match atomic.compare_and_set(true, false) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => assert!(!actual),
    }
    assert!(!atomic.get());
}

#[test]
fn test_compare_and_exchange() {
    let atomic = AtomicBool::new(false);
    let prev = atomic.compare_and_exchange(false, true);
    assert!(!prev);
    assert!(atomic.get());

    let prev = atomic.compare_and_exchange(false, false);
    assert!(prev);
    assert!(atomic.get());
}

#[test]
fn test_get_and_set() {
    let atomic = AtomicBool::new(false);
    let old = atomic.get_and_set();
    assert!(!old);
    assert!(atomic.get());
}

#[test]
fn test_set_and_get() {
    let atomic = AtomicBool::new(false);
    let new = atomic.set_and_get();
    assert!(new);
    assert!(atomic.get());
}

#[test]
fn test_get_and_clear() {
    let atomic = AtomicBool::new(true);
    let old = atomic.get_and_clear();
    assert!(old);
    assert!(!atomic.get());
}

#[test]
fn test_clear_and_get() {
    let atomic = AtomicBool::new(true);
    let new = atomic.clear_and_get();
    assert!(!new);
    assert!(!atomic.get());
}

#[test]
fn test_get_and_negate() {
    let atomic = AtomicBool::new(false);
    assert!(!atomic.get_and_negate());
    assert!(atomic.get());
    assert!(atomic.get_and_negate());
    assert!(!atomic.get());
}

#[test]
fn test_negate_and_get() {
    let atomic = AtomicBool::new(false);
    assert!(atomic.negate_and_get());
    assert!(atomic.get());
}

#[test]
fn test_get_and_logical_and() {
    let atomic = AtomicBool::new(true);
    assert!(atomic.get_and_logical_and(false));
    assert!(!atomic.get());

    atomic.set(true);
    assert!(atomic.get_and_logical_and(true));
    assert!(atomic.get());
}

#[test]
fn test_get_and_logical_or() {
    let atomic = AtomicBool::new(false);
    assert!(!atomic.get_and_logical_or(true));
    assert!(atomic.get());

    atomic.set(false);
    assert!(!atomic.get_and_logical_or(false));
    assert!(!atomic.get());
}

#[test]
fn test_get_and_logical_xor() {
    let atomic = AtomicBool::new(false);
    assert!(!atomic.get_and_logical_xor(true));
    assert!(atomic.get());

    assert!(atomic.get_and_logical_xor(true));
    assert!(!atomic.get());
}

#[test]
fn test_compare_and_set_if_false() {
    let atomic = AtomicBool::new(false);
    assert!(atomic.compare_and_set_if_false(true).is_ok());
    assert!(atomic.get());

    assert!(atomic.compare_and_set_if_false(false).is_err());
    assert!(atomic.get());
}

#[test]
fn test_compare_and_set_if_true() {
    let atomic = AtomicBool::new(true);
    assert!(atomic.compare_and_set_if_true(false).is_ok());
    assert!(!atomic.get());

    assert!(atomic.compare_and_set_if_true(true).is_err());
    assert!(!atomic.get());
}

#[test]
fn test_concurrent_toggle() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];

    for _ in 0..10 {
        let flag = flag.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                flag.get_and_negate();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // After 1000 toggles, should be false (even number)
    assert!(!flag.get());
}

#[test]
fn test_concurrent_set_once() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];
    let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    for _ in 0..10 {
        let flag = flag.clone();
        let success_count = success_count.clone();
        let handle = thread::spawn(move || {
            if flag.compare_and_set_if_false(true).is_ok() {
                success_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Only one thread should succeed
    assert!(flag.get());
    assert_eq!(success_count.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[test]
fn test_trait_atomic() {
    fn test_atomic<T: Atomic<Value = bool>>(atomic: &T) {
        atomic.set(true);
        assert!(atomic.get());
        let old = atomic.swap(false);
        assert!(old);
    }

    let atomic = AtomicBool::new(false);
    test_atomic(&atomic);
}

#[test]
fn test_debug_display() {
    let atomic = AtomicBool::new(true);
    let debug_str = format!("{:?}", atomic);
    assert!(debug_str.contains("true"));
    let display_str = format!("{}", atomic);
    assert_eq!(display_str, "true");
}

#[test]
fn test_compare_and_set_weak_success() {
    let atomic = AtomicBool::new(false);
    assert!(atomic.compare_and_set_weak(false, true).is_ok());
    assert!(atomic.get());
}

#[test]
fn test_compare_and_set_weak_failure() {
    let atomic = AtomicBool::new(false);
    match atomic.compare_and_set_weak(true, false) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => assert!(!actual),
    }
    assert!(!atomic.get());
}

#[test]
fn test_compare_and_exchange_weak() {
    let atomic = AtomicBool::new(false);
    let prev = atomic.compare_and_exchange_weak(false, true);
    assert!(!prev);
    assert!(atomic.get());

    let prev = atomic.compare_and_exchange_weak(false, false);
    assert!(prev);
    assert!(atomic.get());
}

#[test]
fn test_inner() {
    use std::sync::atomic::Ordering;

    let atomic = AtomicBool::new(false);
    atomic.inner().store(true, Ordering::Relaxed);
    assert!(atomic.inner().load(Ordering::Relaxed));

    atomic.inner().store(false, Ordering::Release);
    assert!(!atomic.inner().load(Ordering::Acquire));
}

#[test]
fn test_get_and_set_already_true() {
    let atomic = AtomicBool::new(true);
    let old = atomic.get_and_set();
    assert!(old);
    assert!(atomic.get());
}

#[test]
fn test_set_and_get_already_true() {
    let atomic = AtomicBool::new(true);
    let new = atomic.set_and_get();
    assert!(new);
    assert!(atomic.get());
}

#[test]
fn test_get_and_clear_already_false() {
    let atomic = AtomicBool::new(false);
    let old = atomic.get_and_clear();
    assert!(!old);
    assert!(!atomic.get());
}

#[test]
fn test_clear_and_get_already_false() {
    let atomic = AtomicBool::new(false);
    let new = atomic.clear_and_get();
    assert!(!new);
    assert!(!atomic.get());
}

#[test]
fn test_negate_and_get_from_true() {
    let atomic = AtomicBool::new(true);
    assert!(!atomic.negate_and_get());
    assert!(!atomic.get());
}

#[test]
fn test_get_and_logical_and_both_false() {
    let atomic = AtomicBool::new(false);
    assert!(!atomic.get_and_logical_and(false));
    assert!(!atomic.get());
}

#[test]
fn test_get_and_logical_and_false_true() {
    let atomic = AtomicBool::new(false);
    assert!(!atomic.get_and_logical_and(true));
    assert!(!atomic.get());
}

#[test]
fn test_get_and_logical_or_both_true() {
    let atomic = AtomicBool::new(true);
    assert!(atomic.get_and_logical_or(true));
    assert!(atomic.get());
}

#[test]
fn test_get_and_logical_or_true_false() {
    let atomic = AtomicBool::new(true);
    assert!(atomic.get_and_logical_or(false));
    assert!(atomic.get());
}

#[test]
fn test_get_and_logical_xor_both_false() {
    let atomic = AtomicBool::new(false);
    assert!(!atomic.get_and_logical_xor(false));
    assert!(!atomic.get());
}

#[test]
fn test_get_and_logical_xor_both_true() {
    let atomic = AtomicBool::new(true);
    assert!(atomic.get_and_logical_xor(true));
    assert!(!atomic.get());
}

#[test]
fn test_get_and_logical_xor_true_false() {
    let atomic = AtomicBool::new(true);
    assert!(atomic.get_and_logical_xor(false));
    assert!(atomic.get());
}

#[test]
fn test_compare_and_set_if_false_already_true() {
    let atomic = AtomicBool::new(true);
    match atomic.compare_and_set_if_false(false) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => assert!(actual),
    }
    assert!(atomic.get());
}

#[test]
fn test_compare_and_set_if_true_already_false() {
    let atomic = AtomicBool::new(false);
    match atomic.compare_and_set_if_true(true) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => assert!(!actual),
    }
    assert!(!atomic.get());
}

#[test]
fn test_trait_atomic_compare_and_set() {
    fn test_atomic<T: Atomic<Value = bool>>(atomic: &T) {
        assert!(atomic.compare_and_set(false, true).is_ok());
        assert!(atomic.get());
        assert!(atomic.compare_and_set(false, false).is_err());
    }

    let atomic = AtomicBool::new(false);
    test_atomic(&atomic);
}

#[test]
fn test_trait_atomic_compare_and_exchange() {
    fn test_atomic<T: Atomic<Value = bool>>(atomic: &T) {
        let prev = atomic.compare_and_exchange(false, true);
        assert!(!prev);
        assert!(atomic.get());
    }

    let atomic = AtomicBool::new(false);
    test_atomic(&atomic);
}

#[test]
fn test_concurrent_compare_and_set_weak() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];
    let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    for _ in 0..10 {
        let flag = flag.clone();
        let success_count = success_count.clone();
        let handle = thread::spawn(move || {
            let mut current = flag.get();
            loop {
                match flag.compare_and_set_weak(current, true) {
                    Ok(_) => {
                        success_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        break;
                    }
                    Err(actual) => current = actual,
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert!(flag.get());
    // At least one thread should succeed
    assert!(success_count.load(std::sync::atomic::Ordering::Relaxed) >= 1);
}
