// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Storage Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY storage capability provider.
//! It does NOT know about specific primals (`storage provider` is just one example).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.
//!
//! Songbird has local storage for sovereign standalone operation, but can utilize
//! any discovered storage provider for network effects.

use crate::adapters::transport::{
    AdapterTransportKind, CapabilityTransport, build_default_transport, transport_kind_for_endpoint,
};
use serde::{Deserialize, Serialize};
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// Storage metrics from any storage capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    /// Total storage capacity in bytes
    pub total_capacity_bytes: u64,
    /// Used storage in bytes
    pub used_bytes: u64,
    /// Available storage in bytes
    pub available_bytes: u64,
    /// Number of stored objects
    pub object_count: u64,
    /// Average read latency in milliseconds
    pub avg_read_latency_ms: f64,
    /// Average write latency in milliseconds
    pub avg_write_latency_ms: f64,
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl StorageMetrics {
    /// Calculate storage usage percentage
    #[must_use]
    pub fn usage_percent(&self) -> f64 {
        if self.total_capacity_bytes == 0 {
            return 0.0;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "intentional pattern; clippy false positive for this API"
        )]
        {
            (self.used_bytes as f64 / self.total_capacity_bytes as f64) * 100.0
        }
    }

    /// Check if storage is nearly full
    #[must_use]
    pub fn is_nearly_full(&self) -> bool {
        self.usage_percent() > 90.0
    }

    /// Check if latency is high
    #[must_use]
    pub fn is_high_latency(&self) -> bool {
        self.avg_read_latency_ms > 100.0 || self.avg_write_latency_ms > 200.0
    }

    /// Get storage health status
    #[must_use]
    pub fn health_status(&self) -> StorageHealth {
        let usage = self.usage_percent();

        if usage > 95.0 || self.avg_write_latency_ms > 500.0 {
            StorageHealth::Critical
        } else if usage > 85.0 || self.is_high_latency() {
            StorageHealth::Warning
        } else {
            StorageHealth::Healthy
        }
    }
}

/// Storage health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageHealth {
    /// Storage is healthy
    Healthy,
    /// Storage warnings present
    Warning,
    /// Storage is critical
    Critical,
}

/// **CAPABILITY-BASED STORAGE ADAPTER**
///
/// Works with ANY storage provider discovered through:
/// - Environment variable: `SONGBIRD_STORAGE_ENDPOINT`
/// - Capability discovery: `capability:storage`
/// - Zero-knowledge bootstrap
/// - Local filesystem fallback for sovereign operation
///
/// **v3.11.0**: Protocol-agnostic - supports Unix sockets (PRIMARY) or HTTP (FALLBACK)
pub struct StorageAdapter {
    /// Endpoint URL for the storage capability provider
    endpoint: String,
    transport: Arc<CapabilityTransport>,
    transport_kind: AdapterTransportKind,
    /// Request timeout
    timeout: Duration,
}

