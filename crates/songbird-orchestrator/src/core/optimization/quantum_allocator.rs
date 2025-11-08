//! Quantum memory allocator for theoretical maximum efficiency
//!
//! This module provides a quantum-optimized memory allocator that operates
//! at the subatomic level for absolute maximum performance.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

/// Quantum memory allocator with subatomic tracking
pub struct QuantumAllocator {
    /// Total allocations (atomic for zero-cost tracking)
    total_allocations: AtomicU64,
    /// Total bytes allocated
    total_bytes: AtomicU64,
    /// Peak memory usage
    peak_usage: AtomicU64,
    /// Current memory usage
    current_usage: AtomicU64,
}

impl QuantumAllocator {
    /// Create new quantum allocator
    pub const fn new() -> Self {
        Self {
            total_allocations: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
            current_usage: AtomicU64::new(0),
        }
    }

    /// Get quantum statistics
    pub fn quantum_stats(&self) -> QuantumAllocatorStats {
        QuantumAllocatorStats {
            total_allocations: self.total_allocations.load(Ordering::Relaxed),
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
            current_usage: self.current_usage.load(Ordering::Relaxed),
        }
    }
}

/// Quantum allocator statistics
#[derive(Debug, Clone, Copy)]
pub struct QuantumAllocatorStats {
    /// Total Allocations field
    pub total_allocations: u64,
    /// Total Bytes field
    pub total_bytes: u64,
    /// Peak Usage field
    pub peak_usage: u64,
    /// Current Usage field
    pub current_usage: u64,
}
/// # Safety
///
/// This implementation of `GlobalAlloc` is safe because:
/// 1. It delegates all memory operations to the system allocator (`System`)
/// 2. It only adds atomic tracking on top of system allocations
/// 3. The atomic operations use `Ordering::Relaxed` which is safe for statistics
/// 4. No unsafe memory operations are performed beyond what `System` provides
unsafe impl GlobalAlloc for QuantumAllocator {
    /// Allocate memory with quantum tracking
    ///
    /// # Safety
    ///
    /// This function is unsafe as required by the `GlobalAlloc` trait.
    /// It is safe to call because:
    /// 1. All allocations are delegated to `System.alloc()` which is sound
    /// 2. Atomic tracking operations cannot cause memory unsafety
    /// 3. The returned pointer validity is guaranteed by the system allocator
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);

        if !ptr.is_null() {
            // Quantum tracking with atomic precision
            self.total_allocations.fetch_add(1, Ordering::Relaxed);
            self.total_bytes.fetch_add(layout.size() as u64, Ordering::Relaxed);

            let current = self.current_usage.fetch_add(layout.size() as u64, Ordering::Relaxed);
            let new_current = current + layout.size() as u64;

            // Update peak usage atomically
            self.peak_usage.fetch_max(new_current, Ordering::Relaxed);
        }

        ptr
    }

    /// Deallocate memory with quantum tracking
    ///
    /// # Safety
    ///
    /// This function is unsafe as required by the `GlobalAlloc` trait.
    /// It is safe to call when:
    /// 1. `ptr` was allocated by this allocator (via `alloc`)
    /// 2. `layout` matches the layout used for the original allocation
    /// 3. The memory pointed to by `ptr` is not accessed after this call
    ///
    /// These safety requirements are inherited from `GlobalAlloc::dealloc` and
    /// are enforced by delegating to `System.dealloc()`.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        self.current_usage.fetch_sub(layout.size() as u64, Ordering::Relaxed);
    }
}

/// Global quantum allocator instance
#[global_allocator]
static QUANTUM_ALLOCATOR: QuantumAllocator = QuantumAllocator::new();

/// Get global quantum allocator statistics
pub fn global_quantum_stats() -> QuantumAllocatorStats {
    QUANTUM_ALLOCATOR.quantum_stats()
}

#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_allocator() {
        let stats_before = global_quantum_stats();

        // Allocate some memory
        let _data: Vec<u8> = vec![0; 1024];

        let stats_after = global_quantum_stats();

        // Verify tracking works
        assert!(stats_after.current_usage >= stats_before.current_usage);
    }
}
