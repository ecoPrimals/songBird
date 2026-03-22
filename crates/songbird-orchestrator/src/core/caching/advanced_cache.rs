// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Advanced in-memory cache with LRU/LFU/FIFO/Random eviction and optional TTL.

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

/// Compatibility alias used by orchestrator AI cache layers.
pub type CanonicalCacheConfig = CacheConfig;

static GLOBAL_CACHE: OnceLock<AdvancedCache> = OnceLock::new();

/// Advanced cache with pluggable eviction and byte/entry limits.
#[derive(Debug)]
pub struct AdvancedCache {
    storage: Arc<RwLock<CacheStorage>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStatistics>>,
    start_time: Instant,
}

#[derive(Debug)]
struct CacheStorage {
    data: HashMap<CacheKey, CacheEntry>,
    lru_queue: VecDeque<CacheKey>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub max_size_bytes: usize,
    pub default_ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
    pub cleanup_interval: Duration,
    pub enable_compression: bool,
    pub enable_persistence: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_size_bytes: 100 * 1024 * 1024,
            default_ttl: Some(Duration::from_secs(3600)),
            eviction_policy: EvictionPolicy::Lru,
            cleanup_interval: Duration::from_secs(60),
            enable_compression: false,
            enable_persistence: false,
        }
    }
}

/// Eviction policy when the cache is over capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    Lru,
    Lfu,
    Fifo,
    Random,
    /// Do not evict based on size; only `cleanup_expired` removes entries.
    TtlOnly,
}

/// Cache key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    String(Arc<str>),
    Binary(Vec<u8>),
    Namespaced {
        namespace: Arc<str>,
        key: Arc<str>,
    },
    Numeric(u64),
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: CacheValue,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    ttl: Option<Duration>,
    expires_at: Option<Instant>,
    size_bytes: usize,
}

/// Cached value
#[derive(Debug, Clone)]
pub enum CacheValue {
    String(Arc<str>),
    Binary(Arc<Vec<u8>>),
    Json(Arc<serde_json::Value>),
    Serialized {
        data: Arc<Vec<u8>>,
        type_hint: String,
    },
    Reference {
        location: String,
        checksum: Option<String>,
    },
}

/// Snapshot statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_operations: u64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub expirations: u64,
    pub current_entries: usize,
    pub current_size_bytes: usize,
    pub hit_ratio: f64,
    pub avg_access_time_us: f64,
    pub memory_efficiency: f64,
    pub uptime_seconds: u64,
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

/// Result of a cache operation including timing and stats snapshot.
#[derive(Debug, Clone)]
pub struct CacheOperationResult<T> {
    pub value: T,
    pub timing: CacheOperationTiming,
    pub stats: CacheStatistics,
}

#[derive(Debug, Clone)]
pub struct CacheOperationTiming {
    pub total_duration: Duration,
    pub lock_duration: Duration,
    pub processing_duration: Duration,
}