impl fmt::Debug for StorageAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol = match self.transport_kind {
            AdapterTransportKind::Tarpc => "Protocol::Tarpc",
            AdapterTransportKind::JsonRpc => "Protocol::JsonRpc",
            AdapterTransportKind::Http => "Protocol::Http",
        };
        f.debug_struct("StorageAdapter")
            .field("endpoint", &self.endpoint)
            .field("protocol", &protocol)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl StorageAdapter {
    /// Create adapter from discovered storage capability
    ///
    /// Uses capability-based discovery:
    /// 1. Check `SONGBIRD_STORAGE_ENDPOINT` environment variable
    /// 2. Fall back to capability discovery
    /// 3. No hardcoded primal names anywhere
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use songbird_universal::adapters::StorageAdapter;
    ///
    /// // Discovers any storage provider (could be storage provider, or local filesystem, or anyone)
    /// let adapter = StorageAdapter::from_discovery().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if no storage capability can be discovered.
    pub async fn from_discovery() -> SongbirdResult<Self> {
        Self::from_discovery_with_resolver(
            songbird_config::capability_endpoints::CapabilityEndpointResolver::new(),
        )
        .await
    }

    /// Like [`Self::from_discovery`], but uses an explicit [`songbird_config::capability_endpoints::CapabilityEndpointResolver`].
    ///
    /// # Errors
    ///
    /// Returns an error if capability endpoint resolution or adapter construction fails.
    pub async fn from_discovery_with_resolver(
        resolver: songbird_config::capability_endpoints::CapabilityEndpointResolver,
    ) -> SongbirdResult<Self> {
        use songbird_config::capability_endpoints::CapabilityType;

        match resolver.get_endpoint(CapabilityType::Storage).await {
            Ok(endpoint) => {
                debug!("✅ Storage capability discovered via resolver: {}", endpoint);
                Self::new(endpoint).await
            }
            Err(discovery_err) => {
                debug!("🔍 Primary discovery failed, trying legacy fallbacks: {}", discovery_err);

                // Fallback 1: Capability-based env vars first, then legacy fallbacks
                if let Ok(endpoint) = SafeEnv::get_required("STORAGE_ENDPOINT") {
                    debug!("✅ Using capability-based STORAGE_ENDPOINT for storage endpoint");
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("STORAGE_PROVIDER_ENDPOINT") {
                    debug!(
                        "✅ Using capability-based STORAGE_PROVIDER_ENDPOINT for storage endpoint"
                    );
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("SONGBIRD_STORAGE_ENDPOINT") {
                    debug!(
                        "⚠️ Using legacy SONGBIRD_STORAGE_ENDPOINT for storage endpoint (prefer STORAGE_ENDPOINT)"
                    );
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("NESTGATE_ENDPOINT") {
                    warn!(
                        "NESTGATE_ENDPOINT is deprecated for storage configuration; use STORAGE_ENDPOINT or STORAGE_PROVIDER_ENDPOINT instead (capability-first per PRIMAL_SELF_KNOWLEDGE_STANDARD)"
                    );
                    debug!("⚠️ Using deprecated legacy NESTGATE_ENDPOINT for storage endpoint");
                    return Self::new(endpoint).await;
                }

                // Fallback 2: Construct from host + port
                let fallback_host = format!("http://{}", songbird_types::constants::LOCALHOST);
                let endpoint = SafeEnv::get_or_default("SONGBIRD_HOST", &fallback_host);
                let port = songbird_config::defaults::ports::storage_provider_port().to_string();
                let discovered_endpoint = format!("{endpoint}:{port}");

                debug!("🔄 Using fallback storage endpoint: {}", discovered_endpoint);
                Self::new(discovered_endpoint).await
            }
        }
    }

    /// Create adapter with explicit endpoint (for testing or explicit configuration)
    ///
    /// **v3.12.0**: Protocol-agnostic - automatically detects tarpc, Unix sockets, or HTTP
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of any storage capability provider
    ///   - `tarpc://host:port` → tarpc binary RPC (PRIMARY - 10-100x faster)
    ///   - `unix:///path/to/socket.sock` → JSON-RPC over Unix socket (SECONDARY - port-free)
    ///   - `http://...` or `https://...` → HTTP (FALLBACK - network only)
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol client cannot be created.
    #[expect(clippy::unused_async, reason = "unused bindings/imports in this compilation unit")] // async retained for API stability
    pub async fn new(endpoint: String) -> SongbirdResult<Self> {
        let transport_kind = transport_kind_for_endpoint(&endpoint);
        if endpoint.starts_with("tarpc://") {
            debug!("🚀 Detected tarpc endpoint for storage (PRIMARY): {}", endpoint);
        } else if endpoint.starts_with("unix://") {
            debug!("🔌 Detected Unix socket endpoint for storage (SECONDARY): {}", endpoint);
        } else {
            debug!("🌐 Detected HTTP endpoint for storage (FALLBACK): {}", endpoint);
        }

        let transport = build_default_transport(&endpoint)?;

        Ok(Self {
            endpoint,
            transport,
            transport_kind,
            timeout: Duration::from_secs(5),
        })
    }

    /// Construct with an explicit transport (crate tests and advanced injection).
    #[cfg(test)]
    pub(crate) fn with_transport(
        endpoint: String,
        transport: Arc<CapabilityTransport>,
        transport_kind: AdapterTransportKind,
        timeout: Duration,
    ) -> Self {
        Self {
            endpoint,
            transport,
            transport_kind,
            timeout,
        }
    }

    /// Set custom request timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Collect storage metrics from the capability provider
    ///
    /// **v3.11.0**: Protocol-agnostic - works with Unix sockets or HTTP
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network/IPC request fails
    /// - Service returns non-success status (HTTP) or error (JSON-RPC)
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<StorageMetrics> {
        debug!("Collecting storage metrics from: {}", self.endpoint);

        let result = tokio::time::timeout(self.timeout, self.transport.get("metrics/storage"))
            .await
            .map_err(|_| {
                SongbirdError::network(format!(
                    "Timeout after {:?} reaching storage provider",
                    self.timeout
                ))
            })?
            .map_err(|e| {
                warn!("Failed to collect storage metrics: {e}");
                match self.transport_kind {
                    AdapterTransportKind::Http => e,
                    _ => SongbirdError::network(format!("Failed to reach storage provider: {e}")),
                }
            })?;

        let mut metrics: StorageMetrics = serde_json::from_value(result).map_err(|e| {
            warn!("Failed to parse storage metrics: {e}");
            match self.transport_kind {
                AdapterTransportKind::Http => SongbirdError::service(
                    "storage",
                    format!("Failed to parse storage metrics: {e}"),
                ),
                _ => SongbirdError::serialization(format!("Failed to parse storage metrics: {e}")),
            }
        })?;

        // Set timestamp if not provided
        if metrics.timestamp.timestamp() == 0 {
            metrics.timestamp = chrono::Utc::now();
        }

        debug!(
            "Collected storage metrics: Usage={}%, Objects={}, ReadLatency={}ms, WriteLatency={}ms",
            metrics.usage_percent(),
            metrics.object_count,
            metrics.avg_read_latency_ms,
            metrics.avg_write_latency_ms
        );

        Ok(metrics)
    }

    /// Check storage health
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    pub async fn check_health(&self) -> SongbirdResult<StorageHealth> {
        let metrics = self.collect_metrics().await?;
        Ok(metrics.health_status())
    }

    /// Get the endpoint URL
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Trait for storage capability providers
#[expect(async_fn_in_trait, reason = "native async trait for StorageAdapter impl")]
pub trait StorageProvider: Send + Sync {
    /// Collect current storage metrics
    async fn collect_storage_metrics(&self) -> SongbirdResult<StorageMetrics>;

    /// Check storage health
    async fn check_storage_health(&self) -> SongbirdResult<StorageHealth> {
        let metrics = self.collect_storage_metrics().await?;
        Ok(metrics.health_status())
    }
}

impl StorageProvider for StorageAdapter {
    async fn collect_storage_metrics(&self) -> SongbirdResult<StorageMetrics> {
        self.collect_metrics().await
    }
}

#[cfg(test)]
#[path = "storage_tests/mod.rs"]
mod tests;
