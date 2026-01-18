//! Pure Rust secure random number generation
//!
//! This module provides a `SecureRandom` implementation using the `getrandom` crate,
//! which is a Pure Rust interface to OS-provided entropy sources.
//!
//! # Security
//!
//! - Uses OS-level CSPRNG (e.g., `/dev/urandom` on Linux, `BCryptGenRandom` on Windows)
//! - No unsafe code
//! - Battle-tested (used by millions of Rust applications)
//! - Audited and actively maintained
//!
//! # Performance
//!
//! - Direct system calls, minimal overhead
//! - No additional buffering or processing
//! - Scales well with concurrent usage

use rustls::crypto::{GetRandomFailed, SecureRandom};

/// Pure Rust secure random number generator
///
/// Implements `rustls::crypto::SecureRandom` using the `getrandom` crate.
///
/// # Example
///
/// ```rust
/// use rustls::crypto::SecureRandom;
/// use songbird_orchestrator::crypto::rustls_provider::GETRANDOM_WRAPPER;
///
/// let mut buf = [0u8; 32];
/// GETRANDOM_WRAPPER.fill(&mut buf).expect("RNG failed");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct GetrandomWrapper;

impl SecureRandom for GetrandomWrapper {
    fn fill(&self, buf: &mut [u8]) -> Result<(), GetRandomFailed> {
        getrandom::getrandom(buf).map_err(|_| GetRandomFailed)
    }
}

/// Static instance for use in `CryptoProvider`
///
/// This is the singleton instance that will be used by all rustls connections.
pub static GETRANDOM_WRAPPER: GetrandomWrapper = GetrandomWrapper;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getrandom_fills_buffer() {
        let mut buf = [0u8; 32];
        GETRANDOM_WRAPPER.fill(&mut buf).expect("RNG should not fail");

        // Verify buffer was filled (should not be all zeros)
        assert!(
            buf.iter().any(|&b| b != 0),
            "Random buffer should not be all zeros"
        );
    }

    #[test]
    fn test_getrandom_different_calls_produce_different_bytes() {
        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];

        GETRANDOM_WRAPPER.fill(&mut buf1).expect("RNG should not fail");
        GETRANDOM_WRAPPER.fill(&mut buf2).expect("RNG should not fail");

        // Two consecutive calls should produce different random bytes
        assert_ne!(
            buf1, buf2,
            "Two random generations should produce different bytes"
        );
    }

    #[test]
    fn test_getrandom_works_with_different_sizes() {
        // Test with various buffer sizes
        for size in [1, 16, 32, 64, 128, 256, 1024] {
            let mut buf = vec![0u8; size];
            GETRANDOM_WRAPPER
                .fill(&mut buf)
                .expect("RNG should work with any buffer size");

            // Verify some bytes are non-zero (probabilistically certain for size > 1)
            if size > 1 {
                assert!(
                    buf.iter().any(|&b| b != 0),
                    "Random buffer of size {} should have some non-zero bytes",
                    size
                );
            }
        }
    }

    #[test]
    fn test_getrandom_empty_buffer() {
        let mut buf = [];
        GETRANDOM_WRAPPER
            .fill(&mut buf)
            .expect("RNG should handle empty buffer");
    }

    #[test]
    fn test_getrandom_is_send_sync() {
        // Compile-time check that GetrandomWrapper is Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<GetrandomWrapper>();
    }
}

