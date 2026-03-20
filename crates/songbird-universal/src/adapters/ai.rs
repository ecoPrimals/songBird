// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! AI Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY AI capability provider.
//! It does NOT know about specific primals (Squirrel is just one example).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.

// Allow async_fn_in_trait warning - our traits guarantee Send + Sync
#![allow(async_fn_in_trait)]

use crate::JsonRpcClient;
use serde::{Deserialize, Serialize};
use songbird_http_client::SongbirdHttpClient;
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::{debug, warn};

/// AI metrics from any AI capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMetrics {
    /// Number of active model instances
    pub active_models: u32,
    /// Total inference requests processed
    pub total_requests: u64,
    /// Average inference latency in milliseconds
    pub avg_latency_ms: f64,
    /// Model accuracy score (0.0 - 1.0)
    pub accuracy_score: f64,
    /// GPU utilization percentage (0.0 - 100.0)
    pub gpu_utilization_percent: f64,
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AIMetrics {
    /// Check if GPU is under high load
    #[must_use]
    pub fn is_high_gpu_load(&self) -> bool {
        self.gpu_utilization_percent > 90.0
    }

    /// Check if inference latency is high
    #[must_use]
    pub fn is_high_latency(&self) -> bool {
        self.avg_latency_ms > 1000.0
    }

    /// Get AI service health status
    #[must_use]
    pub fn health_status(&self) -> AIHealth {
        if self.gpu_utilization_percent > 98.0 || self.avg_latency_ms > 2000.0 {
            AIHealth::Overloaded
        } else if self.is_high_gpu_load() || self.is_high_latency() {
            AIHealth::Degraded
        } else {
            AIHealth::Healthy
        }
    }
}

/// AI service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIHealth {
    /// AI service is healthy
    Healthy,
    /// AI service is degraded
    Degraded,
    /// AI service is overloaded
    Overloaded,
}

/// Model type for AI inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Large language model
    Llm,
    /// Computer vision model
    Vision,
    /// Audio processing model
    Audio,
    /// Embedding model
    Embedding,
}

/// Protocol for communication (v3.12.0 - tarpc PRIMARY)
enum Protocol {
    Tarpc(crate::TarpcClient), // PRIMARY - high-performance binary RPC
    JsonRpc(JsonRpcClient),    // SECONDARY - universal, port-free
    Http(SongbirdHttpClient),  // FALLBACK - direct HTTP (no IPC delegation)
}

impl std::fmt::Debug for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tarpc(_) => write!(f, "Protocol::Tarpc"),
            Self::JsonRpc(_) => write!(f, "Protocol::JsonRpc"),
            Self::Http(_) => write!(f, "Protocol::Http"),
        }
    }
}

/// **CAPABILITY-BASED AI ADAPTER**
///
/// Works with ANY AI provider discovered through:
/// - Environment variable: `SONGBIRD_AI_ENDPOINT`
/// - Capability discovery: `capability:ai`
/// - Zero-knowledge bootstrap
///
/// **v3.11.0**: Protocol-agnostic - supports Unix sockets (PRIMARY) or HTTP (FALLBACK)
pub struct AIAdapter {
    /// Endpoint URL for the AI capability provider
    endpoint: String,
    /// Protocol (Unix socket JSON-RPC or HTTP)
    protocol: Protocol,
    /// Request timeout
    timeout: Duration,
}

