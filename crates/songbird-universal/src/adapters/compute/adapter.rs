// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::super::metrics::{ComputeMetrics, HealthStatus};
    use super::{ComputeAdapter, ComputeMetricsProvider};
    use crate::adapters::transport::{AdapterTransportKind, DelayTransport, MockTransport};
    use serde_json::json;
    use songbird_config::capability_endpoints::CapabilityEndpointResolver;
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tracing_subscriber::layer::SubscriberExt;

    fn discovery_env_guard() -> std::sync::MutexGuard<'static, ()> {
        crate::adapters::discovery_test_sync::lock_discovery_env()
    }

    #[tokio::test]
    async fn new_propagates_build_default_transport_unix_empty_path_err() {
        let err = ComputeAdapter::new("unix://".to_string()).await.expect_err("empty unix path");
        assert!(
            err.to_string().contains("Empty socket path")
                || err.to_string().contains("configuration"),
            "unexpected: {err}"
        );
    }

    #[tokio::test]
    async fn new_propagates_build_default_transport_invalid_tarpc_err() {
        let err = ComputeAdapter::new("tarpc://not-a-host:99999".to_string())
            .await
            .expect_err("invalid tarpc endpoint");
        assert!(
            err.to_string().contains("tarpc") || err.to_string().contains("configuration"),
            "unexpected: {err}"
        );
    }

    #[tokio::test]
    async fn new_accepts_nonstandard_scheme_as_http_transport() -> SongbirdResult<()> {
        let adapter = ComputeAdapter::new("ftp://compute.example/metrics".to_string()).await?;
        assert_eq!(adapter.endpoint(), "ftp://compute.example/metrics");
        let dbg = format!("{:?}", adapter);
        assert!(dbg.contains("Protocol::Http"), "{}", dbg);
        Ok(())
    }

    #[tokio::test]
    async fn debug_shows_tarpc_and_jsonrpc_labels() -> SongbirdResult<()> {
        let tarpc = ComputeAdapter::new("tarpc://127.0.0.1:9100".to_string()).await?;
        assert!(format!("{:?}", tarpc).contains("Protocol::Tarpc"));
        let uds =
            ComputeAdapter::new("unix:///tmp/songbird-compute-adapter-tests.sock".to_string())
                .await?;
        assert!(format!("{:?}", uds).contains("Protocol::JsonRpc"));
        Ok(())
    }

    #[tokio::test]
    async fn collect_metrics_times_out() {
        let delayed = DelayTransport {
            inner: Arc::new(MockTransport::new(vec![])),
            delay: Duration::from_secs(30),
        };
        let adapter = ComputeAdapter::with_transport(
            "http://mock-compute".to_string(),
            Arc::new(delayed),
            AdapterTransportKind::Http,
            Duration::from_millis(20),
        );
        let err = adapter.collect_metrics().await.expect_err("should time out");
        assert!(err.to_string().to_lowercase().contains("timeout"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn collect_metrics_http_transport_error_passes_through() {
        let boom = SongbirdError::network("upstream http failure");
        let adapter = ComputeAdapter::with_transport(
            "http://mock".to_string(),
            Arc::new(MockTransport::new(vec![Err(boom.clone())])),
            AdapterTransportKind::Http,
            Duration::from_secs(5),
        );
        let err = adapter.collect_metrics().await.expect_err("transport error");
        assert_eq!(err.to_string(), boom.to_string());
    }

    #[tokio::test]
    async fn collect_metrics_non_http_transport_error_is_wrapped() {
        let boom = SongbirdError::network("rpc down");
        let adapter = ComputeAdapter::with_transport(
            "tarpc://127.0.0.1:1".to_string(),
            Arc::new(MockTransport::new(vec![Err(boom)])),
            AdapterTransportKind::Tarpc,
            Duration::from_secs(5),
        );
        let err = adapter.collect_metrics().await.expect_err("wrapped");
        let s = err.to_string();
        assert!(s.contains("Failed to reach compute service"), "{}", s);
    }

    #[tokio::test]
    async fn collect_metrics_http_parse_error_maps_to_service() {
        let adapter = ComputeAdapter::with_transport(
            "http://mock".to_string(),
            Arc::new(MockTransport::new(vec![Ok(json!("not-metrics"))])),
            AdapterTransportKind::Http,
            Duration::from_secs(5),
        );
        let err = adapter.collect_metrics().await.expect_err("bad json shape");
        let s = err.to_string();
        assert!(s.contains("compute") || s.contains("parse"), "{}", s);
    }

    #[tokio::test]
    async fn collect_metrics_tarpc_parse_error_maps_to_serialization() {
        let adapter = ComputeAdapter::with_transport(
            "tarpc://127.0.0.1:1".to_string(),
            Arc::new(MockTransport::new(vec![Ok(json!("nope"))])),
            AdapterTransportKind::Tarpc,
            Duration::from_secs(5),
        );
        let err = adapter.collect_metrics().await.expect_err("serde");
        let s = err.to_string();
        assert!(s.to_lowercase().contains("serial") || s.contains("parse"), "{}", s);
    }

    #[tokio::test]
    async fn collect_metrics_sets_timestamp_when_unix_epoch() -> SongbirdResult<()> {
        let body = json!({
            "cpu_usage_percent": 1.0,
            "memory_usage_bytes": 1,
            "memory_available_bytes": 1,
            "active_containers": 0,
            "queued_jobs": 0,
            "performance_score": 1.0,
            "timestamp": "1970-01-01T00:00:00Z"
        });
        let adapter = ComputeAdapter::with_transport(
            "http://mock".to_string(),
            Arc::new(MockTransport::new(vec![Ok(body)])),
            AdapterTransportKind::Http,
            Duration::from_secs(5),
        );
        let m = adapter.collect_metrics().await?;
        assert_ne!(m.timestamp.timestamp(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn check_health_delegates_to_collect_metrics() -> SongbirdResult<()> {
        let body = serde_json::to_value(ComputeMetrics {
            cpu_usage_percent: 10.0,
            memory_usage_bytes: 1,
            memory_available_bytes: 9,
            active_containers: 1,
            queued_jobs: 0,
            performance_score: 1.0,
            timestamp: chrono::Utc::now(),
        })?;
        let adapter = ComputeAdapter::with_transport(
            "http://mock".to_string(),
            Arc::new(MockTransport::new(vec![Ok(body)])),
            AdapterTransportKind::Http,
            Duration::from_secs(5),
        );
        assert_eq!(adapter.check_health().await?, HealthStatus::Healthy);
        Ok(())
    }

    #[tokio::test]
    async fn compute_metrics_provider_trait_forwards() -> SongbirdResult<()> {
        let body = serde_json::to_value(ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1,
            memory_available_bytes: 1,
            active_containers: 0,
            queued_jobs: 0,
            performance_score: 0.5,
            timestamp: chrono::Utc::now(),
        })?;
        let adapter = ComputeAdapter::with_transport(
            "http://mock".to_string(),
            Arc::new(MockTransport::new(vec![Ok(body)])),
            AdapterTransportKind::Http,
            Duration::from_secs(5),
        );
        let m = adapter.collect_compute_metrics().await?;
        assert_eq!(m.cpu_usage_percent, 50.0);
        Ok(())
    }

    #[tokio::test]
    async fn discovery_fallback_compute_endpoint_env() -> SongbirdResult<()> {
        let _g = discovery_env_guard();
        songbird_process_env::reset_overlay();
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
        songbird_process_env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
        songbird_process_env::remove_var("COMPUTE_CAPABILITY_ENDPOINT");
        songbird_process_env::set_var("COMPUTE_ENDPOINT", "http://from-compute-endpoint-only:7700");

        let adapter =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await?;
        assert_eq!(adapter.endpoint(), "http://from-compute-endpoint-only:7700");

        songbird_process_env::reset_overlay();
        Ok(())
    }

    #[tokio::test]
    async fn discovery_fallback_default_host_and_compute_port() -> SongbirdResult<()> {
        let _g = discovery_env_guard();
        songbird_process_env::reset_overlay();
        for key in [
            "CAPABILITY_COMPUTE_ENDPOINT",
            "SONGBIRD_COMPUTE_ENDPOINT",
            "COMPUTE_CAPABILITY_ENDPOINT",
            "COMPUTE_ENDPOINT",
            "TOADSTOOL_ENDPOINT",
            "SONGBIRD_HOST",
            "SONGBIRD_COMPUTE_PORT",
        ] {
            songbird_process_env::remove_var(key);
        }

        let adapter =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await?;
        assert_eq!(adapter.endpoint(), "http://localhost:8080");

        songbird_process_env::reset_overlay();
        Ok(())
    }

    #[tokio::test]
    async fn discovery_fallback_propagates_new_error_from_bad_env_endpoint() {
        let _g = discovery_env_guard();
        songbird_process_env::reset_overlay();
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
        songbird_process_env::set_var("SONGBIRD_COMPUTE_ENDPOINT", "unix://");

        let err =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await
                .expect_err("adapter new should fail");
        assert!(
            err.to_string().contains("Empty socket path")
                || err.to_string().contains("configuration"),
            "unexpected: {err}"
        );

        songbird_process_env::reset_overlay();
    }

    #[tokio::test]
    async fn toadstool_endpoint_logs_deprecation_warning() -> SongbirdResult<()> {
        #[derive(Clone)]
        struct BufWriter(Arc<Mutex<Vec<u8>>>);
        impl Write for BufWriter {
            fn write(&mut self, d: &[u8]) -> std::io::Result<usize> {
                self.0
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .extend_from_slice(d);
                Ok(d.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let _g = discovery_env_guard();
        songbird_process_env::reset_overlay();
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
        for key in ["SONGBIRD_COMPUTE_ENDPOINT", "COMPUTE_CAPABILITY_ENDPOINT", "COMPUTE_ENDPOINT"]
        {
            songbird_process_env::remove_var(key);
        }
        songbird_process_env::set_var("TOADSTOOL_ENDPOINT", "http://from-toadstool-warn:6611");

        let log_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let w = Arc::clone(&log_buf);
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_level(false)
                .with_ansi(false)
                .with_writer(move || BufWriter(Arc::clone(&w))),
        );
        let _trace_guard = tracing::subscriber::set_default(subscriber);

        let adapter =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await?;
        assert_eq!(adapter.endpoint(), "http://from-toadstool-warn:6611");
        drop(_trace_guard);
        let logs = String::from_utf8_lossy(
            &log_buf.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(),
        )
        .into_owned();
        assert!(
            logs.contains("TOADSTOOL_ENDPOINT") && logs.contains("deprecated"),
            "logs were: {logs}"
        );

        songbird_process_env::reset_overlay();
        Ok(())
    }
}
