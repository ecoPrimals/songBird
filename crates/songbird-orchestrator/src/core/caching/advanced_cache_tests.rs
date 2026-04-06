// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    c.set_with_ttl(CacheKey::from("t"), "v", Some(StdDuration::from_millis(1))).unwrap();
    // AdvancedCache TTL uses `std::time::Instant`; wall-clock delay required for expiry.
    thread::sleep(StdDuration::from_millis(20));
    let got = c.get(&CacheKey::from("t")).unwrap().value;
    assert!(got.is_none());
}

#[test]
fn cleanup_expired_removes_stale_entries() {
    let mut cfg = tiny_config();
    cfg.default_ttl = None;
    let c = AdvancedCache::with_config(cfg);
    c.set_with_ttl(CacheKey::from("e"), "v", Some(StdDuration::from_millis(1))).unwrap();
    // TTL/expiry checks use `std::time::Instant`, not Tokio's mock clock.
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
    // FIFO ordering uses `Instant::now()` per entry; brief wall delay differentiates creation time.
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