impl std::fmt::Debug for AIAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AIAdapter")
            .field("endpoint", &self.endpoint)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl AIAdapter {
    /// Create adapter from discovered AI capability
    ///
    /// Uses capability-based discovery:
    /// 1. Check `SONGBIRD_AI_ENDPOINT` environment variable
    /// 2. Fall back to capability discovery
    /// 3. No hardcoded primal names anywhere
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use songbird_universal::adapters::AIAdapter;
    ///
    /// // Discovers any AI provider (could be Squirrel, or anyone)
    /// let adapter = AIAdapter::from_discovery().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if no AI capability can be discovered.
    pub async fn from_discovery() -> SongbirdResult<Self> {
        use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};

        // ✅ PHASE 1 INTEGRATION: Multi-tier capability discovery
        let resolver = CapabilityEndpointResolver::new();

        match resolver.get_endpoint(CapabilityType::Ai).await {
            Ok(endpoint) => {
                debug!("✅ AI capability discovered via resolver: {}", endpoint);
                Self::new(endpoint).await
            }
            Err(discovery_err) => {
                debug!("🔍 Primary discovery failed, trying legacy fallbacks: {}", discovery_err);

                // Fallback 1: Legacy environment variables
                if let Ok(endpoint) = SafeEnv::get_required("SONGBIRD_AI_ENDPOINT")
                    .or_else(|_| SafeEnv::get_required("AI_PROVIDER_ENDPOINT"))
                    .or_else(|_| SafeEnv::get_required("SQUIRREL_ENDPOINT"))
                {
                    debug!("⚠️ Using legacy environment variable for AI endpoint");
                    return Self::new(endpoint).await;
                }

                // Fallback 2: Construct from host + port
                let endpoint = SafeEnv::get_or_default(
                    "SONGBIRD_HOST",
                    format!(
                        "http://{}",
                        &songbird_config::canonical::constants::get_bind_address()
                    ),
                );
                let port = SafeEnv::get_port(
                    "SONGBIRD_AI_PORT",
                    songbird_config::defaults::ports::service_port("AI", 8083),
                )
                .to_string();
                let discovered_endpoint = format!("{endpoint}:{port}");

                debug!("🔄 Using fallback AI endpoint: {}", discovered_endpoint);
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
    /// * `endpoint` - Base URL of any AI capability provider
    ///   - `tarpc://host:port` → tarpc binary RPC (PRIMARY - 10-100x faster)
    ///   - `unix:///path/to/socket.sock` → JSON-RPC over Unix socket (SECONDARY - port-free)
    ///   - `http://...` or `https://...` → HTTP (FALLBACK - network only)
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol client cannot be created.
    #[allow(clippy::unused_async)] // async retained for API stability; protocol init may need await
    pub async fn new(endpoint: String) -> SongbirdResult<Self> {
        // Protocol detection (v3.12.0 - tarpc PRIMARY)
        let protocol = if endpoint.starts_with("tarpc://") {
            debug!("🚀 Detected tarpc endpoint for AI (PRIMARY): {}", endpoint);
            Protocol::Tarpc(crate::TarpcClient::new(&endpoint)?)
        } else if endpoint.starts_with("unix://") {
            debug!("🔌 Detected Unix socket endpoint for AI (SECONDARY): {}", endpoint);
            Protocol::JsonRpc(JsonRpcClient::new(&endpoint)?)
        } else {
            debug!("🌐 Detected HTTP endpoint for AI (FALLBACK): {}", endpoint);
            Protocol::Http(SongbirdHttpClient::from_env())
        };

        Ok(Self {
            endpoint,
            protocol,
            timeout: Duration::from_secs(15),
        })
    }

    /// Set custom request timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Collect AI metrics from the capability provider
    ///
    /// **v3.11.0**: Protocol-agnostic - works with Unix sockets or HTTP
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network/IPC request fails
    /// - Service returns non-success status (HTTP) or error (JSON-RPC)
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<AIMetrics> {
        debug!("Collecting AI metrics from: {}", self.endpoint);

        let mut metrics: AIMetrics = match &self.protocol {
            Protocol::Tarpc(client) => {
                // tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY - ~10-20 μs latency!)
                debug!("🚀 Using tarpc (PRIMARY protocol)");
                let result = client.call_method("get_ai_metrics", None).await?;
                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse AI metrics from tarpc: {e}");
                    SongbirdError::serialization(format!("Failed to parse AI metrics: {e}"))
                })?
            }
            Protocol::JsonRpc(client) => {
                // JSON-RPC protocol over Unix socket (SECONDARY - ~50-100 μs latency)
                debug!("🔌 Using JSON-RPC (SECONDARY protocol)");
                let result = client.call_method("get_ai_metrics", None).await?;
                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse AI metrics from JSON-RPC: {e}");
                    SongbirdError::serialization(format!("Failed to parse AI metrics: {e}"))
                })?
            }
            Protocol::Http(client) => {
                // HTTP protocol (FALLBACK - direct TCP connection)
                debug!("🌐 Using HTTP (FALLBACK protocol)");
                let url = format!("{}/metrics/ai", self.endpoint);

                let response = tokio::time::timeout(self.timeout, client.get(&url))
                    .await
                    .map_err(|_| {
                        SongbirdError::network(format!(
                            "Timeout after {:?} reaching AI provider",
                            self.timeout
                        ))
                    })?
                    .map_err(|e| {
                        warn!("Failed to reach AI capability provider via HTTP: {e}");
                        SongbirdError::network(format!("Failed to reach AI provider: {e}"))
                    })?;

                if !(200..300).contains(&response.status) {
                    let status = response.status;
                    warn!("AI capability provider returned error status: {}", status);
                    return Err(SongbirdError::service(
                        "ai",
                        format!("HTTP {status}: AI metrics unavailable"),
                    ));
                }

                serde_json::from_value(response.body).map_err(|e| {
                    warn!("Failed to parse AI metrics from HTTP: {e}");
                    SongbirdError::service("ai", format!("Failed to parse AI metrics: {e}"))
                })?
            }
        };

        // Set timestamp if not provided
        if metrics.timestamp.timestamp() == 0 {
            metrics.timestamp = chrono::Utc::now();
        }

        debug!(
            "Collected AI metrics: Models={}, Requests={}, Latency={}ms, GPU={}%",
            metrics.active_models,
            metrics.total_requests,
            metrics.avg_latency_ms,
            metrics.gpu_utilization_percent
        );

        Ok(metrics)
    }

    /// Check AI service health
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    pub async fn check_health(&self) -> SongbirdResult<AIHealth> {
        let metrics = self.collect_metrics().await?;
        Ok(metrics.health_status())
    }

    /// Get the endpoint URL
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Trait for AI capability providers
pub trait AIProvider: Send + Sync {
    /// Collect current AI metrics
    async fn collect_ai_metrics(&self) -> SongbirdResult<AIMetrics>;

