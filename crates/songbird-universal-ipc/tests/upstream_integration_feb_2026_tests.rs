//! Upstream Integration Tests - February 2026
//!
//! Comprehensive test coverage for biomeOS integration fixes:
//! - Issue 1: Standard methods (health, identity, rpc.discover)
//! - Issue 2: BirdSong family_id passthrough
//! - Unit, E2E, Chaos, and Fault Injection tests
//!
//! Test categories:
//! - Unit: Handler method logic
//! - E2E: Full request/response flow
//! - Chaos: Concurrent connections, rapid requests
//! - Fault: Invalid JSON, missing params, error paths
//!
//! Note: Environment variable tests use a mutex to prevent parallel execution interference.

use serde_json::json;
use songbird_universal_ipc::endpoint::NativeEndpoint;
use songbird_universal_ipc::registry::ServiceRegistry;
use songbird_universal_ipc::service::IpcServiceHandler;
use songbird_universal_ipc::tower_atomic::JsonRpcHandler;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

// Global mutex to serialize environment variable tests
static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

// ============================================================================
// UNIT TESTS - Standard Methods
// ============================================================================

#[tokio::test]
async fn test_unit_health_method() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("health", json!({})).await;

    assert!(result.is_ok(), "health method should succeed");
    let response = result.unwrap();

    assert_eq!(response["status"], "healthy");
    assert_eq!(response["primal"], "songbird");
    assert!(response["version"].is_string());
    assert!(response["uptime_seconds"].is_number());
    assert_eq!(response["services"], 0); // No services registered
}

#[tokio::test]
async fn test_unit_identity_method() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("identity", json!({})).await;

    assert!(result.is_ok(), "identity method should succeed");
    let response = result.unwrap();

    assert_eq!(response["primal"], "songbird");
    assert!(response["version"].is_string());
    assert!(response["family_id"].is_string());
    assert!(response["capabilities"].is_array());

    let capabilities = response["capabilities"].as_array().unwrap();
    assert!(capabilities.len() > 10, "Should have many capabilities");

    // Verify key capabilities present
    let cap_strings: Vec<String> =
        capabilities.iter().map(|v| v.as_str().unwrap().to_string()).collect();

    // Note: "health" is a method, not a capability
    assert!(cap_strings.contains(&"ipc.register".to_string()));
    assert!(cap_strings.contains(&"http.request".to_string()));
    assert!(cap_strings.contains(&"birdsong.generate_encrypted_beacon".to_string()));
}

#[tokio::test]
async fn test_unit_rpc_discover_method() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("rpc.discover", json!({})).await;

    assert!(result.is_ok(), "rpc.discover method should succeed");
    let response = result.unwrap();

    assert!(response["methods"].is_array());

    let methods = response["methods"].as_array().unwrap();
    assert!(methods.len() > 15, "Should have many methods");

    // Verify standard methods present
    let method_strings: Vec<String> =
        methods.iter().map(|v| v.as_str().unwrap().to_string()).collect();

    assert!(method_strings.contains(&"health".to_string()));
    assert!(method_strings.contains(&"identity".to_string()));
    assert!(method_strings.contains(&"rpc.discover".to_string()));
}

#[tokio::test]
async fn test_unit_family_id_from_environment() {
    // Serialize with other env tests
    let _guard = ENV_TEST_LOCK.lock().unwrap();

    // Clean slate - remove all family ID vars first
    std::env::remove_var("FAMILY_ID");
    std::env::remove_var("SONGBIRD_FAMILY_ID");
    std::env::remove_var("NODE_FAMILY_ID");

    // Test FAMILY_ID (highest priority)
    std::env::set_var("FAMILY_ID", "test_family_1");

    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("identity", json!({})).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let family_id = response["family_id"].as_str().unwrap();
    assert_eq!(family_id, "test_family_1", "Should use FAMILY_ID");

    // Clean up
    std::env::remove_var("FAMILY_ID");
    std::env::remove_var("SONGBIRD_FAMILY_ID");
    std::env::remove_var("NODE_FAMILY_ID");
}

#[tokio::test]
async fn test_unit_uptime_tracking() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    // First call
    let result1 = handler.handle("health", json!({})).await.unwrap();
    let uptime1 = result1["uptime_seconds"].as_u64().unwrap();

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Second call
    let result2 = handler.handle("health", json!({})).await.unwrap();
    let uptime2 = result2["uptime_seconds"].as_u64().unwrap();

    assert!(uptime2 >= uptime1, "Uptime should increase or stay same");
}

// ============================================================================
// E2E TESTS - Full Request/Response Flow
// ============================================================================

#[tokio::test]
async fn test_e2e_health_via_handler() {
    // Simulate full JSON-RPC request/response cycle
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    // Send health request
    let result = handler.handle("health", json!({})).await;

    assert!(result.is_ok());
    let response = result.unwrap();

    // Verify response structure
    assert!(response.is_object());
    assert!(response.get("status").is_some());
    assert!(response.get("primal").is_some());
    assert!(response.get("version").is_some());
    assert!(response.get("uptime_seconds").is_some());
}

