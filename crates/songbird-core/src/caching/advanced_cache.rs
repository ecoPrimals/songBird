//! Advanced Caching System for Songbird Universal Orchestrator
//!
//! This module provides a high-performance, feature-rich caching layer with:
//! - LRU (Least Recently Used) eviction policy
//! - TTL (Time To Live) support for automatic expiration
//! - Multi-tier caching with different storage backends
//! - Integration with metrics dashboard for monitoring
//! - Zero-copy optimizations where possible

use crate::performance::string_interning::InternedString;
use serde::{Deserialize, Serialize};
use songbird_errors::SongbirdError;
use songbird_errors::EvolvedResult;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Global advanced cache instance
static ADVANCED_CACHE: once_cell::sync::Lazy<AdvancedCache> =
    once_cell::sync::Lazy::new(AdvancedCache::new);

/// Advanced caching system with multiple eviction policies and TTL support
#[derive(Debug)]
pub struct AdvancedCache {
    /// Primary cache storage
    storage: Arc<RwLock<CacheStorage>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: Arc<RwLock<CacheStatistics>>,
    /// Cache start time for uptime tracking
    start_time: Instant,
}

/// Cache storage backend
#[derive(Debug)]
struct CacheStorage {
    /// Main data storage
    data: HashMap<CacheKey, CacheEntry>,
    /// LRU tracking queue
    lru_queue: VecDeque<CacheKey>,
    /// TTL expiration tracking
    expiration_queue: VecDeque<(Instant, CacheKey)>,
    /// Current cache size in bytes
    current_size_bytes: usize,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum cache size in bytes
    pub max_size_bytes: usize,
    /// Default TTL for entries (None = no expiration)
    pub default_ttl: Option<Duration>,
    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Cleanup interval for expired entries
    pub cleanup_interval: Duration,
    /// Enable cache compression
    pub enable_compression: bool,
    /// Enable cache persistence
    pub enable_persistence: bool,
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In, First Out
    FIFO,
    /// Random eviction
    Random,
    /// Time-based eviction only
    TTLOnly,
}

/// Cache key type supporting different key formats
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    /// String key (interned for efficiency)
    String(InternedString),
    /// Binary key
    Binary(Vec<u8>),
    /// Composite key with namespace
    Namespaced {
        namespace: InternedString,
        key: InternedString,
    },
    /// Numeric key
    Numeric(u64),
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached value
    value: CacheValue,
    /// Entry creation time
    created_at: Instant,
    /// Last access time
    last_accessed: Instant,
    /// Access count for LFU
    access_count: u64,
    /// Time to live (None = no expiration)
    ttl: Option<Duration>,
    /// Entry size in bytes
    size_bytes: usize,
    /// Entry metadata
    metadata: HashMap<String, String>,
}

/// Cache value types supporting zero-copy operations
#[derive(Debug, Clone)]
pub enum CacheValue {
    /// String value (interned)
    String(InternedString),
    /// Binary data
    Binary(Arc<Vec<u8>>),
    /// JSON data
    Json(Arc<serde_json::Value>),
    /// Serialized data with type information
    Serialized {
        data: Arc<Vec<u8>>,
        type_hint: String,
    },
    /// Reference to external data
    Reference {
        location: String,
        checksum: Option<String>,
    },
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Total cache operations
    pub total_operations: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache evictions
    pub evictions: u64,
    /// Expired entries cleaned up
    pub expirations: u64,
    /// Current entry count
    pub current_entries: usize,
    /// Current size in bytes
    pub current_size_bytes: usize,
    /// Hit ratio percentage
    pub hit_ratio: f64,
    /// Average access time in microseconds
    pub avg_access_time_us: f64,
    /// Memory efficiency (data size / total size)
    pub memory_efficiency: f64,
    /// Cache uptime in seconds
    pub uptime_seconds: u64,
}

/// Cache operation result
#[derive(Debug, Clone)]
pub struct CacheOperationResult<T> {
    /// The result value
    pub value: T,
    /// Operation timing
    pub timing: CacheOperationTiming,
    /// Cache statistics snapshot
    pub stats: CacheStatistics,
}

/// Cache operation timing information
#[derive(Debug, Clone)]
pub struct CacheOperationTiming {
    /// Total operation duration
    pub total_duration: Duration,
    /// Lock acquisition time
    pub lock_duration: Duration,
    /// Data processing time
    pub processing_duration: Duration,
}

