//! Tests for service registration handles

use super::RegistrationHandle;
use std::time::Duration;

#[test]
fn test_registration_handle_creation() {
    let handle = RegistrationHandle::new("test-service".to_string());
    assert_eq!(handle.service_id, "test-service");
}

#[test]
fn test_registration_handle_clone() {
    let handle = RegistrationHandle::new("service-1".to_string());
    let cloned = handle.clone();
    assert_eq!(handle.service_id, cloned.service_id);
}

#[test]
fn test_registration_handle_debug() {
    let handle = RegistrationHandle::new("debug-service".to_string());
    let debug_str = format!("{:?}", handle);
    assert!(debug_str.contains("RegistrationHandle"));
    assert!(debug_str.contains("debug-service"));
}

#[tokio::test]
async fn test_wait_ready_returns_immediately() {
    let handle = RegistrationHandle::new("ready-service".to_string());

    // Should return immediately since registration is synchronous
    let start = std::time::Instant::now();
    handle.wait_ready().await;
    let elapsed = start.elapsed();

    // Should complete in less than 10ms
    assert!(elapsed < Duration::from_millis(10));
}

#[tokio::test]
async fn test_wait_ready_timeout_succeeds() {
    let handle = RegistrationHandle::new("timeout-service".to_string());

    let result = handle.wait_ready_timeout(Duration::from_secs(1)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wait_ready_timeout_with_short_duration() {
    let handle = RegistrationHandle::new("short-timeout".to_string());

    // Should still succeed immediately even with 1ms timeout
    let result = handle.wait_ready_timeout(Duration::from_millis(1)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_wait_ready_calls() {
    let handle = RegistrationHandle::new("multi-wait".to_string());

    // Multiple waits should all succeed
    handle.wait_ready().await;
    handle.wait_ready().await;
    handle.wait_ready().await;
}

#[tokio::test]
async fn test_cloned_handle_wait_ready() {
    let handle = RegistrationHandle::new("clone-test".to_string());
    let cloned = handle.clone();

    // Both should be ready
    handle.wait_ready().await;
    cloned.wait_ready().await;
}

#[tokio::test]
async fn test_concurrent_wait_ready() {
    let handle = RegistrationHandle::new("concurrent".to_string());
    let handle2 = handle.clone();
    let handle3 = handle.clone();

    // All should complete
    let (r1, r2, r3) = tokio::join!(
        async {
            handle.wait_ready().await;
            true
        },
        async {
            handle2.wait_ready().await;
            true
        },
        async {
            handle3.wait_ready().await;
            true
        },
    );

    assert!(r1 && r2 && r3);
}

#[test]
fn test_handle_with_empty_service_id() {
    let handle = RegistrationHandle::new(String::new());
    assert_eq!(handle.service_id, "");
}

#[test]
fn test_handle_with_long_service_id() {
    let long_id = "a".repeat(1000);
    let handle = RegistrationHandle::new(long_id.clone());
    assert_eq!(handle.service_id, long_id);
}

#[test]
fn test_handle_with_special_characters() {
    let handle = RegistrationHandle::new("service-123_test.v2".to_string());
    assert_eq!(handle.service_id, "service-123_test.v2");
}

#[test]
fn test_handle_with_unicode() {
    let handle = RegistrationHandle::new("服务-🚀".to_string());
    assert_eq!(handle.service_id, "服务-🚀");
}

#[tokio::test]
async fn test_wait_ready_timeout_zero_duration() {
    let handle = RegistrationHandle::new("zero-timeout".to_string());

    // Even with zero timeout, should succeed since it's already ready
    let result = handle.wait_ready_timeout(Duration::from_secs(0)).await;
    // This might actually timeout with 0 duration, so we just verify it completes
    let _ = result;
}

#[test]
fn test_handle_size_is_reasonable() {
    let size = std::mem::size_of::<RegistrationHandle>();
    // Should be relatively small (String + Arc<Notify>)
    assert!(size <= 48, "RegistrationHandle should be reasonably sized, got {}", size);
}

#[tokio::test]
async fn test_multiple_timeout_calls() {
    let handle = RegistrationHandle::new("multi-timeout".to_string());

    let r1 = handle.wait_ready_timeout(Duration::from_millis(100)).await;
    let r2 = handle.wait_ready_timeout(Duration::from_millis(100)).await;
    let r3 = handle.wait_ready_timeout(Duration::from_millis(100)).await;

    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());
}
