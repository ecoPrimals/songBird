// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Zero-Copy Utilities
//!
//! **CANONICAL**: Zero-copy and memory-efficient utilities for high-performance scenarios

use bytes::Bytes;
use std::borrow::Cow;
use std::sync::Arc;

/// Zero-copy byte buffer for IPC payloads.
/// Wraps `bytes::Bytes` for reference-counted, zero-copy sharing.
#[derive(Debug, Clone)]
pub struct SharedBytes {
    inner: Bytes,
}

impl SharedBytes {
    /// Wrap existing `Bytes` without copying.
    #[must_use]
    pub const fn from_bytes(bytes: Bytes) -> Self {
        Self {
            inner: bytes,
        }
    }

    /// Borrow the underlying `Bytes` handle.
    #[must_use]
    pub const fn as_bytes(&self) -> &Bytes {
        &self.inner
    }

    /// Consume `self`, returning the underlying `Bytes`.
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }
}

impl From<Bytes> for SharedBytes {
    fn from(inner: Bytes) -> Self {
        Self {
            inner,
        }
    }
}

impl From<Vec<u8>> for SharedBytes {
    fn from(value: Vec<u8>) -> Self {
        Self {
            inner: Bytes::from(value),
        }
    }
}

impl From<&[u8]> for SharedBytes {
    fn from(value: &[u8]) -> Self {
        Self {
            inner: Bytes::copy_from_slice(value),
        }
    }
}

impl From<String> for SharedBytes {
    fn from(value: String) -> Self {
        Self {
            inner: Bytes::from(value),
        }
    }
}

impl AsRef<[u8]> for SharedBytes {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

/// Shared reference wrapper for zero-copy sharing
#[derive(Debug, Clone)]
pub struct Shared<T> {
    /// Inner Arc-wrapped data
    inner: Arc<T>,
}

impl<T> Shared<T> {
    /// Create a new shared reference
    #[must_use]
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(data),
        }
    }

    /// Get a reference to the inner data
    #[must_use]
    #[allow(
        clippy::should_implement_trait,
        reason = "intentional pattern; clippy false positive for this API"
    )]
    pub fn as_ref(&self) -> &T {
        &self.inner
    }

    /// Get the Arc directly
    #[must_use]
    pub const fn as_arc(&self) -> &Arc<T> {
        &self.inner
    }

    /// Try to get mutable access (only works if there's exactly one reference)
    pub fn get_mut(&mut self) -> Option<&mut T> {
        Arc::get_mut(&mut self.inner)
    }

    /// Clone the inner Arc
    #[must_use]
    pub fn clone_arc(&self) -> Arc<T> {
        Arc::clone(&self.inner)
    }
}

/// Trait for types that can be converted to shared references
pub trait Shareable: Sized {
    /// Convert to a shared reference
    fn into_shared(self) -> Shared<Self> {
        Shared::new(self)
    }
}

// Implement Shareable for common types
impl<T> Shareable for T {}

/// Utility function to create a shared reference
#[must_use]
pub fn share<T>(item: T) -> Shared<T> {
    Shared::new(item)
}

/// Utility function to create an Arc
#[must_use]
pub fn arc<T>(item: T) -> Arc<T> {
    Arc::new(item)
}

/// Smart Cow that can decide whether to borrow or own based on usage
#[must_use]
pub fn smart_cow<T: Clone>(item: &T, need_owned: bool) -> Cow<'_, T> {
    if need_owned {
        Cow::Owned(item.clone())
    } else {
        Cow::Borrowed(item)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::useless_vec,
    clippy::unreadable_literal,
    clippy::items_after_statements,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "test assertions and harness ergonomics"
)]
mod tests {
    #![allow(clippy::expect_used, reason = "test assertions")]
    #![allow(clippy::all, reason = "test assertions and harness ergonomics")]
    #![allow(unused, reason = "unused bindings/imports in this compilation unit")]

    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_shared_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let shared1 = Shared::new(data);
        let shared2 = shared1.clone();

        assert_eq!(shared1.as_ref(), shared2.as_ref());
        assert_eq!(shared1.as_ref().len(), 5);
        assert_eq!(shared2.as_ref().len(), 5);
    }

    #[test]
    fn test_shared_get_mut_when_unique() {
        let mut s = Shared::new(42u32);
        *s.get_mut().expect("unique ref") += 1;
        assert_eq!(*s.as_ref(), 43);
    }

    #[test]
    fn test_shared_get_mut_none_when_cloned() {
        let mut s = Shared::new(1u32);
        let _t = s.clone();
        assert!(s.get_mut().is_none());
    }

    #[test]
    fn test_smart_cow() {
        let data = "test".to_string();
        let cow_borrowed = smart_cow(&data, false);
        let cow_owned = smart_cow(&data, true);

        match cow_borrowed {
            Cow::Borrowed(_) => {} // Expected
            Cow::Owned(_) => panic!("Expected borrowed, got owned"),
        }

        match cow_owned {
            Cow::Owned(_) => {} // Expected
            Cow::Borrowed(_) => panic!("Expected owned, got borrowed"),
        }
    }

    #[test]
    fn test_shareable_trait() {
        let data = vec![1, 2, 3];
        let shared = data.into_shared();
        assert_eq!(shared.as_ref().len(), 3);
    }

    #[test]
    fn test_utility_functions() {
        let data = "test".to_string();
        let shared = share(data.clone());
        let arc_data = arc(data);

        assert_eq!(shared.as_ref(), arc_data.as_ref());
    }

    #[test]
    fn shared_bytes_from_vec_roundtrip() {
        let v = vec![1u8, 2, 3];
        let sb = SharedBytes::from(v);
        assert_eq!(sb.as_ref(), &[1, 2, 3]);
        let b: Bytes = sb.into_bytes();
        assert_eq!(b.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn shared_bytes_from_slice_copies() {
        let slice = b"hello";
        let sb = SharedBytes::from(slice.as_slice());
        assert_eq!(sb.as_ref(), slice);
    }

    #[test]
    fn shared_bytes_from_string_utf8() {
        let s = "json".to_string();
        let sb = SharedBytes::from(s);
        assert_eq!(sb.as_ref(), b"json");
    }

    #[test]
    fn shared_bytes_from_bytes_identity() {
        let b = Bytes::from_static(b"abc");
        let sb = SharedBytes::from_bytes(b.clone());
        assert_eq!(sb.as_bytes(), &b);
    }

    #[test]
    fn test_clone_arc_matches_inner() {
        let s = Shared::new("x".to_string());
        let a = s.clone_arc();
        assert_eq!(a.as_ref(), s.as_ref());
    }
}
