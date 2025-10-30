//! Mock Squirrel AI Primal
//!
//! Provides HTTP endpoints that simulate Squirrel's AI integration and MCP protocol capabilities.

#![allow(clippy::unused_async)]

use super::common::{HealthStatus, MockPrimalServer, MockServerState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// AI model type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum ModelType {
    /// Large language model
    LLM,
    /// Computer vision model
    Vision,
    /// Audio processing model
    Audio,
    /// Embedding model
    Embedding,
}

/// AI inference metrics
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
    /// GPU utilization percentage
    pub gpu_utilization_percent: f64,
}

impl Default for AIMetrics {
    fn default() -> Self {
        Self {
            active_models: 3,
            total_requests: 1_500,
            avg_latency_ms: 250.0,
            accuracy_score: 0.92,
            gpu_utilization_percent: 45.0,
        }
    }
}

/// AI inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Model to use for inference
    pub model_type: ModelType,
    /// Input data
    pub input: String,
    /// Optional parameters
    pub parameters: HashMap<String, String>,
}

/// AI inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Response output
    pub output: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Latency in milliseconds
    pub latency_ms: f64,
}

/// Mock Squirrel AI server
#[derive(Debug, Clone)]
pub struct MockSquirrel {
    state: Arc<MockServerState>,
    ai_metrics: Arc<RwLock<AIMetrics>>,
    responses: Arc<RwLock<HashMap<String, InferenceResponse>>>,
}

impl MockSquirrel {
    /// Create a new mock Squirrel server
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(MockServerState::new(0)),
            ai_metrics: Arc::new(RwLock::new(AIMetrics::default())),
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the mock server
    ///
    /// # Errors
    ///
    /// Currently never returns an error, but signature allows for future error cases.
    pub async fn start(&mut self) -> Result<u16, Box<dyn std::error::Error>> {
        let port = fastrand::u16(10000..60000);
        self.state = Arc::new(MockServerState::new(port));
        Ok(port)
    }

    /// Stop the mock server
    pub async fn stop(&self) {
        // Server cleanup
    }

    /// Configure a canned response for a specific input
    ///
    /// # Panics
    ///
    /// Panics if the internal responses lock is poisoned.
    pub fn set_response(&self, input: impl Into<String>, response: InferenceResponse) {
        let mut responses = self.responses.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        responses.insert(input.into(), response);
    }

    /// Perform inference (simulated)
    ///
    /// # Panics
    ///
    /// Panics if the internal locks are poisoned.
    #[must_use]
    pub fn infer(&self, request: &InferenceRequest) -> Option<InferenceResponse> {
        self.state.increment_requests();

        let responses = self.responses.read().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        let response = responses.get(&request.input).cloned();

        // Update metrics
        let mut metrics = self.ai_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.total_requests += 1;

        response
    }

    /// Set GPU utilization
    ///
    /// # Panics
    ///
    /// Panics if the internal AI metrics lock is poisoned.
    pub fn set_gpu_utilization(&self, percent: f64) {
        let mut metrics = self.ai_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.gpu_utilization_percent = percent.clamp(0.0, 100.0);
    }

    /// Get AI metrics
    ///
    /// # Panics
    ///
    /// Panics if the internal AI metrics lock is poisoned.
    #[must_use]
    pub fn get_metrics(&self) -> AIMetrics {
        self.ai_metrics
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned in test mock, recovering");
                poisoned.into_inner()
            })
            .clone()
    }

    /// Simulate high AI load
    ///
    /// # Panics
    ///
    /// Panics if the internal AI metrics lock is poisoned.
    pub fn simulate_high_load(&self) {
        let mut metrics = self.ai_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.active_models = 15;
        metrics.avg_latency_ms = 1500.0;
        metrics.gpu_utilization_percent = 98.0;
        self.state.set_health(HealthStatus::Degraded);
    }

    /// Simulate normal AI operation
    ///
    /// # Panics
    ///
    /// Panics if the internal AI metrics lock is poisoned.
    pub fn simulate_normal_operation(&self) {
        let mut metrics = self.ai_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.active_models = 3;
        metrics.avg_latency_ms = 250.0;
        metrics.gpu_utilization_percent = 45.0;
        self.state.set_health(HealthStatus::Healthy);
    }
}

impl Default for MockSquirrel {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPrimalServer for MockSquirrel {
    fn port(&self) -> u16 {
        self.state.port
    }

    fn set_health(&self, status: HealthStatus) {
        self.state.set_health(status);
    }

    fn get_health(&self) -> HealthStatus {
        self.state.get_health()
    }
}

#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::*;

    #[tokio::test]
    async fn test_mock_squirrel_inference() {
        let mock = MockSquirrel::new();

        // Configure canned response
        let response = InferenceResponse {
            output: "Test output".to_string(),
            confidence: 0.95,
            latency_ms: 200.0,
        };
        mock.set_response("test input", response.clone());

        // Test inference
        let request = InferenceRequest {
            model_type: ModelType::LLM,
            input: "test input".to_string(),
            parameters: HashMap::new(),
        };

        let result = mock.infer(&request);
        assert!(result.is_some());
        if let Some(response) = result {
            assert_eq!(response.output, "Test output");
        }
    }

    #[tokio::test]
    async fn test_mock_squirrel_scenarios() {
        let mock = MockSquirrel::new();

        // Test high load
        mock.simulate_high_load();
        let metrics = mock.get_metrics();
        assert!(metrics.gpu_utilization_percent > 95.0);
        assert_eq!(mock.get_health(), HealthStatus::Degraded);

        // Test normal operation
        mock.simulate_normal_operation();
        let metrics = mock.get_metrics();
        assert!(metrics.gpu_utilization_percent < 50.0);
        assert_eq!(mock.get_health(), HealthStatus::Healthy);
    }
}
