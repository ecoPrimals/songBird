//! # Storage Adapter Implementation
//!
//! Main storage adapter that delegates to capability providers.

use super::stats::StorageStats;
use super::types::*;
use crate::adapters::{context::AdapterContext, routing};
use serde_json::json;
use songbird_errors::{SongbirdError, SongbirdResult};
use std::sync::Arc;
// Tracing imports removed - not used in this focused module

/// Universal Storage Adapter
///
/// Routes all storage operations to capability providers via the universal adapter.
/// No direct filesystem operations or fallbacks - clean separation of concerns.
#[derive(Debug)]
pub struct StorageAdapter {
    stats: Arc<StorageStats>,
    _config: StorageConfig,
}

impl Default for StorageAdapter {
    fn default() -> Self {
        Self::new(StorageConfig::default())
    }
}

impl StorageAdapter {
    /// Create new storage adapter with configuration
    pub fn new(_config: StorageConfig) -> Self {
        Self {
            stats: Arc::new(StorageStats::new()),
            _config,
        }
    }

    /// Store data with the given key
    pub async fn store(&self) -> SongbirdResult<StorageResult> {
        let ctx = AdapterContext::new("storage_adapter");
        let start_time = std::time::Instant::now();

        let payload = json!({
            "operation": "store",
            "key": key,
            "data": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data),
            "size": data.len()
        });

        match routing::storage_request(ctx, "store".to_string(), payload).await {
            Ok(songbird_errors::evolved_success(_response)) => {
                let latency = start_time.elapsed();
                self.stats.record_success(latency, data.len() as u64);

                Ok(success(StorageResult::Stored {
                    key,
                    size: data.len() as u64,
                }))
            }
            Err(e) => {
                let latency = start_time.elapsed();
                self.stats.record_failure(latency);
                Err(e)
            }
        }
    }

    /// Retrieve data by key
    pub async fn retrieve(&self) -> SongbirdResult<StorageResult> {
        let ctx = AdapterContext::new("storage_adapter");
        let start_time = std::time::Instant::now();

        let payload = json!({
            "operation": "retrieve",
            "key": key
        });

        match routing::storage_request(ctx, "retrieve".to_string(), payload).await {
            Ok(songbird_errors::evolved_success(response)) => {
                let latency = start_time.elapsed();

                // Parse response to extract data
                let data =
                    if let Some(data_str) = response.data.get("data").and_then(|v| v.as_str()) {
                        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data_str)
                            .map_err(|e| {
                                SongbirdError::internal_error(format!("Failed to decode data: {e}"))
                            })?
                    } else {
                        Vec::new()
                    };

                self.stats.record_success(latency, data.len() as u64);

                Ok(success(StorageResult::Retrieved { key, data }))
            }
            Err(e) => {
                let latency = start_time.elapsed();
                self.stats.record_failure(latency);
                Err(e)
            }
        }
    }

    /// Delete data by key
    pub async fn delete(&self) -> SongbirdResult<StorageResult> {
        let ctx = AdapterContext::new("storage_adapter");
        let start_time = std::time::Instant::now();

        let payload = json!({
            "operation": "delete",
            "key": key
        });

        match routing::storage_request(ctx, "delete".to_string(), payload).await {
            Ok(songbird_errors::evolved_success(_response)) => {
                let latency = start_time.elapsed();
                self.stats.record_success(latency, 0);

                Ok(success(StorageResult::Deleted { key }))
            }
            Err(e) => {
                let latency = start_time.elapsed();
                self.stats.record_failure(latency);
                Err(e)
            }
        }
    }

    /// List keys with optional prefix
    pub async fn list(&self) -> SongbirdResult<StorageResult> {
        let ctx = AdapterContext::new("storage_adapter");
        let start_time = std::time::Instant::now();

        let payload = json!({
            "operation": "list",
            "prefix": prefix
        });

        match routing::storage_request(ctx, "list".to_string(), payload).await {
            Ok(songbird_errors::evolved_success(response)) => {
                let latency = start_time.elapsed();

                // Parse response to extract keys
                let keys = response
                    .data
                    .get("keys")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                self.stats.record_success(latency, 0);

                Ok(success(StorageResult::Listed { keys }))
            }
            Err(e) => {
                let latency = start_time.elapsed();
                self.stats.record_failure(latency);
                Err(e)
            }
        }
    }

    /// Check if key exists
    pub async fn exists(&self) -> SongbirdResult<StorageResult> {
        let ctx = AdapterContext::new("storage_adapter");
        let start_time = std::time::Instant::now();

        let payload = json!({
            "operation": "exists",
            "key": key
        });

        match routing::storage_request(ctx, "exists".to_string(), payload).await {
            Ok(songbird_errors::evolved_success(response)) => {
                let latency = start_time.elapsed();

                let exists = response
                    .data
                    .get("exists")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                self.stats.record_success(latency, 0);

                Ok(success(StorageResult::Exists { key, exists }))
            }
            Err(e) => {
                let latency = start_time.elapsed();
                self.stats.record_failure(latency);
                Err(e)
            }
        }
    }

    /// Get performance statistics
    pub fn stats(&self) -> &StorageStats {
        &self.stats
    }

    /// Perform health check
    pub async fn health_check(&self) -> SongbirdResult<serde_json::Value> {
        let ctx = AdapterContext::new("storage_adapter");
        routing::health_check(ctx, "storage").await
    }
}
