// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Insert, lookup, and removal operations.

use songbird_types::{SongbirdError, SongbirdResult};
use std::time::{Duration, Instant};

use super::helpers::{entry_is_expired, estimate_entry_size, should_evict};
use super::types::{
    CacheEntry, CacheKey, CacheOperationResult, CacheOperationTiming, CacheValue, EvictionPolicy,
};

use super::AdvancedCache;

impl AdvancedCache {
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn set<V: Into<CacheValue>>(
        &self,
        key: CacheKey,
        value: V,
    ) -> SongbirdResult<CacheOperationResult<()>> {
        self.set_with_ttl(key, value, self.config.default_ttl)
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
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

    /// # Errors
    ///
    /// Returns an error if the operation fails.
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

    /// # Errors
    ///
    /// Returns an error if the operation fails.
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
}
