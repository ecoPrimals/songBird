// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::types::Task;
use super::CapabilityRouter;
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::info;

impl CapabilityRouter {
    /// Execute a task on an external provider
    ///
    /// Sends the task to the provider's execution endpoint and waits for results
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn execute_on_external_provider(
        &self,
        endpoint: &str,
        task: &Task,
    ) -> SongbirdResult<serde_json::Value> {
        info!("Executing task on external provider: {}", endpoint);

        // ✅ EVOLVED (Jan 21, 2026): 100% Pure Rust HTTP via SongbirdHttpClient
        let crypto_socket =
            crate::primal_discovery::discover_crypto_provider().await.map_err(|e| {
                SongbirdError::Network {
                    message: format!("Failed to discover crypto provider: {e}"),
                    interface: None,
                    suggestion: Some("Check security provider availability".to_string()),
                }
            })?;

        let client = songbird_http_client::SongbirdHttpClient::new(crypto_socket);
        let task_json = serde_json::to_value(task).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: e.to_string(),
            debug_info: None,
        })?;

        let response =
            tokio::time::timeout(songbird_types::defaults::timeouts::DEFAULT_COMPUTE_TIMEOUT, client.post(endpoint, task_json))
                .await
                .map_err(|_| SongbirdError::Network {
                    message: "Request timeout (5 minutes)".to_string(),
                    interface: Some(endpoint.to_string()),
                    suggestion: Some(
                        "Check provider endpoint and network connectivity".to_string(),
                    ),
                })?
                .map_err(|e| SongbirdError::Network {
                    message: format!("Failed to send task to external provider: {e}"),
                    interface: Some(endpoint.to_string()),
                    suggestion: Some(
                        "Check provider endpoint and network connectivity".to_string(),
                    ),
                })?;

        if response.status < 200 || response.status >= 300 {
            return Err(SongbirdError::Service {
                service: "external_provider".to_string(),
                message: format!("Provider returned error status: {}", response.status),
                suggested_alternatives: vec![],
                recovery_actions: vec!["retry".to_string(), "route_to_fallback".to_string()],
            });
        }

        let result =
            serde_json::from_value(response.body).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Failed to parse provider response: {e}"),
                debug_info: None,
            })?;

        info!("Task execution completed successfully on external provider");
        Ok(result)
    }
}