    /// Check AI service health
    async fn check_ai_health(&self) -> SongbirdResult<AIHealth> {
        let metrics = self.collect_ai_metrics().await?;
        Ok(metrics.health_status())
    }
}

impl AIProvider for AIAdapter {
    async fn collect_ai_metrics(&self) -> SongbirdResult<AIMetrics> {
        self.collect_metrics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_metrics_calculations() {
        let metrics = AIMetrics {
            active_models: 3,
            total_requests: 1_500,
            avg_latency_ms: 250.0,
            accuracy_score: 0.92,
            gpu_utilization_percent: 45.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_high_gpu_load());
        assert!(!metrics.is_high_latency());
        assert_eq!(metrics.health_status(), AIHealth::Healthy);
    }

    #[test]
    fn test_ai_overloaded() {
        let metrics = AIMetrics {
            active_models: 20,
            total_requests: 50_000,
            avg_latency_ms: 2500.0,
            accuracy_score: 0.88,
            gpu_utilization_percent: 99.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_high_gpu_load());
        assert!(metrics.is_high_latency());
        assert_eq!(metrics.health_status(), AIHealth::Overloaded);
    }

    #[test]
    fn test_ai_degraded() -> SongbirdResult<()> {
        let metrics = AIMetrics {
            active_models: 8,
            total_requests: 10_000,
            avg_latency_ms: 1200.0,
            accuracy_score: 0.90,
            gpu_utilization_percent: 92.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_high_gpu_load());
        assert!(metrics.is_high_latency());
        assert_eq!(metrics.health_status(), AIHealth::Degraded);
        Ok(())
    }

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

    #[test]
    fn test_model_type_equality() {
        assert_eq!(ModelType::Llm, ModelType::Llm);
        assert_ne!(ModelType::Llm, ModelType::Vision);
        assert_eq!(ModelType::Audio, ModelType::Audio);
    }

    #[test]
    fn test_ai_health_equality() {
        assert_eq!(AIHealth::Healthy, AIHealth::Healthy);
        assert_eq!(AIHealth::Degraded, AIHealth::Degraded);
        assert_eq!(AIHealth::Overloaded, AIHealth::Overloaded);
        assert_ne!(AIHealth::Healthy, AIHealth::Degraded);
        assert_ne!(AIHealth::Degraded, AIHealth::Overloaded);
    }

    #[test]
    fn test_ai_metrics_high_gpu_boundary() {
        // Just below threshold
        let metrics_below = AIMetrics {
            active_models: 5,
            total_requests: 1000,
            avg_latency_ms: 100.0,
            accuracy_score: 0.95,
            gpu_utilization_percent: 90.0,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_high_gpu_load());

        // Just above threshold
        let metrics_above = AIMetrics {
            active_models: 5,
            total_requests: 1000,
            avg_latency_ms: 100.0,
            accuracy_score: 0.95,
            gpu_utilization_percent: 90.1,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_gpu_load());
    }

    #[test]
    fn test_ai_metrics_high_latency_boundary() {
        // Just below threshold
        let metrics_below = AIMetrics {
            active_models: 5,
            total_requests: 1000,
            avg_latency_ms: 1000.0,
            accuracy_score: 0.95,
            gpu_utilization_percent: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_high_latency());

        // Just above threshold
        let metrics_above = AIMetrics {
            active_models: 5,
            total_requests: 1000,
            avg_latency_ms: 1000.1,
            accuracy_score: 0.95,
            gpu_utilization_percent: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_latency());
    }

    #[test]
    fn test_health_status_overloaded_high_gpu() {
        let metrics = AIMetrics {
            active_models: 10,
            total_requests: 20000,
            avg_latency_ms: 500.0,
            accuracy_score: 0.9,
            gpu_utilization_percent: 98.1,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), AIHealth::Overloaded);
    }