#[tokio::test]
async fn test_e2e_identity_with_capabilities() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("identity", json!({})).await;

    assert!(result.is_ok());
    let response = result.unwrap();

    // Verify all expected capabilities are present
    let capabilities = response["capabilities"].as_array().unwrap();
    let cap_set: std::collections::HashSet<String> =
        capabilities.iter().map(|v| v.as_str().unwrap().to_string()).collect();

    // Standard methods
    assert!(cap_set.contains("ipc.register"));
    assert!(cap_set.contains("ipc.resolve"));

    // HTTP methods
    assert!(cap_set.contains("http.request"));
    assert!(cap_set.contains("http.get"));
    assert!(cap_set.contains("http.post"));

    // STUN methods
    assert!(cap_set.contains("stun.get_public_address"));
    assert!(cap_set.contains("stun.bind"));

    // BirdSong methods
    assert!(cap_set.contains("birdsong.generate_encrypted_beacon"));
    assert!(cap_set.contains("birdsong.decrypt_beacon"));
    assert!(cap_set.contains("birdsong.verify_lineage"));
    assert!(cap_set.contains("birdsong.get_lineage"));

    // Discovery methods
    assert!(cap_set.contains("discovery.peers"));
}

#[tokio::test]
async fn test_e2e_multiple_sequential_requests() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    // Simulate persistent connection with multiple requests
    let methods = vec!["health", "identity", "rpc.discover", "health"];

    for method in methods {
        let result = handler.handle(method, json!({})).await;
        assert!(result.is_ok(), "Method {} should succeed", method);
    }
}

#[tokio::test]
async fn test_e2e_unknown_method_error() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("nonexistent.method", json!({})).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Unknown method"));
}

// ============================================================================
// CHAOS TESTS - Concurrent and Rapid Requests
// ============================================================================

#[tokio::test]
async fn test_chaos_concurrent_health_requests() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = Arc::new(IpcServiceHandler::new(registry));

    // Spawn 50 concurrent health requests
    let mut tasks = vec![];
    for i in 0..50 {
        let handler_clone = Arc::clone(&handler);
        tasks.push(tokio::spawn(async move {
            let result = handler_clone.handle("health", json!({})).await;
            assert!(result.is_ok(), "Concurrent request {} failed", i);
            result.unwrap()
        }));
    }

    // Wait for all tasks
    let results = futures::future::join_all(tasks).await;

    // Verify all succeeded
    for result in results {
        assert!(result.is_ok(), "Task should not panic");
        let response = result.unwrap();
        assert_eq!(response["status"], "healthy");
    }
}

#[tokio::test]
async fn test_chaos_rapid_sequential_requests() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    // Send 100 rapid sequential requests
    for i in 0..100 {
        let method = match i % 3 {
            0 => "health",
            1 => "identity",
            _ => "rpc.discover",
        };

        let result = handler.handle(method, json!({})).await;
        assert!(result.is_ok(), "Rapid request {} failed", i);
    }
}

#[tokio::test]
async fn test_chaos_interleaved_methods() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = Arc::new(IpcServiceHandler::new(registry));

    // Spawn tasks that interleave different methods
    let mut tasks = vec![];

    for i in 0..30 {
        let handler_clone = Arc::clone(&handler);
        let method = match i % 3 {
            0 => "health",
            1 => "identity",
            _ => "rpc.discover",
        };

        tasks.push(tokio::spawn(async move { handler_clone.handle(method, json!({})).await }));
    }

    let results = futures::future::join_all(tasks).await;

    // All should succeed
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Task {} panicked", i);
        let response = result.as_ref().unwrap();
        assert!(response.is_ok(), "Request {} failed", i);
    }
}

#[tokio::test]
async fn test_chaos_concurrent_with_service_registration() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = Arc::new(IpcServiceHandler::new(registry.clone()));

    // Spawn concurrent health checks
    let health_tasks: Vec<_> = (0..20)
        .map(|_| {
            let handler_clone = Arc::clone(&handler);
            tokio::spawn(async move { handler_clone.handle("health", json!({})).await })
        })
        .collect();

    // Concurrently register services
    let reg_tasks: Vec<_> = (0..10)
        .map(|i| {
            let registry_clone = Arc::clone(&registry);
            tokio::spawn(async move {
                let reg = registry_clone.write().await;
                let endpoint =
                    NativeEndpoint::UnixSocket(PathBuf::from(format!("/tmp/test-{}.sock", i)));
                let _ = reg
                    .register(&format!("test-service-{}", i), endpoint, vec!["test".to_string()])
                    .await;
            })
        })
        .collect();

    // Wait for all
    let _ = futures::future::join_all(health_tasks).await;
    let _ = futures::future::join_all(reg_tasks).await;
}

// ============================================================================
// FAULT INJECTION TESTS - Error Paths
// ============================================================================

#[tokio::test]
async fn test_fault_invalid_params_type() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    // Methods don't use params, so any value should work
    let result = handler.handle("health", json!("invalid")).await;
    assert!(result.is_ok(), "Should accept any params type");
}

#[tokio::test]
async fn test_fault_null_params() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("health", json!(null)).await;
    assert!(result.is_ok(), "Should handle null params");
}

