// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP health probing for registered service endpoints.

use std::time::{Duration, SystemTime};

use tracing::{debug, warn};

use crate::discovery::core::ServiceInstance;

use super::types::ServiceHealthStatus;

/// Health check result (internal)
#[derive(Debug)]
pub(super) struct HealthCheckResult {
    pub status: ServiceHealthStatus,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}

/// Perform a GET on `{endpoint}/health` with the given timeout.
pub(super) async fn perform_health_check(
    service: &ServiceInstance,
    timeout: Duration,
) -> HealthCheckResult {
    let start_time = SystemTime::now();

    let health_url = if service.endpoint.ends_with('/') {
        format!("{}health", service.endpoint)
    } else {
        format!("{}/health", service.endpoint)
    };

    debug!("Performing health check for service: {} at {}", service.id, health_url);

    let client =
        match songbird_http_client::IpcHttpClient::builder().with_timeout(timeout).build().await {
            Ok(client) => client,
            Err(e) => {
                return HealthCheckResult {
                    status: ServiceHealthStatus::Unhealthy,
                    response_time_ms: 0,
                    error_message: Some(format!("Failed to create HTTP client: {e}")),
                };
            }
        };

    match client.get(&health_url).await {
        Ok(response) => {
            let response_time =
                u64::try_from(start_time.elapsed().unwrap_or(Duration::ZERO).as_millis())
                    .unwrap_or(u64::MAX);

            if (200..300).contains(&response.status()) {
                HealthCheckResult {
                    status: ServiceHealthStatus::Healthy,
                    response_time_ms: response_time,
                    error_message: None,
                }
            } else {
                HealthCheckResult {
                    status: ServiceHealthStatus::Degraded,
                    response_time_ms: response_time,
                    error_message: Some(format!("HTTP {}", response.status())),
                }
            }
        }
        Err(e) => {
            let response_time =
                u64::try_from(start_time.elapsed().unwrap_or(Duration::ZERO).as_millis())
                    .unwrap_or(u64::MAX);
            warn!("Health check failed for service {}: {}", service.id, e);

            HealthCheckResult {
                status: ServiceHealthStatus::Unhealthy,
                response_time_ms: response_time,
                error_message: Some(e.to_string()),
            }
        }
    }
}
