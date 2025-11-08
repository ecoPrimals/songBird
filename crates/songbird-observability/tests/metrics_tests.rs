//! Tests for observability metrics functionality

use songbird_observability::*;
use songbird_types::SongbirdResult;

#[tokio::test]
async fn test_observability_manager_creation() -> SongbirdResult<()> {
    let manager = ObservabilityManager::new();

    // Should create without panic
    assert!(format!("{manager:?}").contains("ObservabilityManager"));
    Ok(())
}

#[tokio::test]
async fn test_observability_manager_start() {
    let manager = ObservabilityManager::new();

    let result = manager.start().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_observability_manager_stop() -> SongbirdResult<()> {
    let manager = ObservabilityManager::new();

    manager.start().await.unwrap();

    let result = manager.stop().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_observability_manager_lifecycle() -> SongbirdResult<()> {
    let manager = ObservabilityManager::new();

    // Full lifecycle
    assert!(manager.start().await.is_ok());
    assert!(manager.stop().await.is_ok());
    Ok(())
}

#[test]
fn test_observability_manager_debug() {
    let manager = ObservabilityManager::new();

    let debug_str = format!("{manager:?}");
    assert!(debug_str.contains("ObservabilityManager"));
}

#[tokio::test]
async fn test_multiple_managers() -> SongbirdResult<()> {
    let manager1 = ObservabilityManager::new();
    let manager2 = ObservabilityManager::new();

    // Should be able to create multiple managers
    assert!(format!("{manager1:?}").contains("ObservabilityManager"));
    assert!(format!("{manager2:?}").contains("ObservabilityManager"));
    Ok(())
}

#[tokio::test]
async fn test_manager_start_idempotent() {
    let manager = ObservabilityManager::new();

    // Starting multiple times should be safe
    assert!(manager.start().await.is_ok());
    assert!(manager.start().await.is_ok());
}

#[tokio::test]
async fn test_manager_stop_before_start() {
    let manager = ObservabilityManager::new();

    // Stopping before starting should be safe
    let result = manager.stop().await;
    assert!(result.is_ok());
}

#[test]
fn test_manager_creation_is_fast() {
    use std::time::Instant;

    let start = Instant::now();
    let _manager = ObservabilityManager::new();
    let elapsed = start.elapsed();

    // Manager creation should be nearly instantaneous
    assert!(elapsed.as_millis() < 100);
}

#[tokio::test]
async fn test_manager_lifecycle_timing() {
    use std::time::Instant;

    let manager = ObservabilityManager::new();

    let start = Instant::now();
    manager.start().await.unwrap();
    manager.stop().await.unwrap();
    let elapsed = start.elapsed();

    // Lifecycle operations should be fast
    assert!(elapsed.as_millis() < 1000);
}
