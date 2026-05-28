// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Mock AI capability provider (HTTP test harness)
//!
//! Provides HTTP endpoints that simulate an AI provider's integration and MCP protocol capabilities.

#![expect(clippy::unused_async, reason = "unused bindings/imports in this compilation unit")]

use super::common::{HealthStatus, MockPrimalServer, MockServerState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// AI model type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Mock AI capability provider server
#[derive(Debug, Clone)]
pub struct MockAiProvider {
    state: Arc<MockServerState>,
    ai_metrics: Arc<RwLock<AIMetrics>>,
    responses: Arc<RwLock<HashMap<String, InferenceResponse>>>,
}

impl MockAiProvider {
    /// Create a new mock AI provider server
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
    pub async fn start(&mut self) -> anyhow::Result<u16> {
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

        let response = self
            .responses
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned in test mock, recovering");
                poisoned.into_inner()
            })
            .get(&request.input)
            .cloned();

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
        drop(metrics);
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
        drop(metrics);
        self.state.set_health(HealthStatus::Healthy);
    }
}

impl Default for MockAiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPrimalServer for MockAiProvider {
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

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
#[expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#[expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#[expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#[expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#[expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#[expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#[allow(
    clippy::cast_possible_truncation,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "intentional pattern; clippy false positive for this API"
)]
mod tests {
    #![allow(clippy::all, reason = "test assertions and harness ergonomics")]
    #![allow(unused, reason = "test assertions and harness ergonomics")]
    use super::*;
    use songbird_types::SongbirdError;

    #[tokio::test]
    async fn test_mock_ai_provider_inference() {
        let mock = MockAiProvider::new();

        // Configure canned response
        let response = InferenceResponse {
            output: "Test output".to_string(),
            confidence: 0.95,
            latency_ms: 200.0,
        };
        mock.set_response("test input", response);

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
    async fn test_mock_ai_provider_scenarios() {
        let mock = MockAiProvider::new();

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

    // ========== NEW TESTS (5 tests to improve coverage) ==========

    #[tokio::test]
    async fn test_ai_provider_server_lifecycle() -> anyhow::Result<()> {
        let mut mock = MockAiProvider::new();
        let port = mock
            .start()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Server should start: {}", e)))?;
        assert!(port > 0);
        assert_eq!(mock.port(), port);
        mock.stop().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_ai_metrics_default() {
        let mock = MockAiProvider::new();
        let metrics = mock.get_metrics();
        assert_eq!(metrics.active_models, 3);
        assert_eq!(metrics.total_requests, 1_500);
        assert!((metrics.avg_latency_ms - 250.0).abs() < 0.001);
        assert!((metrics.gpu_utilization_percent - 45.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_inference_with_different_models() {
        let mock = MockAiProvider::new();

        let llm_response = InferenceResponse {
            output: "LLM output".to_string(),
            confidence: 0.9,
            latency_ms: 150.0,
        };
        mock.set_response("llm_input", llm_response);

        let vision_request = InferenceRequest {
            model_type: ModelType::Vision,
            input: "vision_input".to_string(),
            parameters: HashMap::new(),
        };

        // Unknown input returns None
        let result = mock.infer(&vision_request);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_health_status_changes() {
        let mock = MockAiProvider::new();
        assert_eq!(mock.get_health(), HealthStatus::Healthy);

        mock.set_health(HealthStatus::Degraded);
        assert_eq!(mock.get_health(), HealthStatus::Degraded);

        mock.set_health(HealthStatus::Unhealthy);
        assert_eq!(mock.get_health(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_ai_provider_default_trait() {
        let mock = MockAiProvider::default();
        assert_eq!(mock.port(), 0);
        assert_eq!(mock.get_health(), HealthStatus::Healthy);
    }
}
