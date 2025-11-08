//! Compiler-optimized operations for pedantic performance
//!
//! This module provides high-performance operations by leveraging modern compiler
//! auto-vectorization instead of manual SIMD intrinsics. The Rust compiler (LLVM)
//! automatically generates SIMD instructions (AVX2, SSE2, NEON, etc.) when optimizations
//! are enabled, making this approach:
//!
//! 1. **100% Safe** - No unsafe code required
//! 2. **Portable** - Works on all architectures (x86, ARM, RISC-V, etc.)
//! 3. **Fast** - Compiler knows the target CPU better than hand-written SIMD
//! 4. **Maintainable** - Simple code that's easy to understand and verify
//!
//! ## Performance Notes
//!
//! When compiled with `-C opt-level=3` and `-C target-cpu=native`, the compiler will:
//! - Auto-vectorize `compare_bytes_safe()` to use AVX2/SSE2 on x86_64
//! - Auto-vectorize `clear_bytes_safe()` to use `memset` with SIMD
//! - Generate optimal code for the target CPU's capabilities
//!
//! To verify SIMD generation, compile with: `RUSTFLAGS="-C opt-level=3 -C target-cpu=native"` 
//! and inspect the assembly output.

/// Safe byte operations with compiler auto-vectorization
///
/// ## Safety Evolution
///
/// This replaces manual SIMD intrinsics with simple, safe code that the compiler
/// auto-vectorizes. Benchmarks show equivalent or better performance than hand-written
/// SIMD in most cases, with the added benefits of safety, portability, and maintainability.
pub struct SimdByteOps;

impl SimdByteOps {
    /// Ultra-fast byte comparison using compiler auto-vectorization
    ///
    /// The compiler will automatically use SIMD instructions when:
    /// - Optimizations are enabled (`-C opt-level=2` or higher)
    /// - Target CPU supports SIMD (detected via `target-cpu=native` or explicit target)
    ///
    /// On x86_64 with AVX2, this generates code equivalent to manual AVX2 intrinsics.
    /// On ARM with NEON, this generates NEON instructions automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_orchestrator::core::optimization::simd_optimizations::SimdByteOps;
    ///
    /// let a = b"hello world";
    /// let b = b"hello world";
    /// assert!(SimdByteOps::compare_bytes_safe(a, b));
    /// ```
    #[inline]
    #[must_use]
    pub fn compare_bytes_safe(a: &[u8], b: &[u8]) -> bool {
        // SAFE: Standard slice comparison
        // LLVM auto-vectorizes this to SIMD instructions when:
        // 1. Slices are long enough (typically > 16 bytes)
        // 2. Optimizations are enabled
        // 3. Target CPU has SIMD support
        //
        // On x86_64 with AVX2, this generates:
        // - `vmovdqu` for unaligned loads
        // - `vpcmpeqb` for byte-wise comparison
        // - Efficient handling of unaligned data
        a == b
    }

    /// Ultra-fast byte clearing using compiler-optimized memset
    ///
    /// The compiler/libc will automatically use SIMD memset when:
    /// - Buffer is large enough
    /// - Target platform has optimized memset (which uses SIMD internally)
    ///
    /// Modern memset implementations use AVX2/SSE2 on x86_64, NEON on ARM, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_orchestrator::core::optimization::simd_optimizations::SimdByteOps;
    ///
    /// let mut buffer = vec![0xFF; 1024];
    /// SimdByteOps::clear_bytes_safe(&mut buffer);
    /// assert!(buffer.iter().all(|&b| b == 0));
    /// ```
    #[inline]
    pub fn clear_bytes_safe(data: &mut [u8]) {
        // SAFE: Standard slice fill operation
        // LLVM optimizes this to:
        // 1. Call to `memset` for large buffers (which uses SIMD internally)
        // 2. Inline SIMD instructions for medium buffers
        // 3. Simple stores for tiny buffers
        //
        // On x86_64 with AVX2, libc's memset uses:
        // - `vmovdqu` for unaligned stores
        // - `rep stosb` for very large buffers (CPU microcode optimized)
        data.fill(0);
    }

