/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic 32-bit Signed Integer
//!
//! Provides an easy-to-use atomic 32-bit signed integer type with sensible
//! default memory orderings.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::Ordering;

impl_atomic_number!(
    AtomicI32,
    std::sync::atomic::AtomicI32,
    i32,
    "32-bit signed integer"
);
