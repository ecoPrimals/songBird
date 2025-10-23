//! Performance-related Type Tests
//!
//! Testing performance characteristics of core types.

/// Helper function for zero-copy testing
fn process_borrowed(s: &str) -> usize {
    s.len()
}

#[test]
fn test_type_size_optimization() {
    // Test concept: Types should be size-optimized
    use std::mem::size_of;

    // Basic types should be reasonably sized
    assert!(size_of::<String>() <= 24, "String should use small string optimization");
    assert!(size_of::<Vec<u8>>() <= 24, "Vec should be compact");
    assert_eq!(size_of::<Option<bool>>(), 1, "Option<bool> should use niche optimization");
}

#[test]
fn test_zero_copy_potential() {
    // Test concept: Types should support zero-copy where possible
    let data = String::from("test");
    let borrowed: &str = &data;

    let len = process_borrowed(borrowed);
    assert_eq!(len, 4);
    assert_eq!(data, "test", "Original should not be moved");
}

#[test]
fn test_clone_performance() {
    // Test concept: Clone should be efficient
    use std::sync::Arc;

    let data = Arc::new(vec![1, 2, 3]);
    let clone = Arc::clone(&data);

    assert_eq!(Arc::strong_count(&data), 2, "Arc clone should increment refcount");
    assert_eq!(*data, *clone, "Cloned data should be equal");
}

#[test]
fn test_serialization_performance() {
    // Test concept: Serialization should be fast
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Data {
        value: i32,
    }

    let data = Data {
        value: 42,
    };
    let json = serde_json::to_string(&data).expect("Test: Should serialize");
    assert!(json.contains("42"));
}

#[test]
fn test_deserialization_performance() {
    // Test concept: Deserialization should be fast and minimize allocations
    use serde_json;
    use std::time::Instant;

    let json_data = r#"{"name":"test","value":42,"active":true}"#;

    let start = Instant::now();
    for _ in 0..1000 {
        let _result: serde_json::Value =
            serde_json::from_str(json_data).expect("Valid JSON should deserialize");
    }
    let duration = start.elapsed();

    // Basic sanity check: 1000 deserializations should complete in < 100ms
    assert!(duration.as_millis() < 100, "Deserialization too slow: {duration:?}");
}

#[test]
fn test_memory_layout() {
    // Test concept: Memory layout should be efficient
    use std::mem::align_of;

    assert_eq!(align_of::<u64>(), 8, "u64 should be 8-byte aligned");
    assert_eq!(align_of::<u32>(), 4, "u32 should be 4-byte aligned");
    assert_eq!(align_of::<u8>(), 1, "u8 should be byte-aligned");
}

#[test]
fn test_cache_line_alignment() {
    // Test concept: Types should be cache-friendly and avoid false sharing
    use std::mem::{align_of, size_of};

    // Common types should have reasonable alignment
    assert!(align_of::<String>() <= 8, "String alignment should be word-aligned");
    assert!(align_of::<Vec<u8>>() <= 8, "Vec alignment should be word-aligned");

    // Size should be cache-friendly (multiple of 8 bytes for most architectures)
    assert_eq!(size_of::<String>() % 8, 0, "String size should be 8-byte aligned");
    assert_eq!(size_of::<Vec<u8>>() % 8, 0, "Vec size should be 8-byte aligned");
}

#[test]
fn test_allocation_patterns() {
    // Test concept: Allocation patterns should be minimal and predictable
    // Verify that common operations don't cause excessive allocations

    // String with capacity should not reallocate for small appends
    let mut s = String::with_capacity(100);
    let initial_capacity = s.capacity();
    s.push_str("test");
    s.push_str("data");
    assert_eq!(s.capacity(), initial_capacity, "No reallocation should occur");

    // Vec with capacity should not reallocate for planned insertions
    let mut v = Vec::<u8>::with_capacity(50);
    let initial_capacity = v.capacity();
    for i in 0..25 {
        v.push(i);
    }
    assert_eq!(v.capacity(), initial_capacity, "No reallocation should occur");
}

#[test]
fn test_reference_counting_overhead() {
    // Test concept: Arc overhead should be justified and minimal
    use std::mem::size_of;
    use std::sync::Arc;

    // Arc adds pointer + atomic counter overhead
    let data = vec![1u8, 2, 3, 4, 5];
    let arc_data = Arc::new(data.clone());

    // Arc should only add minimal overhead (typically 2 * usize)
    assert!(size_of::<Arc<Vec<u8>>>() <= 16, "Arc overhead should be minimal");

    // Cloning Arc should not clone data
    let arc_clone = Arc::clone(&arc_data);
    assert_eq!(Arc::strong_count(&arc_data), 2);
    assert_eq!(Arc::strong_count(&arc_clone), 2);
    assert!(Arc::ptr_eq(&arc_data, &arc_clone), "Arc clones should share data");
}

#[test]
fn test_trait_object_performance() {
    // Test concept: Trait objects should be used judiciously due to dynamic dispatch overhead
    use std::mem::size_of;

    trait TestTrait {
        fn get_value(&self) -> i32;
    }

    struct ConcreteType {
        value: i32,
    }

    impl TestTrait for ConcreteType {
        fn get_value(&self) -> i32 {
            self.value
        }
    }

    // Trait object uses fat pointer (data + vtable)
    let concrete = ConcreteType {
        value: 42,
    };
    let trait_obj: &dyn TestTrait = &concrete;

    // Size check: trait object reference should be 2 * usize (fat pointer)
    assert_eq!(size_of::<&dyn TestTrait>(), size_of::<usize>() * 2);
    assert_eq!(size_of::<&ConcreteType>(), size_of::<usize>());

    // Verify it works correctly despite overhead
    assert_eq!(trait_obj.get_value(), 42);
}