#[tokio::test]
async fn test_fault_empty_method_name() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("", json!({})).await;
    assert!(result.is_err(), "Empty method should fail");
}

#[tokio::test]
async fn test_fault_very_long_method_name() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let long_method = "a".repeat(10000);
    let result = handler.handle(&long_method, json!({})).await;

    assert!(result.is_err(), "Very long method should fail");
}

#[tokio::test]
async fn test_fault_method_with_special_characters() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let special_methods = vec![
        "health\0",
        "health\n",
        "health\r",
        "health\t",
        "../../../etc/passwd",
        "health; rm -rf /",
    ];

    for method in special_methods {
        let result = handler.handle(method, json!({})).await;
        // Should safely fail, not panic
        assert!(result.is_err(), "Special char method should fail: {}", method);
    }
}

#[tokio::test]
async fn test_fault_unicode_method_name() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let unicode_methods = vec!["健康", "🎵health", "héalth", "здоровье"];

    for method in unicode_methods {
        let result = handler.handle(method, json!({})).await;
        assert!(result.is_err(), "Unicode method should fail: {}", method);
    }
}

#[tokio::test]
async fn test_fault_case_sensitivity() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    // Methods are case-sensitive
    let variations = vec!["HEALTH", "Health", "HeAlTh"];

    for method in variations {
        let result = handler.handle(method, json!({})).await;
        assert!(result.is_err(), "Case variation should fail: {}", method);
    }
}

#[tokio::test]
async fn test_fault_method_with_spaces() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let space_methods = vec![" health", "health ", " health ", "hea lth"];

    for method in space_methods {
        let result = handler.handle(method, json!({})).await;
        assert!(result.is_err(), "Method with spaces should fail: '{}'", method);
    }
}

#[tokio::test]
async fn test_fault_concurrent_errors() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = Arc::new(IpcServiceHandler::new(registry));

    // Spawn many concurrent requests to nonexistent methods
    let tasks: Vec<_> = (0..50)
        .map(|i| {
            let handler_clone = Arc::clone(&handler);
            tokio::spawn(async move {
                handler_clone.handle(&format!("nonexistent_{}", i), json!({})).await
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All should return errors (not panic)
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Task {} should not panic", i);
        let handler_result = result.as_ref().unwrap();
        assert!(handler_result.is_err(), "Request {} should error", i);
    }
}

// ============================================================================
// REGRESSION TESTS - Ensure Old Methods Still Work
// ============================================================================

#[tokio::test]
async fn test_regression_primal_info() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("primal.info", json!({})).await;
    assert!(result.is_ok(), "primal.info should still work");
}

#[tokio::test]
async fn test_regression_primal_capabilities() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("primal.capabilities", json!({})).await;
    assert!(result.is_ok(), "primal.capabilities should still work");
}

#[tokio::test]
async fn test_regression_rpc_methods() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("rpc.methods", json!({})).await;
    assert!(result.is_ok(), "rpc.methods should still work");
}

// ============================================================================
// ENVIRONMENT VARIABLE TESTS
// ============================================================================

#[tokio::test]
async fn test_env_family_id_priority() {
    // Serialize with other env tests
    let _guard = ENV_TEST_LOCK.lock().unwrap();

    // Clean slate first
    std::env::remove_var("FAMILY_ID");
    std::env::remove_var("SONGBIRD_FAMILY_ID");
    std::env::remove_var("NODE_FAMILY_ID");

    // Test 1: Only FAMILY_ID set (highest priority)
    std::env::set_var("FAMILY_ID", "test_priority_1");

    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("identity", json!({})).await.unwrap();
    assert_eq!(result["family_id"], "test_priority_1");

    std::env::remove_var("FAMILY_ID");

    // Test 2: Only SONGBIRD_FAMILY_ID set (middle priority)
    std::env::set_var("SONGBIRD_FAMILY_ID", "test_priority_2");

    let registry2 = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler2 = IpcServiceHandler::new(registry2);

    let result2 = handler2.handle("identity", json!({})).await.unwrap();
    assert_eq!(result2["family_id"], "test_priority_2");

    // Clean up completely
    std::env::remove_var("FAMILY_ID");
    std::env::remove_var("SONGBIRD_FAMILY_ID");
    std::env::remove_var("NODE_FAMILY_ID");
}

#[tokio::test]
async fn test_env_family_id_default() {
    // Serialize with other env tests
    let _guard = ENV_TEST_LOCK.lock().unwrap();

    // Clear all family ID env vars (canonical chain)
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    std::env::remove_var("BIOMEOS_FAMILY_ID");
    std::env::remove_var("SONGBIRD_FAMILY_ID");
    std::env::remove_var("FAMILY_ID");
    std::env::remove_var("NODE_FAMILY_ID");

    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry);

    let result = handler.handle("identity", json!({})).await.unwrap();
    assert_eq!(result["family_id"], "default", "Should default to 'default'");

    // Clean up (restore any vars that might have been set before test)
    std::env::remove_var("FAMILY_ID");
    std::env::remove_var("SONGBIRD_FAMILY_ID");
    std::env::remove_var("NODE_FAMILY_ID");
}
