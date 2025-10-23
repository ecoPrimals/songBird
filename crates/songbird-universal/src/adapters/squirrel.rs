//! `Squirrel` AI Adapter
//!
//! Coordinates with `Squirrel` primal for AI, inference, and MCP protocol operations.
//! This adapter is capability-based and works with any service providing AI
//! capabilities in the expected format.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::{debug, warn};

/// AI metrics from `Squirrel` or any AI primal
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

/// Adapter for `Squirrel` AI coordination
pub struct SquirrelAIAdapter {
    /// Endpoint URL for the AI service
    endpoint: String,
    /// HTTP client for requests
    client: reqwest::Client,
    /// Request timeout
    timeout: Duration,
}

impl SquirrelAIAdapter {
    /// Create a new `Squirrel` AI adapter with default endpoint from configuration
    ///
    /// Uses environment variables for endpoint configuration:
    /// - `SQUIRREL_ENDPOINT` - Direct endpoint override
    /// - `PRIMAL_SQUIRREL_ENDPOINT` - Alternative format
    /// - Falls back to `$SONGBIRD_HOST:$SQUIRREL_PORT`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use songbird_universal::adapters::SquirrelAIAdapter;
    ///
    /// // Uses environment-configured endpoint
    /// let adapter = SquirrelAIAdapter::new_default().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new_default() -> SongbirdResult<Self> {
        let endpoint = songbird_config::endpoints::get_primal_endpoint("squirrel");
        Self::new(endpoint)
    }

    /// Create a new `Squirrel` AI adapter with custom endpoint
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of the AI service
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use songbird_universal::adapters::SquirrelAIAdapter;
    ///
    /// // Custom endpoint
    /// let adapter = SquirrelAIAdapter::new("http://ai-service:8083".to_string()).unwrap();
    ///
    /// // Or use default from configuration
    /// let adapter = SquirrelAIAdapter::new_default().unwrap();
    /// ```
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

    /// Collect AI metrics from the service
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
            warn!("Failed to reach AI service: {e}");
            songbird_types::SongbirdError::network(format!("Failed to reach AI service: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("AI service returned error status: {}", status);
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

/// Trait for AI coordination
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
impl AIProvider for SquirrelAIAdapter {
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
        // Test uses localhost - acceptable for unit tests
        let adapter = SquirrelAIAdapter::new("http://localhost:8083".to_string())
            .expect("Test: adapter creation should succeed");
        assert_eq!(
            adapter.endpoint(), // Test uses localhost - acceptable for unit tests
            "http://localhost:8083"
        );
    }

    #[test]
    fn test_adapter_with_timeout() {
        let adapter = SquirrelAIAdapter::new(
            // Test uses localhost - acceptable for unit tests
            "http://localhost:8083".to_string(),
        )
        .expect("Test: adapter creation should succeed")
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
