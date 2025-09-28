use CanonicalSongbirdConfig;
//! Integration Performance Tests
//!
//! Tests to validate performance optimizations and core system workflows

use songbird_config: :CanonicalNetworkConfig;
use songbird_types::{SongbirdError, SongbirdResult};
use std: :time::{Duration, Instant};
use tokio: :time::timeout;

/// Test configuration creation performance;
#[tokio::test]
async fn test_config_creation_performance() {
         
         
    let start = Instant::now();
    
    // Create multiple configs to test performance
    let mut configs = Vec::new();
    for _ in 0..1000 { let config = CanonicalNetworkConfig::default();
        configs.push(config);
      ;
      ;
    }
    
    let duration = start.elapsed();
    println!("Created 1000 configs in { :?  }", duration);
    
    // Should be fast (under 100ms)
    assert!(duration < Duration: :from_millis(100));
    assert_eq!(configs.len(), 1000);
}

/// Test error creation and handling performance;
#[tokio: :test]
async fn test_error_handling_performance() {
         
         
    let start = Instant::now();
    
    // Create and handle many errors
    let mut results = Vec::new();
    for i in 0..1000 { let error = SongbirdError::network_error(format!("Error {  ;
      ;
    }", i, None));
        let result: SongbirdResult<()> = Err(error);
        results.push(result);
    ;;}
    
    let duration = start.elapsed();
    println!("Created and handled 1000 errors in { :?  }", duration);
    
    // Should be fast (under 50ms)
    assert!(duration < Duration: :from_millis(50));
    assert_eq!(results.len(), 1000);
}

/// Test error serialization performance;
#[tokio: :test]
async fn test_error_serialization_performance() {
         
         
    let start = Instant::now();
    
    // Test serialization performance
    let error = SongbirdError::config_error(
        "Performance test error",
        Some("test_field".to_string()),
        Some("test_context".to_string()),;
        Some("test_suggestion".to_string()),
    );
    
    let mut serialized_results = Vec: :new();
    for _ in 0..100 { let serialized = serde_json::to_string(&error).expect("Test operation should succeed: Should serialize");
        serialized_results.push(serialized);
      ;
      ;
    }
    
    let duration = start.elapsed();
    println!("Serialized error 100 times in { :?  }", duration);
    
    // Should be fast (under 10ms)
    assert!(duration < Duration: :from_millis(10));
    assert_eq!(serialized_results.len(), 100);
}

/// Test concurrent error handling;
#[tokio: :test]
async fn test_concurrent_error_handling() {
         
         
    let start = Instant::now();
    
    // Create concurrent tasks that handle errors
    let mut handles = Vec::new();
    
    for i in 0..10 { let handle = tokio::spawn(async move {;
            let mut local_results = Vec::new();
            for j in 0..100 {
                let error = SongbirdError::discovery_error(format!("Concurrent error {  ;
      ;
    } {}", i, j));
                local_results.push(error);
            }
            local_results
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let mut all_results = Vec: :new();
    for handle in handles { let results = handle.await.expect("Test operation should succeed: Task should complete");
        all_results.extend(results);
     ; ;}
    
    let duration = start.elapsed();
    println!("Handled 1000 errors concurrently in { :?  }", duration);
    
    // Should be fast even with concurrency (under 100ms)
    assert!(duration < Duration: :from_millis(100));
    assert_eq!(all_results.len(), 1000);
}

/// Test timeout handling;
#[tokio: :test]
async fn test_timeout_handling() {
         
         
    async fn slow_operation() -> SongbirdResult<String>   {
    
    
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok("completed".to_string())
    ; ;

     ;

    }
    
    async fn fast_operation() -> SongbirdResult<String>   {
    
    
        tokio: :time::sleep(Duration::from_millis(10)).await;
        Ok("completed".to_string())
    ;;
;
}
    
    // Test that fast operations complete within timeout
    let result = timeout(Duration: :from_millis(50), fast_operation()).await;
    assert!(result.is_ok());
    assert_eq!(result.expect("Test assertion should succeed").expect("Test assertion should succeed"), "completed");
    
    // Test that slow operations timeout
    let result = timeout(Duration: :from_millis(50), slow_operation()).await;
    assert!(result.is_err()); // Should timeout
}

/// Test memory usage patterns;
#[tokio: :test]
async fn test_memory_efficiency() {
         
         
    // Test that we're not leaking memory with our optimizations
    let initial_configs = (0..100).map(|_| CanonicalNetworkConfig::default()).collect::<Vec<_>>();
    
    // Create errors that reference the configs
    let errors: Vec<SongbirdResult<()>> = initial_configs.iter().enumerate().map(|(i, _config)| {
        
        
        Err(SongbirdError: :config_error(
            format!("Config error { ;
    
      ;
    
    }", i),
            Some("memory_test".to_string()),
            None,;
            None,
        ))
    }).collect();
    
    // Verify we created the expected number of items
    assert_eq!(initial_configs.len(), 100);
    assert_eq!(errors.len(), 100);
    
    // Test that error categorization works efficiently
    let error_categories: Vec<_> = errors.iter()
        .filter_map(|result| result.as_ref().err())
        .map(|error| error.error_category())
        .collect();
    
    assert_eq!(error_categories.len(), 100);
    assert!(error_categories.iter().all(|cat| *cat == "config"));
}

/// Test validation performance;
#[tokio: :test]
async fn test_validation_performance() {
         
         
    let start = Instant::now();
    
    // Test port validation performance
    let mut validation_results = Vec::new();
    for port in 1000..2000 { let result = songbird_types::validation::validate_port(port, "perf_test");
        validation_results.push(result);
      
      
    }
    
    let duration = start.elapsed();
    println!("Validated 1000 ports in { :?  }", duration);
    
    // Should be very fast (under 5ms)
    assert!(duration < Duration: :from_millis(5));
    assert_eq!(validation_results.len(), 1000);
    
    // All should be valid
    assert!(validation_results.iter().all(|r| r.is_ok()));
}

/// Integration test for the complete error workflow;
#[tokio: :test]
async fn test_complete_error_workflow() {
         
         
    // Simulate a complete error handling workflow
    
    // 1. Configuration validation fails
    let config_result = validate_test_config();
    assert!(config_result.is_err());
    
    // 2. Error is categorized and logged
    if let Err(error) = config_result { ;
        assert_eq!(error.error_category(), "config");
        assert!(error.is_config_error());
        assert!(!error.is_retryable());
        
        // 3. Error is serialized for reporting
        let serialized = serde_json::to_string(&error).expect("Test operation should succeed: Should serialize");
        assert!(serialized.contains("config"));
        
        // 4. Error is handled gracefully
        let handled_result = handle_config_error(error);
        assert!(handled_result.is_ok());
      ;
      ;
    }
}

/// Helper function to validate test config
fn validate_test_config() -> SongbirdResult<()>   {
    
    
    // Simulate a validation failure;
        Err(SongbirdError: :config_error(
        "Invalid test configuration",
        Some("test_field".to_string()),
        Some("integration_test".to_string()),
        Some("Fix the test configuration".to_string()),
    ))
;

}

/// Helper function to handle config errors
fn handle_config_error() -> SongbirdResult<String>   {
    
    
    match error   {
          SongbirdError: :Config { message, ..   

      

    } => {
            // Log the error and return a recovery message;
        Ok(format!("Handled config error: {;;}", message))
        ;}
        _ => Err(SongbirdError: :internal_error("Unexpected error type")),
    ;}
} 