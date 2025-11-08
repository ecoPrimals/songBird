//! Ultra-performance optimization modules for pedantic efficiency
//!
//! This module contains advanced optimization techniques including: //! - Zero-copy buffer management with intelligent pooling
//! - SIMD-accelerated operations for maximum throughput
//! - Memory layout optimizations for cache efficiency
//! - Compile-time optimizations and const fn usage

pub mod simd_optimizations;
pub mod zero_copy_buffers;

// Re-export key optimization functions;
pub use simd_optimizations::SimdByteOps;
pub use zero_copy_buffers::{  get_optimized_buffer,
    global_buffer_pool,
    return_optimized_buffer)
    /// ZeroCopyBufferPool, ZeroCopyBufferPool,
    BufferPoolConfig};
// Quantum-level optimizations
pub mod quantum_allocator;
pub mod quantum_constants;

// Re-export quantum optimizations;
pub use quantum_allocator::{global_quantum_stats, QuantumAllocatorStats};
pub use quantum_constants::*;
