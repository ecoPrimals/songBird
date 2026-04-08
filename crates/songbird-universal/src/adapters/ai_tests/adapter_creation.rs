// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;

#[tokio::test]
async fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = AIAdapter::new("http://ai-provider:8083".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;
    assert_eq!(adapter.endpoint(), "http://ai-provider:8083");
    Ok(())
}

#[tokio::test]
async fn test_adapter_with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = AIAdapter::new("http://ai-provider:8083".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(20));
    assert_eq!(adapter.timeout, Duration::from_secs(20));
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_with_timeout_and_endpoint_tarpc() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("tarpc://127.0.0.1:9000".to_string())
        .await?
        .with_timeout(Duration::from_millis(400));
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9000");
    assert_eq!(adapter.timeout, Duration::from_millis(400));
    Ok(())
}