impl AdvancedCache {
    /// Create a new advanced cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new advanced cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            storage: Arc::new(RwLock::new(CacheStorage::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStatistics::default())),
            start_time: Instant::now(),
        }
    }

    /// Get the global cache instance
    pub fn global() -> &'static AdvancedCache {
        &ADVANCED_CACHE
    }

    /// Store a value in the cache
    pub fn set<V>(&self, key: CacheKey, value: V) -> SongbirdResult<CacheOperationResult<()>>
    where
        V: Into<CacheValue>,
    {
        self.set_with_ttl(key, value, self.config.default_ttl)
    }

    /// Store a value in the cache with custom TTL
    pub fn set_with_ttl<V>(
        &self,
        key: CacheKey,
        value: V,
        ttl: Option<Duration>,
    ) -> SongbirdResult<CacheOperationResult<()>>
    where
        V: Into<CacheValue>,
    {
        let start_time = Instant::now();
        let lock_start = Instant::now();

        let cache_value = value.into();
        let entry_size = self.estimate_entry_size(&key, &cache_value);

        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {}", e)))?;

        let lock_duration = lock_start.elapsed();
        let processing_start = Instant::now();

        // Check if we need to evict entries
        while (storage.current_size_bytes + entry_size > self.config.max_size_bytes
            || storage.data.len() >= self.config.max_entries)
            && !storage.data.is_empty()
        {
            self.evict_entry(&mut storage)?;
        }

        let now = Instant::now();
        let entry = CacheEntry {
            value: cache_value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
            size_bytes: entry_size,
            metadata: HashMap::new(),
        };

        // Update LRU queue
        if let Some(pos) = storage.lru_queue.iter().position(|k| k == &key) {
            storage.lru_queue.remove(pos);
        }
        storage.lru_queue.push_back(key.clone());

        // Update expiration queue if TTL is set
        if let Some(ttl_duration) = ttl {
            storage
                .expiration_queue
                .push_back((now + ttl_duration, key.clone()));
        }

        // Store the entry
        let old_entry = storage.data.insert(key, entry);
        if old_entry.is_none() {
            storage.current_size_bytes += entry_size;
        }

        let processing_duration = processing_start.elapsed();
        drop(storage);

        // Update statistics
        self.update_stats(|stats| {
            stats.total_operations += 1;
            stats.current_entries = self.len();
            stats.current_size_bytes = self.size_bytes();
        })?;

        let total_duration = start_time.elapsed();
        let timing = CacheOperationTiming {
            total_duration,
            lock_duration,
            processing_duration,
        };

        Ok(CacheOperationResult {
            value: (),
            timing,
            stats: self.get_statistics()?.data,
        }
        .into())
    }

    /// Retrieve a value from the cache
    pub fn get(&self, key: &CacheKey) -> SongbirdResult<CacheOperationResult<Option<CacheValue>>> {
        let start_time = Instant::now();
        let lock_start = Instant::now();

        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {}", e)))?;

        let lock_duration = lock_start.elapsed();
        let processing_start = Instant::now();

        let result = {
            // First check if entry exists and is expired
            let should_remove = if let Some(entry) = storage.data.get(key) {
                if let Some(ttl) = entry.ttl {
                    entry.created_at.elapsed() > ttl
                } else {
                    false
                }
            } else {
                false
            };

            if should_remove {
                // Entry is expired, remove it
                if let Some(entry) = storage.data.remove(key) {
                    self.remove_from_lru_queue(&mut storage, key);
                    storage.current_size_bytes =
                        storage.current_size_bytes.saturating_sub(entry.size_bytes);

                    self.update_stats(|stats| {
                        stats.total_operations += 1;
                        stats.misses += 1;
                        stats.expirations += 1;
                        stats.current_entries = storage.data.len();
                        stats.current_size_bytes = storage.current_size_bytes;
                    })?;
                }
                None
            } else if let Some(entry) = storage.data.get_mut(key) {
                // Entry exists and is not expired, update access info
                entry.last_accessed = Instant::now();
                entry.access_count += 1;

                // Clone the value before doing LRU operations
                let value = entry.value.clone();

                // Update LRU queue
                self.remove_from_lru_queue(&mut storage, key);
                storage.lru_queue.push_back(key.clone());

                self.update_stats(|stats| {
                    stats.total_operations += 1;
                    stats.hits += 1;
                })?;

                Some(value)
            } else {
                self.update_stats(|stats| {
                    stats.total_operations += 1;
                    stats.misses += 1;
                })?;

                None
            }
        };

        let processing_duration = processing_start.elapsed();
        drop(storage);

        let total_duration = start_time.elapsed();
        let timing = CacheOperationTiming {
            total_duration,
            lock_duration,
            processing_duration,
        };

        Ok(CacheOperationResult {
            value: result,
            timing,
            stats: self.get_statistics()?.data,
        }
        .into())
    }

    /// Remove a value from the cache
    pub fn remove(
        &self,
        key: &CacheKey,
    ) -> SongbirdResult<CacheOperationResult<Option<CacheValue>> {
        let start_time = Instant::now();
        let lock_start = Instant::now();

        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {}", e)))?;

        let lock_duration = lock_start.elapsed();
        let processing_start = Instant::now();

        let result = if let Some(entry) = storage.data.remove(key) {
            storage.current_size_bytes =
                storage.current_size_bytes.saturating_sub(entry.size_bytes);
            self.remove_from_lru_queue(&mut storage, key);
            Some(entry.value)
        } else {
            None
        };

        let processing_duration = processing_start.elapsed();
        drop(storage);

        // Update statistics
        self.update_stats(|stats| {
            stats.total_operations += 1;
            stats.current_entries = self.len();
            stats.current_size_bytes = self.size_bytes();
        })?;

        let total_duration = start_time.elapsed();
        let timing = CacheOperationTiming {
            total_duration,
            lock_duration,
            processing_duration,
        };

        Ok(CacheOperationResult {
            value: result,
            timing,
            stats: self.get_statistics()?.data,
        }
        .into())
    }

    /// Check if a key exists in the cache
    pub async fn contains_key(&self, key: &CacheKey) -> SongbirdResult<bool> {
        let storage = self
            .storage
            .read()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {}", e)))?;

        Ok(songbird_errors::evolved_success(storage.data.contains_key(key)).into())
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        if let Ok(songbird_errors::evolved_success(storage)) = self.storage.read() {
            storage.data.len()
        } else {
            0
        }
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the total size of the cache in bytes
    pub fn size_bytes(&self) -> usize {
        if let Ok(songbird_errors::evolved_success(storage)) = self.storage.read() {
            storage.current_size_bytes
        } else {
            0
        }
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) -> SongbirdResult<()> {
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {}", e)))?;

        storage.data.clear();
        storage.lru_queue.clear();
        storage.expiration_queue.clear();
        storage.current_size_bytes = 0;

        // Update statistics
        self.update_stats(|stats| {
            stats.current_entries = 0;
            stats.current_size_bytes = 0;
        })?;

        Ok(())
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> SongbirdResult<usize> {
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {}", e)))?;

        let now = Instant::now();
        let mut expired_count = 0;

        // Remove expired entries from expiration queue
        while let Some((expiry_time, key)) = storage.expiration_queue.front().cloned() {
            if expiry_time <= now {
                storage.expiration_queue.pop_front();
                if let Some(entry) = storage.data.remove(&key) {
                    storage.current_size_bytes =
                        storage.current_size_bytes.saturating_sub(entry.size_bytes);
                    self.remove_from_lru_queue(&mut storage, &key);
                    expired_count += 1;
                }
            } else {
                break;
            }
        }

        drop(storage);

        // Update statistics
        self.update_stats(|stats| {
            stats.expirations += expired_count as u64;
            stats.current_entries = self.len();
            stats.current_size_bytes = self.size_bytes();
        })?;

        Ok(songbird_errors::evolved_success(success(expired_count)))
    }

    /// Get cache statistics
    pub async fn get_statistics(&self) -> SongbirdResult<CacheStatistics> {
        let stats = self
            .stats
            .read()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {}", e)))?;

        let mut stats_clone = stats.clone();
        stats_clone.uptime_seconds = self.start_time.elapsed().as_secs();
        stats_clone.hit_ratio = if stats_clone.total_operations > 0 {
            (stats_clone.hits as f64 / stats_clone.total_operations as f64) * 100.0
        } else {
            0.0
        };

        Ok(songbird_errors::evolved_success(success(stats_clone)))
    }

    /// Get cache configuration
    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }

    /// Start automatic cleanup of expired entries
    pub async fn start_cleanup_task(&self) -> SongbirdResult<()> {
        let mut interval_timer = tokio::time::interval(interval);

        loop {
            interval_timer.tick().await;

            if let Err(e) = self.cleanup_expired() {
                eprintln!("Cache cleanup error: {}", e);
            }
        }
    }

    // Private helper methods

    fn evict_entry(&self, storage: &mut CacheStorage) -> SongbirdResult<()> {
        match self.config.eviction_policy {
            EvictionPolicy::LRU => {
                if let Some(key) = storage.lru_queue.pop_front() {
                    if let Some(entry) = storage.data.remove(&key) {
                        storage.current_size_bytes =
                            storage.current_size_bytes.saturating_sub(entry.size_bytes);
                        self.update_stats(|stats| stats.evictions += 1)?;
                    }
                }
            }
            EvictionPolicy::LFU => {
                // Find entry with lowest access count
                let key_to_remove = storage
                    .data
                    .iter()
                    .min_by_key(|(_, entry)| entry.access_count)
                    .map(|(key, _)| key.clone());

                if let Some(key) = key_to_remove {
                    if let Some(entry) = storage.data.remove(&key) {
                        storage.current_size_bytes =
                            storage.current_size_bytes.saturating_sub(entry.size_bytes);
                        self.remove_from_lru_queue(storage, &key);
                        self.update_stats(|stats| stats.evictions += 1)?;
                    }
                }
            }
            EvictionPolicy::FIFO => {
                // Find oldest entry
                let key_to_remove = storage
                    .data
                    .iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(key, _)| key.clone());

                if let Some(key) = key_to_remove {
                    if let Some(entry) = storage.data.remove(&key) {
                        storage.current_size_bytes =
                            storage.current_size_bytes.saturating_sub(entry.size_bytes);
                        self.remove_from_lru_queue(storage, &key);
                        self.update_stats(|stats| stats.evictions += 1)?;
                    }
                }
            }
            EvictionPolicy::Random => {
                // Remove a random entry
                if !storage.data.is_empty() {
                    let keys: Vec<_> = storage.data.keys().cloned().collect();
                    if let Some(key) = keys.first() {
                        if let Some(entry) = storage.data.remove(key) {
                            storage.current_size_bytes =
                                storage.current_size_bytes.saturating_sub(entry.size_bytes);
                            self.remove_from_lru_queue(storage, key);
                            self.update_stats(|stats| stats.evictions += 1)?;
                        }
                    }
                }
            }
            EvictionPolicy::TTLOnly => {
                // Only evict expired entries, not size-based eviction
                return Ok(());
            }
        }

        Ok(())
    }

    fn remove_from_lru_queue(&self, storage: &mut CacheStorage, key: &CacheKey) {
        if let Some(pos) = storage.lru_queue.iter().position(|k| k == key) {
            storage.lru_queue.remove(pos);
        }
    }

    fn estimate_entry_size(&self, key: &CacheKey, value: &CacheValue) -> usize {
        let key_size = match key {
            CacheKey::String(s) => s.len(),
            CacheKey::Binary(b) => b.len(),
            CacheKey::Namespaced { namespace, key } => namespace.len() + key.len(),
            CacheKey::Numeric(_) => 8,
        };

        let value_size = match value {
            CacheValue::String(s) => s.len(),
            CacheValue::Binary(b) => b.len(),
            CacheValue::Json(j) => j.to_string().len(),
            CacheValue::Serialized { data, .. } => data.len(),
            CacheValue::Reference { location, .. } => location.len(),
        };

        key_size + value_size + 128 // Add overhead for metadata
    }

    fn update_stats<F>(&self, updater: F) -> SongbirdResult<()>
    where
        F: FnOnce(&mut CacheStatistics),
    {
        let mut stats = self
            .stats
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {}", e)))?;
        updater(&mut stats);
        Ok(())
    }
}

