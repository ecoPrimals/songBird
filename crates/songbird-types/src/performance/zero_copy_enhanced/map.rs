// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use super::string::ZeroCopyString;

/// **ZERO-COST**: Performance statistics with atomic operations
#[derive(Debug)]
pub struct ZeroCopyStats {
    operations: AtomicUsize,
    lookups: AtomicUsize,
    start_time: Instant,
}

impl ZeroCopyStats {
    /// Create new statistics - zero cost
    #[inline]
    pub fn new() -> Self {
        Self {
            operations: AtomicUsize::new(0),
            lookups: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record operation - zero cost atomic increment
    #[inline]
    pub fn record_operation(&self) {
        self.operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record lookup - zero cost atomic increment
    #[inline]
    pub fn record_lookup(&self) {
        self.lookups.fetch_add(1, Ordering::Relaxed);
    }

    /// Get operation count - zero cost atomic read
    #[inline]
    pub fn operations(&self) -> usize {
        self.operations.load(Ordering::Relaxed)
    }

    /// Get lookup count - zero cost atomic read
    #[inline]
    pub fn lookups(&self) -> usize {
        self.lookups.load(Ordering::Relaxed)
    }

    /// Get operations per second
    #[allow(clippy::cast_precision_loss, reason = "usize -> f64 is acceptable for rate metrics")]
    pub fn operations_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.operations() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get lookups per second
    #[allow(clippy::cast_precision_loss, reason = "usize -> f64 is acceptable for rate metrics")]
    pub fn lookups_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.lookups() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.operations.store(0, Ordering::Relaxed);
        self.lookups.store(0, Ordering::Relaxed);
    }
}

impl Default for ZeroCopyStats {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ZeroCopyStats {
    fn clone(&self) -> Self {
        Self {
            operations: AtomicUsize::new(self.operations.load(Ordering::Relaxed)),
            lookups: AtomicUsize::new(self.lookups.load(Ordering::Relaxed)),
            start_time: self.start_time,
        }
    }
}

/// **ZERO-COPY**: Hash map with zero-copy keys and values
#[derive(Debug, Clone)]
pub struct ZeroCopyHashMap<'a, V>
where
    V: Clone,
{
    inner: HashMap<ZeroCopyString<'a>, V>,
    stats: ZeroCopyStats,
}

impl<'a, V> ZeroCopyHashMap<'a, V>
where
    V: Clone,
{
    /// Create new zero-copy hash map
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            stats: ZeroCopyStats::new(),
        }
    }

    /// Create with capacity - pre-allocate to avoid rehashing
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
            stats: ZeroCopyStats::new(),
        }
    }

    /// Insert with zero-copy key
    pub fn insert(&mut self, key: impl Into<ZeroCopyString<'a>>, value: V) -> Option<V> {
        let key = key.into();
        self.stats.record_operation();
        self.inner.insert(key, value)
    }

    /// Get value by zero-copy key - zero allocation lookup
    pub fn get<'b>(&self, key: &'b str) -> Option<&V>
    where
        'b: 'a,
    {
        self.stats.record_lookup();
        self.inner.get(&ZeroCopyString::Borrowed(key))
    }

    /// Get mutable value by key
    pub fn get_mut<'b>(&mut self, key: &'b str) -> Option<&mut V>
    where
        'b: 'a,
    {
        self.stats.record_lookup();
        self.inner.get_mut(&ZeroCopyString::Borrowed(key))
    }

    /// Remove value by key
    pub fn remove<'b>(&mut self, key: &'b str) -> Option<V>
    where
        'b: 'a,
    {
        self.stats.record_operation();
        self.inner.remove(&ZeroCopyString::Borrowed(key))
    }

    /// Get number of entries - zero cost
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty - zero cost
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get performance statistics
    #[inline]
    pub fn stats(&self) -> &ZeroCopyStats {
        &self.stats
    }
}

impl<V> Default for ZeroCopyHashMap<'_, V>
where
    V: Clone,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