    /// Optimized byte copy using compiler-optimized memcpy
    ///
    /// Like memset, modern memcpy uses SIMD internally for large copies.
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_orchestrator::core::optimization::simd_optimizations::SimdByteOps;
    ///
    /// let src = vec![0x42; 1024];
    /// let mut dst = vec![0; 1024];
    /// SimdByteOps::copy_bytes_safe(&src, &mut dst);
    /// assert_eq!(src, dst);
    /// ```
    #[inline]
    pub fn copy_bytes_safe(src: &[u8], dst: &mut [u8]) {
        assert_eq!(
            src.len(),
            dst.len(),
            "Source and destination must have the same length"
        );
        // SAFE: Standard slice copy
        // LLVM optimizes this to:
        // 1. Call to `memcpy` for large buffers (SIMD-optimized)
        // 2. Inline SIMD copy for medium buffers
        // 3. Simple register copies for tiny buffers
        dst.copy_from_slice(src);
    }

    /// Count bytes matching a value (SIMD-optimized by compiler)
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_orchestrator::core::optimization::simd_optimizations::SimdByteOps;
    ///
    /// let data = b"hello world";
    /// assert_eq!(SimdByteOps::count_byte_safe(data, b'l'), 3);
    /// ```
    #[inline]
    #[must_use]
    pub fn count_byte_safe(data: &[u8], byte: u8) -> usize {
        // SAFE: Standard iterator operation
        // LLVM auto-vectorizes this to SIMD operations on supported platforms
        data.iter().filter(|&&b| b == byte).count()
    }
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
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::*;

    #[test]
    fn test_compare_bytes_equal() {
        let a = b"hello world";
        let b = b"hello world";
        assert!(SimdByteOps::compare_bytes_safe(a, b));
    }

    #[test]
    fn test_compare_bytes_different() {
        let a = b"hello world";
        let b = b"hello rust!";
        assert!(!SimdByteOps::compare_bytes_safe(a, b));
    }

    #[test]
    fn test_compare_bytes_different_length() {
        let a = b"hello";
        let b = b"hello world";
        assert!(!SimdByteOps::compare_bytes_safe(a, b));
    }

    #[test]
    fn test_clear_bytes() {
        let mut data = vec![0xFF; 1024];
        SimdByteOps::clear_bytes_safe(&mut data);
        assert!(data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_copy_bytes() {
        let src = vec![0x42; 1024];
        let mut dst = vec![0; 1024];
        SimdByteOps::copy_bytes_safe(&src, &mut dst);
        assert_eq!(src, dst);
    }

    #[test]
    fn test_count_byte() {
        let data = b"hello world";
        assert_eq!(SimdByteOps::count_byte_safe(data, b'l'), 3);
        assert_eq!(SimdByteOps::count_byte_safe(data, b'o'), 2);
        assert_eq!(SimdByteOps::count_byte_safe(data, b'z'), 0);
    }

    #[test]
    fn test_large_buffer_comparison() {
        // Test with large buffers to trigger SIMD code path
        let a = vec![0x42; 4096];
        let b = vec![0x42; 4096];
        assert!(SimdByteOps::compare_bytes_safe(&a, &b));

        let mut c = vec![0x42; 4096];
        c[2048] = 0x43; // Change middle byte
        assert!(!SimdByteOps::compare_bytes_safe(&a, &c));
    }
}

// ============================================================================
// BENCHMARKING NOTES
// ============================================================================
//
// To verify SIMD generation and performance:
//
// 1. Compile with optimizations:
//    ```bash
//    RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release
//    ```
//
// 2. Check generated assembly:
//    ```bash
//    cargo asm --lib --release songbird_orchestrator::core::optimization::simd_optimizations::SimdByteOps::compare_bytes_safe
//    ```
//    Look for: `vmovdqu`, `vpcmpeqb` (AVX2) or `movdqu`, `pcmpeqb` (SSE2)
//
// 3. Run benchmarks:
//    ```bash
//    cargo bench --bench simd_ops
//    ```
//
// Expected results:
// - compare_bytes_safe: ~20-30 GB/s on modern CPUs (AVX2)
// - clear_bytes_safe:   ~30-40 GB/s (SIMD memset)
// - copy_bytes_safe:    ~20-30 GB/s (SIMD memcpy)
//
// These match or exceed hand-written SIMD performance in most cases!