impl AdvancedCache {
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    #[must_use]
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            storage: Arc::new(RwLock::new(CacheStorage {
                data: HashMap::new(),
                lru_queue: VecDeque::new(),
            })),
            config,
            stats: Arc::new(RwLock::new(CacheStatistics::default())),
            start_time: Instant::now(),
        }
    }

    #[must_use]
    pub fn global() -> &'static Self {
        GLOBAL_CACHE.get_or_init(Self::new)
    }

    #[must_use]
    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }

    pub fn set<V: Into<CacheValue>>(
        &self,
        key: CacheKey,
        value: V,
    ) -> SongbirdResult<CacheOperationResult<()>> {
        self.set_with_ttl(key, value, self.config.default_ttl)
    }

    pub fn set_with_ttl<V: Into<CacheValue>>(
        &self,
        key: CacheKey,
        value: V,
        ttl: Option<Duration>,
    ) -> SongbirdResult<CacheOperationResult<()>> {
        let start = Instant::now();
        let lock_start = Instant::now();
        let cache_value = value.into();
        let entry_size = estimate_entry_size(&key, &cache_value);
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        let lock_duration = lock_start.elapsed();
        let processing_start = Instant::now();
        let now = Instant::now();

        while should_evict(&self.config, &storage, entry_size, &key)
            && self.config.eviction_policy != EvictionPolicy::TtlOnly
        {
            if storage.data.is_empty() {
                break;
            }
            self.evict_entry(&mut storage)?;
        }

        if self.config.eviction_policy != EvictionPolicy::TtlOnly
            && should_evict(&self.config, &storage, entry_size, &key)
        {
            return Err(SongbirdError::service(
                "cache",
                "unable to satisfy size/entry limits after eviction attempts",
            ));
        }

        if self.config.eviction_policy == EvictionPolicy::TtlOnly
            && (storage.data.len() >= self.config.max_entries
                || storage.current_size_bytes() + entry_size > self.config.max_size_bytes)
            && !storage.data.contains_key(&key)
        {
            return Err(SongbirdError::service(
                "cache",
                "cache at capacity under TTLOnly policy — use cleanup_expired or raise limits",
            ));
        }

        let expires_at = ttl.map(|t| now + t);
        let entry = CacheEntry {
            value: cache_value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
            expires_at,
            size_bytes: entry_size,
        };

        if storage.data.contains_key(&key) {
            Self::remove_key_from_lru(&mut storage, &key);
        }
        storage.data.insert(key.clone(), entry);
        storage.push_lru(key);

        let processing_duration = processing_start.elapsed();
        drop(storage);

        self.update_stats_snapshot()?;
        let total_duration = start.elapsed();
        let stats = self.snapshot_statistics()?;
        Ok(CacheOperationResult {
            value: (),
            timing: CacheOperationTiming {
                total_duration,
                lock_duration,
                processing_duration,
            },
            stats,
        })
    }

    pub fn get(&self, key: &CacheKey) -> SongbirdResult<CacheOperationResult<Option<CacheValue>>> {
        let start = Instant::now();
        let lock_start = Instant::now();
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        let lock_duration = lock_start.elapsed();
        let processing_start = Instant::now();
        let now = Instant::now();

        let expired = storage.data.get(key).is_some_and(|e| entry_is_expired(e, now));

        let result = if expired {
            if let Some(entry) = storage.data.remove(key) {
                Self::remove_key_from_lru(&mut storage, key);
                self.bump_stat(|s| {
                    s.expirations += 1;
                    s.misses += 1;
                    s.total_operations += 1;
                })?;
                let _ = entry;
            }
            None
        } else if let Some(entry) = storage.data.get_mut(key) {
            entry.last_accessed = now;
            entry.access_count = entry.access_count.saturating_add(1);
            let v = entry.value.clone();
            Self::remove_key_from_lru(&mut storage, key);
            storage.push_lru(key.clone());
            self.bump_stat(|s| {
                s.hits += 1;
                s.total_operations += 1;
            })?;
            Some(v)
        } else {
            self.bump_stat(|s| {
                s.misses += 1;
                s.total_operations += 1;
            })?;
            None
        };

        let processing_duration = processing_start.elapsed();
        drop(storage);

        self.update_stats_snapshot()?;
        let total_duration = start.elapsed();
        let stats = self.snapshot_statistics()?;
        Ok(CacheOperationResult {
            value: result,
            timing: CacheOperationTiming {
                total_duration,
                lock_duration,
                processing_duration,
            },
            stats,
        })
    }

    pub fn remove(
        &self,
        key: &CacheKey,
    ) -> SongbirdResult<CacheOperationResult<Option<CacheValue>>> {
        let start = Instant::now();
        let lock_start = Instant::now();
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        let lock_duration = lock_start.elapsed();
        let processing_start = Instant::now();

        let removed = if let Some(entry) = storage.data.remove(key) {
            Self::remove_key_from_lru(&mut storage, key);
            Some(entry.value)
        } else {
            None
        };

        let processing_duration = processing_start.elapsed();
        drop(storage);

        self.bump_stat(|s| s.total_operations += 1)?;
        self.update_stats_snapshot()?;
        let total_duration = start.elapsed();
        let stats = self.snapshot_statistics()?;
        Ok(CacheOperationResult {
            value: removed,
            timing: CacheOperationTiming {
                total_duration,
                lock_duration,
                processing_duration,
            },
            stats,
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.storage.read().map(|s| s.data.len()).unwrap_or(0)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.storage.read().map(|s| s.current_size_bytes()).unwrap_or(0)
    }

    pub fn clear(&self) -> SongbirdResult<()> {
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        storage.data.clear();
        storage.lru_queue.clear();
        drop(storage);
        self.bump_stat(|s| {
            s.current_entries = 0;
            s.current_size_bytes = 0;
        })?;
        Ok(())
    }

    pub fn cleanup_expired(&self) -> SongbirdResult<usize> {
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        let now = Instant::now();
        let keys: Vec<CacheKey> = storage
            .data
            .iter()
            .filter(|(_, e)| entry_is_expired(e, now))
            .map(|(k, _)| k.clone())
            .collect();
        let mut removed = 0usize;
        for key in keys {
            if let Some(entry) = storage.data.remove(&key) {
                Self::remove_key_from_lru(&mut storage, &key);
                let _ = entry;
                removed += 1;
            }
        }
        drop(storage);
        self.bump_stat(|s| {
            s.expirations += removed as u64;
        })?;
        self.update_stats_snapshot()?;
        Ok(removed)
    }

    pub fn get_statistics(&self) -> SongbirdResult<CacheStatistics> {
        self.snapshot_statistics()
    }

    fn bump_stat(&self, f: impl FnOnce(&mut CacheStatistics)) -> SongbirdResult<()> {
        let mut stats = self
            .stats
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        f(&mut stats);
        Ok(())
    }

    fn update_stats_snapshot(&self) -> SongbirdResult<()> {
        let entries = self.len();
        let bytes = self.size_bytes();
        let mut stats = self
            .stats
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        stats.current_entries = entries;
        stats.current_size_bytes = bytes;
        if stats.total_operations > 0 {
            stats.hit_ratio = (stats.hits as f64 / stats.total_operations as f64) * 100.0;
        } else {
            stats.hit_ratio = 0.0;
        }
        Ok(())
    }

    fn snapshot_statistics(&self) -> SongbirdResult<CacheStatistics> {
        let stats = self
            .stats
            .read()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        let mut s = stats.clone();
        s.uptime_seconds = self.start_time.elapsed().as_secs();
        s.current_entries = self.len();
        s.current_size_bytes = self.size_bytes();
        Ok(s)
    }

    fn evict_entry(&self, storage: &mut CacheStorage) -> SongbirdResult<()> {
        match self.config.eviction_policy {
            EvictionPolicy::Lru => {
                if let Some(k) = storage.lru_queue.pop_front()
                    && storage.data.remove(&k).is_some()
                {
                    self.bump_stat(|s| s.evictions += 1)?;
                }
            }
            EvictionPolicy::Lfu => {
                let key =
                    storage.data.iter().min_by_key(|(_, e)| e.access_count).map(|(k, _)| k.clone());
                if let Some(k) = key
                    && let Some(e) = storage.data.remove(&k)
                {
                    Self::remove_key_from_lru(storage, &k);
                    let _ = e;
                    self.bump_stat(|s| s.evictions += 1)?;
                }
            }
            EvictionPolicy::Fifo => {
                let key =
                    storage.data.iter().min_by_key(|(_, e)| e.created_at).map(|(k, _)| k.clone());
                if let Some(k) = key
                    && let Some(e) = storage.data.remove(&k)
                {
                    Self::remove_key_from_lru(storage, &k);
                    let _ = e;
                    self.bump_stat(|s| s.evictions += 1)?;
                }
            }
            EvictionPolicy::Random => {
                let key = storage.data.keys().next().cloned();
                if let Some(k) = key
                    && let Some(e) = storage.data.remove(&k)
                {
                    Self::remove_key_from_lru(storage, &k);
                    let _ = e;
                    self.bump_stat(|s| s.evictions += 1)?;
                }
            }
            EvictionPolicy::TtlOnly => {}
        }
        Ok(())
    }

    fn remove_key_from_lru(storage: &mut CacheStorage, key: &CacheKey) {
        if let Some(pos) = storage.lru_queue.iter().position(|k| k == key) {
            storage.lru_queue.remove(pos);
        }
    }
}

impl CacheStorage {
    fn current_size_bytes(&self) -> usize {
        self.data.values().map(|e| e.size_bytes).sum()
    }

    fn push_lru(&mut self, key: CacheKey) {
        self.lru_queue.push_back(key);
    }
}

fn should_evict(
    config: &CacheConfig,
    storage: &CacheStorage,
    new_entry_size: usize,
    new_key: &CacheKey,
) -> bool {
    let projected_entries = if storage.data.contains_key(new_key) {
        storage.data.len()
    } else {
        storage.data.len().saturating_add(1)
    };
    let projected_bytes = if let Some(old) = storage.data.get(new_key) {
        storage.current_size_bytes().saturating_sub(old.size_bytes) + new_entry_size
    } else {
        storage.current_size_bytes() + new_entry_size
    };
    projected_entries > config.max_entries || projected_bytes > config.max_size_bytes
}

fn entry_is_expired(entry: &CacheEntry, now: Instant) -> bool {
    entry.expires_at.is_some_and(|t| now >= t)
}

fn estimate_entry_size(key: &CacheKey, value: &CacheValue) -> usize {
    let key_size = match key {
        CacheKey::String(s) => s.len(),
        CacheKey::Binary(b) => b.len(),
        CacheKey::Namespaced {
            namespace,
            key,
        } => namespace.len() + key.len(),
        CacheKey::Numeric(_) => 8,
    };
    let value_size = match value {
        CacheValue::String(s) => s.len(),
        CacheValue::Binary(b) => b.len(),
        CacheValue::Json(j) => j.to_string().len(),
        CacheValue::Serialized {
            data,
            ..
        } => data.len(),
        CacheValue::Reference {
            location,
            ..
        } => location.len(),
    };
    key_size + value_size + 64
}

impl From<String> for CacheValue {
    fn from(s: String) -> Self {
        Self::String(Arc::from(s))
    }
}

impl From<&str> for CacheValue {
    fn from(s: &str) -> Self {
        Self::String(Arc::from(s))
    }
}

impl From<Vec<u8>> for CacheValue {
    fn from(data: Vec<u8>) -> Self {
        Self::Binary(Arc::new(data))
    }
}

impl From<serde_json::Value> for CacheValue {
    fn from(json: serde_json::Value) -> Self {
        Self::Json(Arc::new(json))
    }
}

impl From<String> for CacheKey {
    fn from(s: String) -> Self {
        Self::String(Arc::from(s))
    }
}

impl From<&str> for CacheKey {
    fn from(s: &str) -> Self {
        Self::String(Arc::from(s))
    }
}

impl From<u64> for CacheKey {
    fn from(n: u64) -> Self {
        Self::Numeric(n)
    }
}

impl From<Vec<u8>> for CacheKey {
    fn from(data: Vec<u8>) -> Self {
        Self::Binary(data)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    fn tiny_config() -> CacheConfig {
        CacheConfig {
            max_entries: 3,
            max_size_bytes: 1_000_000,
            default_ttl: None,
            eviction_policy: EvictionPolicy::Lru,
            cleanup_interval: StdDuration::from_secs(1),
            enable_compression: false,
            enable_persistence: false,
        }
    }

    #[test]
    fn set_and_get_string_happy_path() {
        let c = AdvancedCache::with_config(tiny_config());
        c.set(CacheKey::from("k"), CacheValue::from("v")).unwrap();
        let got = c.get(&CacheKey::from("k")).unwrap().value;
        assert!(matches!(got, Some(CacheValue::String(ref s)) if s.as_ref() == "v"));
    }

    #[test]
    fn get_missing_returns_none() {
        let c = AdvancedCache::with_config(tiny_config());
        let got = c.get(&CacheKey::from("nope")).unwrap().value;
        assert!(got.is_none());
    }

    #[test]
    fn remove_existing_returns_value() {
        let c = AdvancedCache::with_config(tiny_config());
        c.set(CacheKey::from("a"), "1").unwrap();
        let r = c.remove(&CacheKey::from("a")).unwrap().value;
        assert!(matches!(r, Some(CacheValue::String(_))));
        assert!(c.get(&CacheKey::from("a")).unwrap().value.is_none());
    }

    #[test]
    fn clear_empties_cache() {
        let c = AdvancedCache::with_config(tiny_config());
        c.set(CacheKey::from("x"), "y").unwrap();
        c.clear().unwrap();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn len_and_size_bytes_track_entries() {
        let c = AdvancedCache::with_config(tiny_config());
        c.set(CacheKey::from("a"), "aaa").unwrap();
        c.set(CacheKey::from("b"), "bbb").unwrap();
        assert_eq!(c.len(), 2);
        assert!(c.size_bytes() > 0);
    }

    #[test]
    fn ttl_expires_on_get() {
        let mut cfg = tiny_config();
        cfg.default_ttl = None;
        let c = AdvancedCache::with_config(cfg);
        c.set_with_ttl(CacheKey::from("t"), "v", Some(Duration::from_millis(1))).unwrap();
        thread::sleep(StdDuration::from_millis(20));
        let got = c.get(&CacheKey::from("t")).unwrap().value;
        assert!(got.is_none());
    }

    #[test]
    fn cleanup_expired_removes_stale_entries() {
        let mut cfg = tiny_config();
        cfg.default_ttl = None;
        let c = AdvancedCache::with_config(cfg);
        c.set_with_ttl(CacheKey::from("e"), "v", Some(Duration::from_millis(1))).unwrap();
        thread::sleep(StdDuration::from_millis(15));
        let n = c.cleanup_expired().unwrap();
        assert_eq!(n, 1);
        assert!(c.is_empty());
    }

    #[test]
    fn lru_evicts_oldest_when_max_entries_exceeded() {
        let mut cfg = tiny_config();
        cfg.max_entries = 2;
        cfg.eviction_policy = EvictionPolicy::Lru;
        let c = AdvancedCache::with_config(cfg);
        c.set(CacheKey::from("a"), "1").unwrap();
        c.set(CacheKey::from("b"), "2").unwrap();
        c.set(CacheKey::from("c"), "3").unwrap();
        assert_eq!(c.len(), 2);
        assert!(c.get(&CacheKey::from("a")).unwrap().value.is_none());
        assert!(c.get(&CacheKey::from("c")).unwrap().value.is_some());
    }

    #[test]
    fn lfu_evicts_lowest_access_count() {
        let mut cfg = tiny_config();
        cfg.max_entries = 2;
        cfg.eviction_policy = EvictionPolicy::Lfu;
        let c = AdvancedCache::with_config(cfg);
        c.set(CacheKey::from("a"), "1").unwrap();
        c.set(CacheKey::from("b"), "2").unwrap();
        let _ = c.get(&CacheKey::from("a")).unwrap();
        let _ = c.get(&CacheKey::from("a")).unwrap();
        c.set(CacheKey::from("c"), "3").unwrap();
        assert!(c.get(&CacheKey::from("b")).unwrap().value.is_none());
        assert!(c.get(&CacheKey::from("a")).unwrap().value.is_some());
    }

    #[test]
    fn fifo_evicts_oldest_created() {
        let mut cfg = tiny_config();
        cfg.max_entries = 2;
        cfg.eviction_policy = EvictionPolicy::Fifo;
        let c = AdvancedCache::with_config(cfg);
        c.set(CacheKey::from("first"), "1").unwrap();
        thread::sleep(StdDuration::from_millis(5));
        c.set(CacheKey::from("second"), "2").unwrap();
        c.set(CacheKey::from("third"), "3").unwrap();
        assert!(c.get(&CacheKey::from("first")).unwrap().value.is_none());
    }

    #[test]
    fn ttl_only_errors_when_full_and_new_key() {
        let mut cfg = tiny_config();
        cfg.max_entries = 1;
        cfg.eviction_policy = EvictionPolicy::TtlOnly;
        let c = AdvancedCache::with_config(cfg);
        c.set(CacheKey::from("only"), "x").unwrap();
        let err = c.set(CacheKey::from("other"), "y");
        assert!(err.is_err());
    }

    #[test]
    fn ttl_only_allows_replace_same_key() {
        let mut cfg = tiny_config();
        cfg.max_entries = 1;
        cfg.eviction_policy = EvictionPolicy::TtlOnly;
        let c = AdvancedCache::with_config(cfg);
        c.set(CacheKey::from("k"), "a").unwrap();
        c.set(CacheKey::from("k"), "b").unwrap();
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn numeric_and_binary_keys_round_trip() {
        let c = AdvancedCache::with_config(tiny_config());
        let k1 = CacheKey::Numeric(42);
        c.set(k1.clone(), "n").unwrap();
        assert!(c.get(&k1).unwrap().value.is_some());

        let k2 = CacheKey::Binary(vec![1, 2, 3]);
        c.set(k2.clone(), vec![9u8]).unwrap();
        assert!(c.get(&k2).unwrap().value.is_some());
    }

    #[test]
    fn namespaced_key_hashing() {
        let c = AdvancedCache::with_config(tiny_config());
        let k = CacheKey::Namespaced {
            namespace: Arc::from("ns"),
            key: Arc::from("item"),
        };
        c.set(k.clone(), "v").unwrap();
        assert!(c.get(&k).unwrap().value.is_some());
    }

    #[test]
    fn json_value_round_trip() {
        let c = AdvancedCache::with_config(tiny_config());
        let v = serde_json::json!({"x": [1,2,3]});
        c.set(CacheKey::from("j"), v).unwrap();
        let got = c.get(&CacheKey::from("j")).unwrap().value;
        assert!(matches!(got, Some(CacheValue::Json(_))));
    }

    #[test]
    fn statistics_reflect_hits_and_misses() {
        let c = AdvancedCache::with_config(tiny_config());
        c.set(CacheKey::from("h"), "1").unwrap();
        let _ = c.get(&CacheKey::from("h")).unwrap();
        let _ = c.get(&CacheKey::from("missing")).unwrap();
        let s = c.get_statistics().unwrap();
        assert!(s.hits >= 1);
        assert!(s.misses >= 1);
    }

    #[test]
    fn replace_key_updates_value() {
        let c = AdvancedCache::with_config(tiny_config());
        c.set(CacheKey::from("r"), "old").unwrap();
        c.set(CacheKey::from("r"), "new").unwrap();
        let got = c.get(&CacheKey::from("r")).unwrap().value;
        assert!(matches!(got, Some(CacheValue::String(s)) if s.as_ref() == "new"));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn zero_max_entries_edge_config_still_allows_logic() {
        let mut cfg = tiny_config();
        cfg.max_entries = 0;
        cfg.eviction_policy = EvictionPolicy::TtlOnly;
        let c = AdvancedCache::with_config(cfg);
        assert!(c.set(CacheKey::from("x"), "y").is_err());
    }

    #[test]
    fn reference_value_variant_size_nonzero() {
        let c = AdvancedCache::with_config(tiny_config());
        let v = CacheValue::Reference {
            location: "http://example.com/blob".to_string(),
            checksum: None,
        };
        c.set(CacheKey::from("ref"), v).unwrap();
        assert!(c.size_bytes() > 0);
    }

    #[test]
    fn global_returns_singleton() {
        let a = std::ptr::from_ref(AdvancedCache::global());
        let b = std::ptr::from_ref(AdvancedCache::global());
        assert_eq!(a, b);
    }
}