impl CacheStorage {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            lru_queue: VecDeque::new(),
            expiration_queue: VecDeque::new(),
            current_size_bytes: 0,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_size_bytes: 100 * 1024 * 1024,            // 100MB
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour
            eviction_policy: EvictionPolicy::LRU,
            cleanup_interval: Duration::from_secs(60), // 1 minute
            enable_compression: false,
            enable_persistence: false,
        }
    }
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            expirations: 0,
            current_entries: 0,
            current_size_bytes: 0,
            hit_ratio: 0.0,
            avg_access_time_us: 0.0,
            memory_efficiency: 0.0,
            uptime_seconds: 0,
        }
    }
}

// Conversion implementations for CacheValue
impl From<String> for CacheValue {
    fn from(s: String) -> Self {
        CacheValue::String(crate::performance::string_interning::intern(&s))
    }
}

impl From<&str> for CacheValue {
    fn from(s: &str) -> Self {
        CacheValue::String(crate::performance::string_interning::intern(s))
    }
}

impl From<Vec<u8>> for CacheValue {
    fn from(data: Vec<u8>) -> Self {
        CacheValue::Binary(Arc::new(data))
    }
}

impl From<serde_json::Value> for CacheValue {
    fn from(json: serde_json::Value) -> Self {
        CacheValue::Json(Arc::new(json))
    }
}

