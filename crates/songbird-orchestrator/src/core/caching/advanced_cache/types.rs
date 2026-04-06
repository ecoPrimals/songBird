// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cache keys, values, entries, configuration, and operation result types.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Compatibility alias used by orchestrator AI cache layers.
pub type CanonicalCacheConfig = CacheConfig;

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
pub struct CacheEntry {
    pub value: CacheValue,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub ttl: Option<Duration>,
    pub expires_at: Option<Instant>,
    pub size_bytes: usize,
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

#[derive(Debug)]
pub struct CacheStorage {
    pub data: HashMap<CacheKey, CacheEntry>,
    pub lru_queue: VecDeque<CacheKey>,
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

impl CacheStorage {
    pub fn current_size_bytes(&self) -> usize {
        self.data.values().map(|e| e.size_bytes).sum()
    }

    pub fn push_lru(&mut self, key: CacheKey) {
        self.lru_queue.push_back(key);
    }
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
