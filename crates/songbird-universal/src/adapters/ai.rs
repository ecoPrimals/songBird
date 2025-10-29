//! AI Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY AI capability provider.
//! It does NOT know about specific primals (Squirrel is just one example).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
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

/// **CAPABILITY-BASED AI ADAPTER**
///
/// Works with ANY AI provider discovered through:
/// - Environment variable: `SONGBIRD_AI_ENDPOINT`
/// - Capability discovery: `capability:ai`
/// - Zero-knowledge bootstrap
pub struct AIAdapter {
    /// Endpoint URL for the AI capability provider
    endpoint: String,
    /// HTTP client for requests
    client: reqwest::Client,
    /// Request timeout
    timeout: Duration,
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
    #[allow(clippy::unused_async)] // Will be async when ZeroKnowledgeBootstrap integration is complete
    pub async fn from_discovery() -> SongbirdResult<Self> {
        // Try environment variable first
        if let Ok(endpoint) = std::env::var("SONGBIRD_AI_ENDPOINT") {
            debug!("🔍 AI capability discovered via SONGBIRD_AI_ENDPOINT");
            return Self::new(endpoint);
        }

        // Fall back to capability discovery
        // TODO: Integrate with ZeroKnowledgeBootstrap for true infant discovery
        let endpoint =
            std::env::var("SONGBIRD_HOST").unwrap_or_else(|_| "http://localhost".to_string());
        let port = std::env::var("SONGBIRD_AI_PORT").unwrap_or_else(|_| "8083".to_string());
        let discovered_endpoint = format!("{endpoint}:{port}");

        debug!("🔍 AI capability discovered at: {}", discovered_endpoint);
        Self::new(discovered_endpoint)
    }

    /// Create adapter with explicit endpoint (for testing or explicit configuration)
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of any AI capability provider
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(endpoint: String) -> SongbirdResult<Self> {
        Ok(Self {
            endpoint,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30)) // AI operations may take longer
                .build()
                .map_err(|e| {
                    SongbirdError::configuration(format!("Failed to create HTTP client: {e}"))
                })?,
            timeout: Duration::from_secs(15),
        })
    }

    /// Set custom request timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Collect AI metrics from the capability provider
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Service returns non-success status
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<AIMetrics> {
        let url = format!("{}/metrics/ai", self.endpoint);

        debug!("Collecting AI metrics from: {}", url);

        let response = self.client.get(&url).timeout(self.timeout).send().await.map_err(|e| {
            warn!("Failed to reach AI capability provider: {e}");
            songbird_types::SongbirdError::network(format!("Failed to reach AI provider: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("AI capability provider returned error status: {}", status);
            return Err(songbird_types::SongbirdError::service(
                "ai",
                format!("HTTP {status}: AI metrics unavailable"),
            ));
        }

        let mut metrics: AIMetrics = response.json().await.map_err(|e| {
            warn!("Failed to parse AI metrics: {e}");
            songbird_types::SongbirdError::service("ai", format!("Failed to parse AI metrics: {e}"))
        })?;

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
#[async_trait]
pub trait AIProvider {
    /// Collect current AI metrics
    async fn collect_ai_metrics(&self) -> SongbirdResult<AIMetrics>;

    /// Check AI service health
    async fn check_ai_health(&self) -> SongbirdResult<AIHealth> {
        let metrics = self.collect_ai_metrics().await?;
        Ok(metrics.health_status())
    }
}

#[async_trait]
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
    fn test_ai_degraded() {
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
    }

    #[test]
    fn test_adapter_creation() {
        let adapter = AIAdapter::new("http://ai-provider:8083".to_string())
            .expect("Adapter creation should succeed");
        assert_eq!(adapter.endpoint(), "http://ai-provider:8083");
    }

    #[test]
    fn test_adapter_with_timeout() {
        let adapter = AIAdapter::new("http://ai-provider:8083".to_string())
            .expect("Adapter creation should succeed")
            .with_timeout(Duration::from_secs(20));
        assert_eq!(adapter.timeout, Duration::from_secs(20));
    }

    #[test]
    fn test_model_type_equality() {
        assert_eq!(ModelType::Llm, ModelType::Llm);
        assert_ne!(ModelType::Llm, ModelType::Vision);
        assert_eq!(ModelType::Audio, ModelType::Audio);
    }
}
