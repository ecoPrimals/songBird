// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Zero-Copy Utilities
//!
//! **CANONICAL**: Zero-copy and memory-efficient utilities for high-performance scenarios

use std::borrow::Cow;
use std::sync::Arc;

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
    #[allow(clippy::should_implement_trait)]
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

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
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
    fn test_shared_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let shared1 = Shared::new(data);
        let shared2 = shared1.clone();

        assert_eq!(shared1.as_ref(), shared2.as_ref());
        assert_eq!(shared1.as_ref().len(), 5);
        assert_eq!(shared2.as_ref().len(), 5);
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
}