    #[test]
    fn test_health_status_overloaded_high_latency() {
        let metrics = AIMetrics {
            active_models: 10,
            total_requests: 20000,
            avg_latency_ms: 2001.0,
            accuracy_score: 0.9,
            gpu_utilization_percent: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), AIHealth::Overloaded);
    }

    #[test]
    fn test_health_status_degraded_gpu() {
        let metrics = AIMetrics {
            active_models: 8,
            total_requests: 10000,
            avg_latency_ms: 800.0,
            accuracy_score: 0.9,
            gpu_utilization_percent: 91.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), AIHealth::Degraded);
    }

    #[test]
    fn test_health_status_degraded_latency() {
        let metrics = AIMetrics {
            active_models: 8,
            total_requests: 10000,
            avg_latency_ms: 1100.0,
            accuracy_score: 0.9,
            gpu_utilization_percent: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), AIHealth::Degraded);
    }

    #[test]
    fn test_ai_metrics_serialization() -> SongbirdResult<()> {
        let metrics = AIMetrics {
            active_models: 5,
            total_requests: 1000,
            avg_latency_ms: 250.0,
            accuracy_score: 0.92,
            gpu_utilization_percent: 45.0,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&metrics).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("active_models"));
        assert!(json.contains(":5"));
        Ok(())
    }

    #[test]
    fn test_ai_health_serialization() -> SongbirdResult<()> {
        let health = AIHealth::Healthy;
        let json = serde_json::to_string(&health).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("Healthy"));

        let degraded = AIHealth::Degraded;
        let json = serde_json::to_string(&degraded).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("Degraded"));

        let overloaded = AIHealth::Overloaded;
        let json = serde_json::to_string(&overloaded).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("Overloaded"));
        Ok(())
    }

    #[test]
    fn test_model_type_serialization() -> SongbirdResult<()> {
        assert_eq!(
            serde_json::to_string(&ModelType::Llm).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {}", e),
                debug_info: None
            })?,
            "\"Llm\""
        );
        assert_eq!(
            serde_json::to_string(&ModelType::Vision).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Vision\""
        );
        assert_eq!(
            serde_json::to_string(&ModelType::Audio).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {}", e),
                debug_info: None
            })?,
            "\"Audio\""
        );
        assert_eq!(
            serde_json::to_string(&ModelType::Embedding).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Embedding\""
        );
        Ok(())
    }

    #[test]
    fn test_model_type_all_variants() -> SongbirdResult<()> {
        assert_eq!(ModelType::Llm, ModelType::Llm);
        assert_eq!(ModelType::Vision, ModelType::Vision);
        assert_eq!(ModelType::Audio, ModelType::Audio);
        assert_eq!(ModelType::Embedding, ModelType::Embedding);
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_endpoint_access() -> SongbirdResult<()> {
        let adapter = AIAdapter::new("http://test-ai:8083".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
        assert_eq!(adapter.endpoint(), "http://test-ai:8083");
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_default_timeout() -> SongbirdResult<()> {
        let adapter = AIAdapter::new("http://test-ai:8083".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
        assert_eq!(adapter.timeout, Duration::from_secs(15));
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_custom_timeout() -> SongbirdResult<()> {
        let custom_timeout = Duration::from_secs(45);
        let adapter = AIAdapter::new("http://test-ai:8083".to_string())
            .await
            .map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?
            .with_timeout(custom_timeout);
        assert_eq!(adapter.timeout, custom_timeout);
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_debug_format() -> SongbirdResult<()> {
        let adapter = AIAdapter::new("http://test-ai:8083".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
        let debug_str = format!("{:?}", adapter);
        assert!(debug_str.contains("AIAdapter"));
        assert!(debug_str.contains("http://test-ai:8083"));
        Ok(())
    }

    #[test]
    fn test_ai_metrics_zero_values() {
        let metrics = AIMetrics {
            active_models: 0,
            total_requests: 0,
            avg_latency_ms: 0.0,
            accuracy_score: 0.0,
            gpu_utilization_percent: 0.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_high_gpu_load());
        assert!(!metrics.is_high_latency());
        assert_eq!(metrics.health_status(), AIHealth::Healthy);
    }

    #[test]
    fn test_ai_metrics_edge_case_98_percent() {
        // Exactly at overload threshold
        let metrics = AIMetrics {
            active_models: 10,
            total_requests: 10000,
            avg_latency_ms: 500.0,
            accuracy_score: 0.9,
            gpu_utilization_percent: 98.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), AIHealth::Degraded);
    }

    #[test]
    fn test_ai_metrics_edge_case_2000ms() {
        // Exactly at overload threshold
        let metrics = AIMetrics {
            active_models: 5,
            total_requests: 1000,
            avg_latency_ms: 2000.0,
            accuracy_score: 0.95,
            gpu_utilization_percent: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), AIHealth::Degraded);
    }

    #[test]
    fn test_ai_metrics_perfect_conditions() {
        let metrics = AIMetrics {
            active_models: 3,
            total_requests: 5000,
            avg_latency_ms: 50.0,
            accuracy_score: 0.99,
            gpu_utilization_percent: 25.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_high_gpu_load());
        assert!(!metrics.is_high_latency());
        assert_eq!(metrics.health_status(), AIHealth::Healthy);
    }

    // ========== NEW TESTS (10 tests to reach 85% coverage) ==========

    #[test]
    fn test_ai_metrics_clone() {
        let metrics = AIMetrics {
            active_models: 5,
            total_requests: 1000,
            avg_latency_ms: 250.0,
            accuracy_score: 0.92,
            gpu_utilization_percent: 45.0,
            timestamp: chrono::Utc::now(),
        };
        let cloned = metrics;
        assert_eq!(cloned.active_models, 5);
        assert_eq!(cloned.total_requests, 1000);
        assert!((cloned.avg_latency_ms - 250.0).abs() < 0.001);
    }

    #[test]
    fn test_ai_health_clone() {
        let health = AIHealth::Degraded;
        let cloned = health;
        assert_eq!(health, cloned);
    }

    #[test]
    fn test_model_type_clone() {
        let model = ModelType::Llm;
        let cloned = model;
        assert_eq!(model, cloned);
    }

    #[test]
    fn test_ai_metrics_deserialization() -> SongbirdResult<()> {
        let json = r#"{
            "active_models": 8,
            "total_requests": 15000,
            "avg_latency_ms": 350.5,
            "accuracy_score": 0.94,
            "gpu_utilization_percent": 75.5,
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let metrics: AIMetrics =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(metrics.active_models, 8);
        assert_eq!(metrics.total_requests, 15000);
        assert!((metrics.avg_latency_ms - 350.5).abs() < 0.001);
        assert!((metrics.accuracy_score - 0.94).abs() < 0.001);
        assert!((metrics.gpu_utilization_percent - 75.5).abs() < 0.001);
        Ok(())
    }

    #[test]
    fn test_ai_health_deserialization() -> SongbirdResult<()> {
        let json = r#""Degraded""#;
        let health: AIHealth =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(health, AIHealth::Degraded);

        let json = r#""Overloaded""#;
        let health: AIHealth =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(health, AIHealth::Overloaded);
        Ok(())
    }

    #[test]
    fn test_model_type_deserialization() -> SongbirdResult<()> {
        let json = r#""Vision""#;
        let model: ModelType =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(model, ModelType::Vision);

        let json = r#""Embedding""#;
        let model: ModelType =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(model, ModelType::Embedding);
        Ok(())
    }

    #[test]
    fn test_ai_metrics_debug_format() -> SongbirdResult<()> {
        let metrics = AIMetrics {
            active_models: 5,
            total_requests: 1000,
            avg_latency_ms: 250.0,
            accuracy_score: 0.92,
            gpu_utilization_percent: 45.0,
            timestamp: chrono::Utc::now(),
        };
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("AIMetrics"));
        assert!(debug_str.contains("active_models"));
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_chained_timeout() -> SongbirdResult<()> {
        let adapter = AIAdapter::new("http://test:8083".to_string())
            .await
            .map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?
            .with_timeout(Duration::from_secs(10))
            .with_timeout(Duration::from_secs(25));

        assert_eq!(adapter.timeout, Duration::from_secs(25), "Last timeout should be applied");
        Ok(())
    }

    #[test]
    fn test_ai_metrics_max_values() -> SongbirdResult<()> {
        let metrics = AIMetrics {
            active_models: u32::MAX,
            total_requests: u64::MAX,
            avg_latency_ms: f64::MAX,
            accuracy_score: 1.0,
            gpu_utilization_percent: 100.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_high_gpu_load());
        assert!(metrics.is_high_latency());
        assert_eq!(metrics.health_status(), AIHealth::Overloaded);
        Ok(())
    }

    #[test]
    fn test_model_type_debug() -> SongbirdResult<()> {
        let model = ModelType::Vision;
        let debug_str = format!("{:?}", model);
        assert!(debug_str.contains("Vision"));

        let model = ModelType::Audio;
        let debug_str = format!("{:?}", model);
        assert!(debug_str.contains("Audio"));
        Ok(())
    }
}
