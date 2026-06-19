// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};
    use std::sync::Arc;
    use std::time::Duration;

    fn make_config(max_entries: usize, max_size_bytes: usize) -> CacheConfig {
        CacheConfig {
            max_entries,
            max_size_bytes,
            default_ttl: None,
            eviction_policy: super::super::types::EvictionPolicy::Lru,
            cleanup_interval: Duration::from_secs(60),
            enable_compression: false,
            enable_persistence: false,
        }
    }

    fn make_entry(size_bytes: usize) -> CacheEntry {
        CacheEntry {
            value: CacheValue::String("x".into()),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            ttl: None,
            expires_at: None,
            size_bytes,
        }
    }

    fn empty_storage() -> CacheStorage {
        CacheStorage {
            data: HashMap::new(),
            lru_queue: VecDeque::new(),
        }
    }

    #[test]
    fn should_evict_empty_storage_under_limits() {
        let config = make_config(10, 1024);
        let storage = empty_storage();
        let key = CacheKey::String("new".into());
        assert!(!should_evict(&config, &storage, 100, &key));
    }

    #[test]
    fn should_evict_entry_count_at_limit() {
        let config = make_config(2, 1_000_000);
        let mut storage = empty_storage();
        storage.data.insert(CacheKey::String("a".into()), make_entry(10));
        storage.data.insert(CacheKey::String("b".into()), make_entry(10));
        let new_key = CacheKey::String("c".into());
        assert!(should_evict(&config, &storage, 10, &new_key));
    }

    #[test]
    fn should_evict_update_existing_key_no_count_increase() {
        let config = make_config(2, 1_000_000);
        let mut storage = empty_storage();
        let key_a = CacheKey::String("a".into());
        let key_b = CacheKey::String("b".into());
        storage.data.insert(key_a.clone(), make_entry(10));
        storage.data.insert(key_b, make_entry(10));
        assert!(!should_evict(&config, &storage, 10, &key_a));
    }

    #[test]
    fn should_evict_byte_limit_exceeded() {
        let config = make_config(100, 50);
        let mut storage = empty_storage();
        storage.data.insert(CacheKey::String("a".into()), make_entry(40));
        let new_key = CacheKey::String("b".into());
        assert!(should_evict(&config, &storage, 20, &new_key));
    }

    #[test]
    fn should_evict_byte_limit_update_replaces_old_size() {
        let config = make_config(100, 50);
        let mut storage = empty_storage();
        let key = CacheKey::String("a".into());
        storage.data.insert(key.clone(), make_entry(40));
        assert!(!should_evict(&config, &storage, 30, &key));
    }

    #[test]
    fn should_evict_exactly_at_limit_does_not_trigger() {
        let config = make_config(2, 100);
        let mut storage = empty_storage();
        storage.data.insert(CacheKey::String("a".into()), make_entry(50));
        let new_key = CacheKey::String("b".into());
        assert!(!should_evict(&config, &storage, 50, &new_key));
    }

    #[test]
    fn entry_is_expired_no_expiry() {
        let entry = CacheEntry {
            value: CacheValue::String("v".into()),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            ttl: None,
            expires_at: None,
            size_bytes: 10,
        };
        assert!(!entry_is_expired(&entry, Instant::now()));
    }

    #[test]
    fn entry_is_expired_future_expiry() {
        let entry = CacheEntry {
            value: CacheValue::String("v".into()),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            ttl: Some(Duration::from_secs(3600)),
            expires_at: Some(Instant::now() + Duration::from_secs(3600)),
            size_bytes: 10,
        };
        assert!(!entry_is_expired(&entry, Instant::now()));
    }

    #[test]
    fn entry_is_expired_past_expiry() {
        let past = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        let entry = CacheEntry {
            value: CacheValue::String("v".into()),
            created_at: past,
            last_accessed: past,
            access_count: 0,
            ttl: Some(Duration::from_secs(5)),
            expires_at: Some(past + Duration::from_secs(5)),
            size_bytes: 10,
        };
        assert!(entry_is_expired(&entry, Instant::now()));
    }

    #[test]
    fn estimate_entry_size_string_key_string_value() {
        let key = CacheKey::String("hello".into());
        let value = CacheValue::String("world".into());
        assert_eq!(estimate_entry_size(&key, &value), 5 + 5 + 64);
    }

    #[test]
    fn estimate_entry_size_binary_key() {
        let key = CacheKey::Binary(vec![1, 2, 3, 4]);
        let value = CacheValue::String("x".into());
        assert_eq!(estimate_entry_size(&key, &value), 4 + 1 + 64);
    }

    #[test]
    fn estimate_entry_size_namespaced_key() {
        let key = CacheKey::Namespaced {
            namespace: "ns".into(),
            key: "mykey".into(),
        };
        let value = CacheValue::String("val".into());
        assert_eq!(estimate_entry_size(&key, &value), 2 + 5 + 3 + 64);
    }

    #[test]
    fn estimate_entry_size_numeric_key() {
        let key = CacheKey::Numeric(42);
        let value = CacheValue::String("".into());
        assert_eq!(estimate_entry_size(&key, &value), 72);
    }

    #[test]
    fn estimate_entry_size_binary_value() {
        let key = CacheKey::Numeric(0);
        let value = CacheValue::Binary(Arc::new(vec![0u8; 100]));
        assert_eq!(estimate_entry_size(&key, &value), 8 + 100 + 64);
    }

    #[test]
    fn estimate_entry_size_json_value() {
        let key = CacheKey::Numeric(0);
        let json = serde_json::json!({"a": 1});
        let value = CacheValue::Json(Arc::new(json.clone()));
        let expected_json_len = json.to_string().len();
        assert_eq!(estimate_entry_size(&key, &value), 8 + expected_json_len + 64);
    }

    #[test]
    fn estimate_entry_size_serialized_value() {
        let key = CacheKey::String("k".into());
        let value = CacheValue::Serialized {
            data: Arc::new(vec![1, 2, 3]),
            type_hint: String::from("proto"),
        };
        assert_eq!(estimate_entry_size(&key, &value), 1 + 3 + 64);
    }

    #[test]
    fn estimate_entry_size_reference_value() {
        let key = CacheKey::String("r".into());
        let value = CacheValue::Reference {
            location: String::from("/path/to/data"),
            checksum: Some(String::from("abc123")),
        };
        assert_eq!(estimate_entry_size(&key, &value), 1 + 13 + 64);
    }
}
