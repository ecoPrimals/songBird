// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Protocol selection, endpoint discovery, and RPC/HTTP dispatch for compute.

use super::metrics::{ComputeMetrics, HealthStatus};
use crate::adapters::transport::{
    AdapterTransportKind, CapabilityTransport, build_default_transport, transport_kind_for_endpoint,
};
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// Generic adapter for compute capability providers
///
/// **SOVEREIGNTY**: This adapter discovers compute providers by capability,
/// not by hardcoded primal names. It works with ANY service that implements
/// the compute capability interface.
///
/// **v3.11.0**: Protocol-agnostic - supports Unix sockets (PRIMARY) or HTTP (FALLBACK)
pub struct ComputeAdapter {
    /// Endpoint URL for the compute service (discovered dynamically)
    endpoint: String,
    transport: Arc<dyn CapabilityTransport>,
    transport_kind: AdapterTransportKind,
    /// Request timeout (`pub(super)` so `compute` integration tests can assert defaults.)
    pub(super) timeout: Duration,
}

impl fmt::Debug for ComputeAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol = match self.transport_kind {
            AdapterTransportKind::Tarpc => "Protocol::Tarpc",
            AdapterTransportKind::JsonRpc => "Protocol::JsonRpc",
            AdapterTransportKind::Http => "Protocol::Http",
        };
        f.debug_struct("ComputeAdapter")
            .field("endpoint", &self.endpoint)
            .field("protocol", &protocol)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl ComputeAdapter {
    /// Create adapter by discovering compute capability provider
    ///
    /// **SOVEREIGNTY**: Discovers whoever provides "compute" capability,
    /// doesn't assume any specific primal exists.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No compute capability provider found
    /// - HTTP client creation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # tokio_test::block_on(async {
    /// use songbird_universal::adapters::ComputeAdapter;
    ///
    /// // Discovers compute provider dynamically
    /// let adapter = ComputeAdapter::new_from_discovery().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn new_from_discovery() -> SongbirdResult<Self> {
        Self::new_from_discovery_with_resolver(
            songbird_config::capability_endpoints::CapabilityEndpointResolver::new(),
        )
        .await
    }

    /// Like [`Self::new_from_discovery`], but uses an explicit resolver (see [`songbird_config::capability_endpoints::CapabilityEndpointResolver::with_endpoint_overrides`]).
    ///
    /// # Errors
    ///
    /// Returns an error if capability endpoint resolution or adapter construction fails.
    pub async fn new_from_discovery_with_resolver(
        resolver: songbird_config::capability_endpoints::CapabilityEndpointResolver,
    ) -> SongbirdResult<Self> {
        use songbird_config::capability_endpoints::CapabilityType;

        match resolver.get_endpoint(CapabilityType::Compute).await {
            Ok(endpoint) => {
                debug!("✅ Compute capability discovered via resolver: {}", endpoint);
                Self::new(endpoint).await
            }
            Err(discovery_err) => {
                debug!("🔍 Primary discovery failed, trying legacy fallbacks: {}", discovery_err);

                // Fallback 1: Environment variables (capability-first, then deprecated primal)
                if let Ok(endpoint) = SafeEnv::get_required("SONGBIRD_COMPUTE_ENDPOINT") {
                    debug!("Using SONGBIRD_COMPUTE_ENDPOINT after resolver miss");
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("COMPUTE_CAPABILITY_ENDPOINT") {
                    debug!("Using COMPUTE_CAPABILITY_ENDPOINT after resolver miss");
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("COMPUTE_ENDPOINT") {
                    debug!("Using COMPUTE_ENDPOINT after resolver miss");
                    return Self::new(endpoint).await;
                }
                if let Ok(endpoint) = SafeEnv::get_required("TOADSTOOL_ENDPOINT") {
                    warn!(
                        "TOADSTOOL_ENDPOINT is deprecated — migrate to COMPUTE_ENDPOINT or COMPUTE_PROVIDER_ENDPOINT"
                    );
                    return Self::new(endpoint).await;
                }

                // Fallback 2: Construct from host + port
                let host = SafeEnv::get_or_default("SONGBIRD_HOST", "http://localhost");
                let port = SafeEnv::get_port(
                    "SONGBIRD_COMPUTE_PORT",
                    songbird_config::defaults::ports::service_port("COMPUTE", 8080),
                )
                .to_string();
                let endpoint = format!("{host}:{port}");

                debug!("🔄 Using fallback compute endpoint: {}", endpoint);
                Self::new(endpoint).await
            }
        }
    }

    /// Create adapter with explicit endpoint
    ///
    /// **v3.11.0**: Protocol-agnostic - automatically detects Unix sockets or HTTP
    ///
    /// Use this when you already know the compute provider's endpoint
    /// (e.g., from service discovery).
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of any compute capability provider
    ///   - `tarpc://host:port` → tarpc binary RPC (PRIMARY - 10-100x faster)
    ///   - `unix:///path/to/socket.sock` → JSON-RPC over Unix socket (SECONDARY - port-free)
    ///   - `http://...` or `https://...` → HTTP (FALLBACK - network only)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use songbird_universal::adapters::ComputeAdapter;
    ///
    /// // Works with ANY compute provider - protocol auto-detected
    /// let adapter = ComputeAdapter::new("tarpc://localhost:9001".to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol client cannot be created.
    #[expect(clippy::unused_async, reason = "unused bindings/imports in this compilation unit")] // async retained for API stability
    pub async fn new(endpoint: String) -> SongbirdResult<Self> {
        let transport_kind = transport_kind_for_endpoint(&endpoint);
        if endpoint.starts_with("tarpc://") {
            debug!("🚀 Detected tarpc endpoint for compute (PRIMARY): {}", endpoint);
        } else if endpoint.starts_with("unix://") {
            debug!("🔌 Detected Unix socket endpoint for compute (SECONDARY): {}", endpoint);
        } else {
            debug!("🌐 Detected HTTP endpoint for compute (FALLBACK): {}", endpoint);
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
        transport: Arc<dyn CapabilityTransport>,
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

    /// Collect compute metrics from the service
    ///
    /// **v3.11.0**: Protocol-agnostic - works with Unix sockets or HTTP
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network/IPC request fails
    /// - Service returns non-success status (HTTP) or error (JSON-RPC)
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<ComputeMetrics> {
        debug!("Collecting compute metrics from: {}", self.endpoint);

        let result = tokio::time::timeout(self.timeout, self.transport.get("metrics/compute"))
            .await
            .map_err(|_| {
                SongbirdError::network(format!(
                    "Timeout after {:?} reaching compute service",
                    self.timeout
                ))
            })?
            .map_err(|e| {
                warn!("Failed to collect compute metrics: {e}");
                match self.transport_kind {
                    AdapterTransportKind::Http => e,
                    _ => SongbirdError::network(format!("Failed to reach compute service: {e}")),
                }
            })?;

        let mut metrics: ComputeMetrics = serde_json::from_value(result).map_err(|e| {
            warn!("Failed to parse compute metrics: {e}");
            match self.transport_kind {
                AdapterTransportKind::Http => SongbirdError::service(
                    "compute",
                    format!("Failed to parse compute metrics: {e}"),
                ),
                _ => SongbirdError::serialization(format!("Failed to parse compute metrics: {e}")),
            }
        })?;

        // Set timestamp if not provided
        if metrics.timestamp.timestamp() == 0 {
            metrics.timestamp = chrono::Utc::now();
        }

        debug!(
            "Collected compute metrics: CPU={}%, Memory={}%, Active={}, Queued={}",
            metrics.cpu_usage_percent,
            metrics.memory_usage_percent(),
            metrics.active_containers,
            metrics.queued_jobs
        );

        Ok(metrics)
    }

    /// Check health of the compute service
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    pub async fn check_health(&self) -> SongbirdResult<HealthStatus> {
        let metrics = self.collect_metrics().await?;
        Ok(metrics.health_status())
    }

    /// Get the endpoint URL
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Trait for compute metrics collection (capability-based)
pub trait ComputeMetricsProvider: Send + Sync {
    /// Collect current compute metrics
    async fn collect_compute_metrics(&self) -> SongbirdResult<ComputeMetrics>;

    /// Check compute service health
    async fn check_compute_health(&self) -> SongbirdResult<HealthStatus> {
        let metrics = self.collect_compute_metrics().await?;
        Ok(metrics.health_status())
    }
}

impl ComputeMetricsProvider for ComputeAdapter {
    async fn collect_compute_metrics(&self) -> SongbirdResult<ComputeMetrics> {
        self.collect_metrics().await
    }
}
