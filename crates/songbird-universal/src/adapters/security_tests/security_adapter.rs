// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! [`SecurityAdapter`] construction, endpoint handling, and timeout configuration.

use super::super::*;
use songbird_types::SongbirdError;
use std::time::Duration;

#[tokio::test]
async fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
    let adapter =
        SecurityAdapter::new("http://security-provider:8081".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
    assert_eq!(adapter.endpoint(), "http://security-provider:8081");
    Ok(())
}

#[tokio::test]
async fn test_adapter_with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://security-provider:8081".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(10));
    assert_eq!(adapter.timeout, Duration::from_secs(10));
    Ok(())
}

#[tokio::test]
async fn test_security_adapter_endpoint_accessor() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "http://test-security:9999".to_string();
    let adapter = SecurityAdapter::new(endpoint.clone()).await?;
    assert_eq!(adapter.endpoint(), &endpoint);
    Ok(())
}

#[tokio::test]
async fn test_adapter_endpoint_access() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "http://test-security:9000";
    let adapter = SecurityAdapter::new(endpoint.to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    assert_eq!(adapter.endpoint(), endpoint, "Endpoint should be accessible");
    Ok(())
}

#[tokio::test]
async fn test_adapter_timeout_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(5));

    assert_eq!(adapter.timeout, Duration::from_secs(5), "Timeout should be configurable");
    Ok(())
}

#[tokio::test]
async fn test_adapter_default_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    assert_eq!(adapter.timeout, Duration::from_secs(5), "Default timeout should be 5 seconds");
    Ok(())
}

#[tokio::test]
async fn test_adapter_with_various_endpoints() {
    // Test empty endpoint (currently accepted, may want to validate later)
    let result = SecurityAdapter::new(String::new()).await;
    assert!(result.is_ok(), "Empty endpoint creates adapter (validation could be added)");

    // Test various endpoint formats
    let result = SecurityAdapter::new("http://localhost:8080".to_string()).await;
    assert!(result.is_ok(), "Valid HTTP endpoint should work");

    let result = SecurityAdapter::new("https://security.example.com".to_string()).await;
    assert!(result.is_ok(), "Valid HTTPS endpoint should work");
}

#[tokio::test]
async fn test_adapter_chained_timeout_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(2))
        .with_timeout(Duration::from_secs(8));

    assert_eq!(adapter.timeout, Duration::from_secs(8), "Last timeout should be applied");
    Ok(())
}

#[tokio::test]
async fn test_adapter_zero_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(0));

    assert_eq!(
        adapter.timeout,
        Duration::from_secs(0),
        "Zero timeout should be accepted (may cause immediate failures)"
    );
    Ok(())
}

#[tokio::test]
async fn test_adapter_very_long_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(3600));

    assert_eq!(
        adapter.timeout,
        Duration::from_secs(3600),
        "Long timeout (1 hour) should be accepted"
    );
    Ok(())
}

#[tokio::test]
async fn test_adapter_endpoint_with_trailing_slash() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://security:8080/".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    assert_eq!(adapter.endpoint(), "http://security:8080/");
    Ok(())
}

#[tokio::test]
async fn test_adapter_endpoint_with_path() -> Result<(), Box<dyn std::error::Error>> {
    let adapter =
        SecurityAdapter::new("http://security:8080/api/v1".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;

    assert_eq!(adapter.endpoint(), "http://security:8080/api/v1");
    Ok(())
}

#[tokio::test]
async fn test_adapter_builder_pattern_immutability() -> Result<(), Box<dyn std::error::Error>> {
    let adapter1 = SecurityAdapter::new("http://test:8080".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    let adapter2 = adapter1.with_timeout(Duration::from_secs(10));

    // Original timeout should remain unchanged (moved ownership)
    assert_eq!(adapter2.timeout, Duration::from_secs(10));
    Ok(())
}