// Conversion implementations for CacheKey
impl From<String> for CacheKey {
    fn from(s: String) -> Self {
        CacheKey::String(crate::performance::string_interning::intern(&s))
    }
}

impl From<&str> for CacheKey {
    fn from(s: &str) -> Self {
        CacheKey::String(crate::performance::string_interning::intern(s))
    }
}

impl From<u64> for CacheKey {
    fn from(n: u64) -> Self {
        CacheKey::Numeric(n)
    }
}

impl From<Vec<u8>> for CacheKey {
    fn from(data: Vec<u8>) -> Self {
        CacheKey::Binary(data)
    }
}

/// Convenience functions for common cache operations
pub mod cache_ops {
    use super::*;
use songbird_errors::SongbirdResult;

    /// Store a JSON-serializable value in the global cache
    pub fn set_json<T: serde::Serialize>(
        key: impl Into<CacheKey>,
        value: &T,
        ttl: Option<Duration>,
    ) -> SongbirdResult<()> {
        let json_value = serde_json::to_value(value).map_err(|e| {
            SongbirdError::service("cache", format!("JSON serialization error: {}", e))
        })?;

        AdvancedCache::global().set_with_ttl(key.into(), json_value, ttl)?;
        Ok(())
    }

    /// Retrieve and deserialize a JSON value from the global cache
    pub fn get_json<T: serde::de::DeserializeOwned>(key: &CacheKey) -> SongbirdResult<Option<T>> {
        let result = AdvancedCache::global().get(key)?;

        if let Some(CacheValue::Json(json_arc)) = result.data.value {
            let value = serde_json::from_value((*json_arc).clone()).map_err(|e| {
                SongbirdError::service("cache", format!("JSON deserialization error: {}", e))
            })?;
            Ok(success(Some(value)))
        } else {
            Ok(songbird_errors::evolved_success(success(None)))
        }
    }

