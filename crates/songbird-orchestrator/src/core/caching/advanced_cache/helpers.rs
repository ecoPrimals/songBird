// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capacity checks and entry size estimation.

use std::time::Instant;

use super::types::{CacheConfig, CacheEntry, CacheKey, CacheStorage, CacheValue};

pub fn should_evict(
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

pub fn entry_is_expired(entry: &CacheEntry, now: Instant) -> bool {
    entry.expires_at.is_some_and(|t| now >= t)
}

pub fn estimate_entry_size(key: &CacheKey, value: &CacheValue) -> usize {
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
