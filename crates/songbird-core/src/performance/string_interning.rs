//! High-Performance String Interning System
//!
//! This module provides zero-allocation string interning for frequently used strings
//! in the Songbird ecosystem, reducing memory overhead and improving performance.

use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

/// Global string interning system
static STRING_INTERNER: Lazy<StringInterner> = Lazy::new(StringInterner::new);

/// Interned string handle - zero-copy string reference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternedString {
    inner: Arc<str>,
    hash: u64,
}

impl InternedString {
    /// Get the string content as a &str
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get the pre-computed hash
    #[must_use]
    pub const fn hash(&self) -> u64 {
        self.hash
    }

    /// Check if this string is interned
    #[must_use]
    pub fn is_interned(&self) -> bool {
        STRING_INTERNER.contains(&self.inner)
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl std::ops::Deref for InternedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// High-performance string interning system
pub struct StringInterner {
    /// Interned strings storage
    strings: RwLock<HashMap<u64, Arc<str>>>,
    /// Statistics for monitoring
    stats: RwLock<InternerStats>,
}

/// Performance statistics for the string interner
#[derive(Debug, Default)]
pub struct InternerStats {
    /// Total number of intern requests
    pub total_requests: u64,
    /// Number of cache hits (existing strings)
    pub cache_hits: u64,
    /// Number of new strings created
    pub new_strings: u64,
    /// Current number of interned strings
    pub current_count: usize,
    /// Total memory saved (estimated)
    pub memory_saved_bytes: u64,
}

impl InternerStats {
    /// Calculate cache hit ratio
    #[must_use]
    pub fn hit_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }

    /// Calculate memory efficiency
    #[must_use]
    pub fn memory_efficiency(&self) -> f64 {
        if self.new_strings == 0 {
            0.0
        } else {
            self.memory_saved_bytes as f64 / (self.new_strings * 64) as f64
        }
    }
}

impl StringInterner {
    /// Create a new string interner
    #[must_use]
    pub fn new() -> Self {
        Self {
            strings: RwLock::new(HashMap::with_capacity(1024)),
            stats: RwLock::new(InternerStats::default()),
        }
    }

    /// Intern a string, returning a zero-copy handle
    pub fn intern(&self, s: &str) -> InternedString {
        // Calculate hash once
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let hash = hasher.finish();

        // Update stats
        {
            let mut stats = self.stats.write().expect("Test operation should succeed");
            stats.total_requests += 1;
        }

        // Try to find existing string
        {
            let strings = self.strings.read().expect("Test operation should succeed");
            if let Some(existing) = strings.get(&hash) {
                // Cache hit - update stats
                {
                    let mut stats = self.stats.write().expect("Test operation should succeed");
                    stats.cache_hits += 1;
                    stats.memory_saved_bytes += s.len() as u64;
                }

                return InternedString {
                    inner: Arc::clone(existing),
                    hash,
                };
            }
        }

        // Need to create new interned string
        let arc_str: Arc<str> = Arc::from(s);

        // Insert into storage
        {
            let mut strings = self.strings.write().expect("Test operation should succeed");
            // Double-check in case another thread inserted it
            if let Some(existing) = strings.get(&hash) {
                {
                    let mut stats = self.stats.write().expect("Test operation should succeed");
                    stats.cache_hits += 1;
                    stats.memory_saved_bytes += s.len() as u64;
                }

                return InternedString {
                    inner: Arc::clone(existing),
                    hash,
                };
            }

            strings.insert(hash, Arc::clone(&arc_str));
        }

        // Update stats for new string
        {
            let mut stats = self.stats.write().expect("Test operation should succeed");
            stats.new_strings += 1;
            stats.current_count = self
                .strings
                .read()
                .map_err(|e| SongbirdError::internal(format!("Operation failed: {:?}", e)))?
                .len();
        }

        InternedString {
            inner: arc_str,
            hash,
        }
    }

    /// Check if a string is already interned
    pub fn contains(&self, s: &str) -> bool {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let hash = hasher.finish();

        self.strings
            .read()
            .map_err(|e| SongbirdError::internal(format!("Operation failed: {:?}", e)))?
            .contains_key(&hash)
    }

    /// Get current statistics
    #[must_use]
    pub fn stats(&self) -> InternerStats {
        let stats = self.stats.read().expect("Test operation should succeed");
        let current_count = self
            .strings
            .read()
            .map_err(|e| SongbirdError::internal(format!("Operation failed: {:?}", e)))?
            .len();

        InternerStats {
            total_requests: stats.total_requests,
            cache_hits: stats.cache_hits,
            new_strings: stats.new_strings,
            current_count,
            memory_saved_bytes: stats.memory_saved_bytes,
        }
    }

    /// Clear all interned strings (for testing/cleanup)
    pub fn clear(&self) {
        let mut strings = self.strings.write().expect("Test operation should succeed");
        let mut stats = self.stats.write().expect("Test operation should succeed");

        strings.clear();
        *stats = InternerStats::default();
    }