    /// Store a string value in the global cache
    pub async fn set_string(
        key: impl Into<CacheKey>,
        value: impl Into<String>,
        ttl: Option<Duration>,
    ) -> SongbirdResult<()> {
        AdvancedCache::global().set_with_ttl(key.into(), value.into(), ttl)?;
        Ok(())
    }

    /// Retrieve a string value from the global cache
    pub fn get_string(key: &CacheKey) -> SongbirdResult<Option<String>> {
        let result = AdvancedCache::global().get(key)?;

        if let Some(CacheValue::String(interned)) = result.data.value {
            Ok(songbird_errors::evolved_success(Some(interned.to_string())).into())
        } else {
            Ok(songbird_errors::evolved_success(success(None)))
        }
    }

    /// Store binary data in the global cache
    pub async fn set_binary(
        key: impl Into<CacheKey>,
        data: Vec<u8>,
        ttl: Option<Duration>,
    ) -> SongbirdResult<()> {
        AdvancedCache::global().set_with_ttl(key.into(), data, ttl)?;
        Ok(())
    }

    /// Retrieve binary data from the global cache
    pub fn get_binary(key: &CacheKey) -> SongbirdResult<Option<Vec<u8>>> {
        let result = AdvancedCache::global().get(key)?;

        if let Some(CacheValue::Binary(data_arc)) = result.data.value {
            Ok(songbird_errors::evolved_success(Some((*data_arc)).clone()).into())
        } else {
            Ok(songbird_errors::evolved_success(success(None)))
        }
    }
}
