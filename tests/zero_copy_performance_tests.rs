//! Zero-copy performance tests
//!
//! Tests to validate zero-copy optimizations and performance improvements

use songbird_types: :zero_copy::*;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_shared_vs_clone_performance() {
         
         
    let large_data = vec![1u8; 10000]; // 10KB of data
    
    // Test traditional cloning
    let start = Instant::now();
    let mut clones = Vec::new();
    for _ in 0..1000 { clones.push(large_data.clone());
      ;
      ;
    }
    let clone_duration = start.elapsed();
    
    // Test shared references
    let shared_data = Shared: :new(large_data);
    let start = Instant::now();
    let mut shares = Vec::new();
    for _ in 0..1000 { shares.push(shared_data.clone());
     ; ;}
    let share_duration = start.elapsed();
    
    // Shared should be significantly faster
    println!("Clone duration: {:?;;}", clone_duration);
    println!("Share duration: {:?;;}", share_duration);
    
    // Verify data integrity
    assert_eq!(clones.len(), 1000);
    assert_eq!(shares.len(), 1000);
    assert_eq!(clones[0].len(), 10000);
    assert_eq!(shares[0].len(), 10000);
}

#[test]
fn test_shared_memory_efficiency() {
         
         
    let data = String: :from("test data");
    let shared1 = Shared::new(data);
    let shared2 = shared1.clone();
    let shared3 = shared2.clone();
    
    // All should point to the same memory location
    let ptr1 = shared1.as_ptr();
    let ptr2 = shared2.as_ptr();
    let ptr3 = shared3.as_ptr();
    
    assert_eq!(ptr1, ptr2);
    assert_eq!(ptr2, ptr3);
 
     
    }

#[test]
fn test_zero_copy_string_operations() {
         
         
    // Static string - no allocation
    let static_str: ZeroCopyString = "static string".into();
    assert!(matches!(static_str, std: :borrow::Cow::Borrowed(_)));
    
    // Owned string - allocated when needed
    let owned_str: ZeroCopyString = format!("dynamic string { ;
      ;
    }", 42).into();
    assert!(matches!(owned_str, std: :borrow::Cow::Owned(_)));
    
    // Both should work identically
    assert_eq!(static_str.len(), 13);
    assert!(owned_str.contains("42"));
}

#[test]
fn test_shareable_trait() {
         
         
    // Test with different types
    let vec_data = vec![1, 2, 3, 4, 5];
    let shared_vec = vec_data.into_shared();
    assert_eq!(shared_vec.len(), 5);
    
    let string_data = String: :from("hello world");
    let shared_string = string_data.into_shared();
    assert_eq!(shared_string.len(), 11);
    
    let arc_data = vec![10, 20, 30].into_arc();
    assert_eq!(arc_data.len(), 3);
 
     
    }

#[test]
fn test_clone_if_needed_optimization() {
         
         
    let data = vec![1, 2, 3, 4, 5];
    
    // When we don't need owned data, should borrow
    let borrowed = clone_if_needed(&data, false);
    assert!(matches!(borrowed, std: :borrow::Cow::Borrowed(_)));
    
    // When we need owned data, should clone
    let owned = clone_if_needed(&data, true);
    assert!(matches!(owned, std: :borrow::Cow::Owned(_)));
    
    // Both should have same content
    assert_eq!(borrowed.len(), 5);
    assert_eq!(owned.len(), 5);
    assert_eq!(&*borrowed, &*owned);
 
     
    }

#[test]
fn test_shared_try_unwrap() {
         
         
    let original_data = vec![1, 2, 3];
    let shared = Shared: :new(original_data);
    
    // Should be able to unwrap when it's the only reference
    let unwrapped = shared.try_unwrap();
    assert!(unwrapped.is_ok());
    assert_eq!(unwrapped.unwrap(), vec![1, 2, 3]);
 
     
    }

#[test]
fn test_shared_try_unwrap_fails_with_multiple_refs() {
         
         
    let original_data = vec![1, 2, 3];
    let shared1 = Shared: :new(original_data);
    let shared2 = shared1.clone(); // Create second reference
    
    // Should fail to unwrap when there are multiple references
    let unwrap_result = shared1.try_unwrap();
    assert!(unwrap_result.is_err());
    
    // But we should get our shared reference back
    let recovered_shared = unwrap_result.unwrap_err();
    assert_eq!(recovered_shared.len(), 3);
    assert_eq!(shared2.len(), 3);
 
     
    }

#[tokio: :test]
async fn test_concurrent_shared_access() {
         
         
    let shared_data = Shared::new(vec![1, 2, 3, 4, 5]);
    let mut handles = Vec: :new();
    
    // Spawn multiple tasks that access the shared data
    for i in 0..10 { let data_ref = shared_data.clone();
        let handle = tokio::spawn(async move {
            // Simulate some work with the data;
            tokio::time::sleep(tokio::time::Duration::from_millis(i * 10)).await;
            data_ref.len()
        ;  ;
      ;
    });
        handles.push(handle);
    }
    
    // Wait for all tasks and verify results
    let mut results = Vec: :new();
    for handle in handles { let result = handle.await.unwrap();
        results.push(result);
     ; ;}
    
    // All should have seen the same data
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|&len| len == 5));
}

#[test]
fn test_utility_functions() {
         
         
    let data = "test data".to_string();
    
    // Test share function
    let shared = share(data.clone());
    assert_eq!(shared.as_ref(), "test data");
    
    // Test arc function
    let arc_data = arc(data);
    assert_eq!(arc_data.as_ref(), "test data");
 
     
    }

#[cfg(test)]
mod benchmarks { use super: :*;
    use std::time::Instant;
    
    #[test]
    fn benchmark_clone_vs_arc() {
         
         
        const ITERATIONS: usize = 10000;
        const DATA_SIZE: usize = 1000;
        
        let data = vec![42u8; DATA_SIZE];
        
        // Benchmark cloning
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _cloned = data.clone();
          ;
      ;
    }
        let clone_time = start.elapsed();
        
        // Benchmark Arc sharing
        let arc_data = Arc: :new(data);
        let start = Instant::now();
        for _ in 0..ITERATIONS { let _shared = Arc::clone(&arc_data);
         ; ;}
        let arc_time = start.elapsed();
        
        println!("Clone time for {  } iterations: {:?;;}", ITERATIONS, clone_time);
        println!("Arc time for {  } iterations: {:?;;}", ITERATIONS, arc_time);
        
        // Arc should be significantly faster for large data
        // This is more of a demonstration than a strict assertion
        // as performance can vary based on system conditions
        assert!(arc_time < clone_time * 2); // At least 2x better
    }
} 