// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(clippy::all, reason = "test assertions and harness ergonomics")]

use std::time::Duration;

use super::*;

#[test]
fn test_zero_copy_string() {
    let static_str = ZeroCopyString::from_static("hello");
    let borrowed_str = ZeroCopyString::from_borrowed("world");
    let owned_str = ZeroCopyString::from_owned("rust".to_string());

    assert_eq!(static_str.as_str(), "hello");
    assert_eq!(borrowed_str.as_str(), "world");
    assert_eq!(owned_str.as_str(), "rust");
}

#[test]
fn test_zero_copy_circular_buffer() {
    let mut buffer: ZeroCopyCircularBuffer<i32, 4> = ZeroCopyCircularBuffer::new();

    assert!(buffer.is_empty());
    assert_eq!(ZeroCopyCircularBuffer::<i32, 4>::capacity(), 4);

    buffer.push(1);
    buffer.push(2);
    buffer.push(3);

    assert_eq!(buffer.len(), 3);
    assert_eq!(buffer.get(0), Some(&1));
    assert_eq!(buffer.get(1), Some(&2));
    assert_eq!(buffer.get(2), Some(&3));
}

#[test]
fn test_zero_copy_hashmap() {
    let mut map = ZeroCopyHashMap::new();

    map.insert("key1", 42);
    map.insert(ZeroCopyString::from_static("key2"), 84);

    assert_eq!(map.get("key1"), Some(&42));
    assert_eq!(map.get("key2"), Some(&84));
    assert_eq!(map.len(), 2);
}

#[test]
fn test_benchmark() {
    let mut bench = ZeroCopyBenchmark::new("test_benchmark");

    let result = bench.measure(|| {
        std::thread::sleep(Duration::from_nanos(100));
        42
    });

    assert_eq!(result, 42);
    assert_eq!(bench.count(), 1);
    assert!(bench.average() > Duration::from_nanos(50));
}
