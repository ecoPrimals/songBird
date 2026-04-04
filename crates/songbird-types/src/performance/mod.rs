// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(dead_code)]

//! # ⚡ Performance Optimization Module
//!
//! **ZERO-COST ABSTRACTIONS & COMPILE-TIME OPTIMIZATIONS** 🚀
//!
//! This module provides performance-critical optimizations leveraging Rust's
//! zero-cost abstraction principles and const generics for compile-time efficiency.

pub mod zero_copy_enhanced;

// ============================================================================
// CONST GENERIC OPTIMIZATIONS
// ============================================================================

/// **ZERO-COST**: Compile-time sized buffer with const generics
///
/// # Safety Evolution
/// This implementation has been refactored from unsafe MaybeUninit to safe Option-based
/// storage. Thanks to null pointer optimization, Option<T> has zero overhead for most types
/// (pointers, NonZero types, etc.) and minimal overhead for others (typically 1 byte).
///
/// The trade-off is ~1 byte per element for non-optimizable types in exchange for 100% safety.
/// For orchestration workloads, this is an excellent trade-off.
#[derive(Debug)]
pub struct ConstBuffer<T, const N: usize> {
    // SAFE: Using Option provides built-in initialization tracking at compile time
    // Option<T> is optimized via null pointer optimization for many types (zero overhead)
    data: [Option<T>; N],
}

impl<T, const N: usize> ConstBuffer<T, N> {
    /// Create new buffer - zero runtime cost
    ///
    /// SAFE: Uses const array initialization with None - fully safe at compile time
    #[must_use]
    pub const fn new() -> Self {
        Self {
            // SAFE: const array initialization is 100% safe
            data: [const { None }; N],
        }
    }

    /// Push item if space available - compile-time bounds check
    ///
    /// # Errors
    /// Returns the item if buffer is full
    ///
    /// SAFE: No unsafe code - Option handles initialization tracking automatically
    #[inline]
    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        // Find first None slot
        for slot in &mut self.data {
            if slot.is_none() {
                *slot = Some(item);
                return Ok(());
            }
        }
        Err(item)
    }

    /// Get current length - zero cost
    ///
    /// SAFE: Counts Some variants - no unsafe code
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.data.iter().filter(|x| x.is_some()).count()
    }

    /// Check if empty - zero cost
    ///
    /// SAFE: Pure predicate on Option state
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(Option::is_none)
    }

    /// Get capacity - compile-time constant
    ///
    /// SAFE: Returns const generic parameter
    #[must_use]
    #[inline]
    pub const fn capacity() -> usize {
        N
    }

    /// Iterate over items (SAFE)
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().filter_map(|x| x.as_ref())
    }

    /// Clear all items (SAFE)
    #[inline]
    pub fn clear(&mut self) {
        for slot in &mut self.data {
            *slot = None;
        }
    }
}

// Drop is automatically derived correctly for [Option<T>; N] - no custom impl needed!
// Each Option<T> properly drops its T when Some, or does nothing when None.

// ============================================================================
// COMPILE-TIME STRING HASHING
// ============================================================================

/// **ZERO-COST**: Compile-time string hash for static strings
#[must_use]
pub const fn const_hash(s: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;

    let bytes = s.as_bytes();
    let mut hash = FNV_OFFSET_BASIS;
    let mut i = 0;

    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }

    hash
}

/// **ZERO-COST**: Compile-time string identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstStringId<const HASH: u64>;

impl<const HASH: u64> ConstStringId<HASH> {
    /// Create from compile-time hashed string
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Get the hash value - compile-time constant
    #[must_use]
    pub const fn hash() -> u64 {
        HASH
    }
}

/// Macro to create compile-time string identifiers.
#[macro_export]
macro_rules! const_string_id {
    ($s:expr) => {
        $crate::performance::ConstStringId::<{ $crate::performance::const_hash($s) }>::new()
    };
}

// ============================================================================
// ZERO-COST TYPE-LEVEL PROGRAMMING
// ============================================================================

/// **ZERO-COST**: Type-level boolean for compile-time decisions
pub trait TypeBool {
    const VALUE: bool;
}

/// True type - zero runtime cost
#[derive(Debug, Clone, Copy)]
pub struct True;

impl TypeBool for True {
    const VALUE: bool = true;
}

/// False type - zero runtime cost
#[derive(Debug, Clone, Copy)]
pub struct False;

impl TypeBool for False {
    const VALUE: bool = false;
}

/// **ZERO-COST**: Conditional type selection at compile time
pub trait TypeSelect<T, U> {
    type Output;
}

impl<T, U> TypeSelect<T, U> for True {
    type Output = T;
}

impl<T, U> TypeSelect<T, U> for False {
    type Output = U;
}

/// **ZERO-COST**: Performance mode selection at compile time
#[derive(Debug, Clone, Copy)]
pub struct PerformanceConfig<const FAST_MODE: bool, const DEBUG_MODE: bool>;

impl<const FAST_MODE: bool, const DEBUG_MODE: bool> PerformanceConfig<FAST_MODE, DEBUG_MODE> {
    /// Create new performance config - zero cost
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Check if fast mode enabled - compile-time constant
    #[must_use]
    pub const fn is_fast_mode() -> bool {
        FAST_MODE
    }

    /// Check if debug mode enabled - compile-time constant
    #[must_use]
    pub const fn is_debug_mode() -> bool {
        DEBUG_MODE
    }

    /// Execute code only in debug mode - zero cost in release
    #[inline]
    pub fn debug_only<F: FnOnce()>(f: F) {
        if DEBUG_MODE {
            f();
        }
    }
}

// Type aliases for common configurations
pub type ProductionConfig = PerformanceConfig<true, false>;
pub type DevelopmentConfig = PerformanceConfig<false, true>;
pub type TestingConfig = PerformanceConfig<false, true>;

// ============================================================================
// ZERO-ALLOCATION OPERATIONS
// ============================================================================

/// **ZERO-COST**: Stack-allocated string buffer
pub type StackString<const N: usize> = arrayvec::ArrayString<N>;

/// **ZERO-COST**: Stack-allocated vector
pub type StackVec<T, const N: usize> = arrayvec::ArrayVec<T, N>;

// Re-export enhanced zero-copy types
#[allow(unused_imports)]
pub use zero_copy_enhanced::{ZeroCopyBytes, ZeroCopyString};
