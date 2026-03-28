// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Advanced in-memory cache with LRU/LFU/FIFO/Random eviction and optional TTL.

mod helpers;
mod operations;
mod types;

pub use types::*;

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Instant;

use helpers::entry_is_expired;

static GLOBAL_CACHE: OnceLock<AdvancedCache> = OnceLock::new();

/// Advanced cache with pluggable eviction and byte/entry limits.
#[derive(Debug)]
pub struct AdvancedCache {
    pub(crate) storage: Arc<RwLock<CacheStorage>>,
    pub(crate) config: types::CacheConfig,
    pub(crate) stats: Arc<RwLock<CacheStatistics>>,
    pub(crate) start_time: Instant,
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
    pub const fn get_config(&self) -> &CacheConfig {
        &self.config
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

    /// # Errors
    ///
    /// Returns an error if the operation fails.
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

    /// # Errors
    ///
    /// Returns an error if the operation fails.
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

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn get_statistics(&self) -> SongbirdResult<CacheStatistics> {
        self.snapshot_statistics()
    }

    pub(crate) fn bump_stat(&self, f: impl FnOnce(&mut CacheStatistics)) -> SongbirdResult<()> {
        let mut stats = self
            .stats
            .write()
            .map_err(|e| SongbirdError::service("cache", format!("Lock error: {e}")))?;
        f(&mut stats);
        Ok(())
    }

    pub(crate) fn update_stats_snapshot(&self) -> SongbirdResult<()> {
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

    pub(crate) fn snapshot_statistics(&self) -> SongbirdResult<CacheStatistics> {
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

    pub(crate) fn evict_entry(&self, storage: &mut CacheStorage) -> SongbirdResult<()> {
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

    pub(crate) fn remove_key_from_lru(storage: &mut CacheStorage, key: &CacheKey) {
        if let Some(pos) = storage.lru_queue.iter().position(|k| k == key) {
            storage.lru_queue.remove(pos);
        }
    }
}

#[cfg(test)]
#[path = "../advanced_cache_tests.rs"]
mod tests;