    /// Get memory usage estimate in bytes
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        let strings = self.strings.read().expect("Test operation should succeed");
        strings
            .iter()
            .map(|(_, s)| s.len() + std::mem::size_of::<Arc<str>>())
            .sum::<usize>()
            + strings.capacity() * std::mem::size_of::<(u64, Arc<str>)>()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to intern a string using the global interner
#[must_use]
pub fn intern(s: &str) -> InternedString {
    STRING_INTERNER.intern(s)
}

/// Get global interner statistics
#[must_use]
pub fn global_stats() -> InternerStats {
    STRING_INTERNER.stats()
}

/// Clear the global interner (for testing)
pub fn clear_global() {
    STRING_INTERNER.clear();
}

/// Common strings that are frequently used in Songbird
pub mod common {
    use super::*;

    /// Commonly used capability strings
    pub static SECURITY: Lazy<InternedString> = Lazy::new(|| intern("security"));
    pub static STORAGE: Lazy<InternedString> = Lazy::new(|| intern("storage"));
    pub static COMPUTE: Lazy<InternedString> = Lazy::new(|| intern("compute"));
    pub static AI: Lazy<InternedString> = Lazy::new(|| intern("ai"));
    pub static NETWORKING: Lazy<InternedString> = Lazy::new(|| intern("networking"));
    pub static HEALTH: Lazy<InternedString> = Lazy::new(|| intern("health"));

    /// Common operation strings
    pub static ENCRYPT: Lazy<InternedString> = Lazy::new(|| intern("encrypt"));
    pub static DECRYPT: Lazy<InternedString> = Lazy::new(|| intern("decrypt"));
    pub static STORE: Lazy<InternedString> = Lazy::new(|| intern("store"));
    pub static RETRIEVE: Lazy<InternedString> = Lazy::new(|| intern("retrieve"));
    pub static HEALTH_CHECK: Lazy<InternedString> = Lazy::new(|| intern("health_check"));

    /// Common status strings
    pub static SUCCESS: Lazy<InternedString> = Lazy::new(|| intern("success"));
    pub static ERROR: Lazy<InternedString> = Lazy::new(|| intern("error"));
    pub static PENDING: Lazy<InternedString> = Lazy::new(|| intern("pending"));
    pub static COMPLETED: Lazy<InternedString> = Lazy::new(|| intern("completed"));

    /// Common endpoint patterns
    pub static LOCALHOST: Lazy<InternedString> = Lazy::new(|| intern("localhost"));
    pub static HTTP_PROTOCOL: Lazy<InternedString> = Lazy::new(|| intern("http"));
    pub static HTTPS_PROTOCOL: Lazy<InternedString> = Lazy::new(|| intern("https"));
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_string_interning() {
        let interner = StringInterner::new();

        // First insertion
        let id1 = interner.intern("hello").await;
        let id2 = interner.intern("hello").await; // Should return same ID
        assert_eq!(id1, id2);

        // Different string
        let id3 = interner.intern("world").await;
        assert_ne!(id1, id3);

        // Verify retrieval
        assert_eq!(interner.get_string(id1).await, Some("hello".to_string()));
        assert_eq!(interner.get_string(id3).await, Some("world".to_string()));

        // Check statistics
        let stats = interner.get_statistics().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.new_strings, 1);
    }

    #[tokio::test]
    async fn test_interner_stats() {
        let interner = StringInterner::new();

        interner.intern("test1").await;
        interner.intern("test2").await;
        interner.intern("test1").await; // Should be cache hit

        let stats = interner.get_statistics().await;
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.new_strings, 2);
        assert_eq!(stats.current_count, 2);
    }

    #[tokio::test]
    async fn test_memory_efficiency() {
        let interner = StringInterner::new();

        // Insert same string 100 times
        for _ in 0..100 {
            interner.intern("repeated").await;
        }

        let stats = interner.get_statistics().await;
        assert_eq!(stats.new_strings, 1); // Only one unique string
        assert_eq!(stats.cache_hits, 99); // 99 cache hits
    }

    #[test]
    fn test_global_interner() {
        clear_global();

        let s1 = intern("global_test");
        let s2 = intern("global_test");

        assert_eq!(s1.as_str(), s2.as_str());

        let stats = global_stats();
        assert!(stats.cache_hits > 0);
    }

    #[test]
    fn test_common_strings() {
        // Test that common strings are properly interned
        let security1 = &*common::SECURITY;
        let security2 = intern("security");

        assert_eq!(security1.as_str(), security2.as_str());
    }

    #[tokio::test]
    async fn test_interned_string_operations() {
        let s = intern("test_operations");

        // Test Display
        assert_eq!(format!("{s}"), "test_operations");

        // Test AsRef
        let s_ref: &str = s.as_ref();
        assert_eq!(s_ref, "test_operations");

        // Test Deref
        assert_eq!(&*s, "test_operations");

        // Test hash consistency
        let s2 = intern("test_operations");
        assert_eq!(s.hash(), s2.hash());
    }
}
