use serde_json::Value;
use songbird_errors::SongbirdResult;
/// Network mocking utilities
///
/// Provides mock network services, simulated network conditions,
/// and network testing utilities for comprehensive network testing.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Network mock manager for testing
#[derive(Debug)]
pub struct NetworkMockManager {
    /// Mock responses
    responses: Arc<RwLock<HashMap<String, MockResponse>>>,
    /// Request count tracking
    request_counts: Arc<RwLock<HashMap<String, u32>>>,
}

/// Mock response configuration
#[derive(Debug, Clone)]
pub struct MockResponse {
    /// Whether the response should succeed
    pub success: bool,
    /// Response data
    pub data: Value,
    /// Artificial delay
    pub delay_ms: Option<u64>,
}

impl NetworkMockManager {
    /// Create a new network mock manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            responses: Arc::new(RwLock::new(HashMap::new())),
            request_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set mock response for an endpoint
    ///
    /// # Errors
    /// Returns an error if the mock response cannot be configured.
    pub async fn set_mock_response(
        &self,
        endpoint: &str,
        response: MockResponse,
    ) -> SongbirdResult<()> {
        let mut responses = self.responses.write().await;
        responses.insert(endpoint.to_string(), response);
        Ok(())
    }

    /// Get mock response for an endpoint
    ///
    /// # Errors
    /// Returns an error if the mock response cannot be retrieved.
    pub async fn get_mock_response(&self, endpoint: &str) -> SongbirdResult<MockResponse> {
        // Increment request count
        {
            let mut counts = self.request_counts.write().await;
            *counts.entry(endpoint.to_string()).or_insert(0) += 1;
        }

        let responses = self.responses.read().await;

        if let Some(response) = responses.get(endpoint) {
            // Apply artificial delay if configured
            if let Some(delay_ms) = response.delay_ms {
                sleep(Duration::from_millis(delay_ms)).await;
            }
            Ok(songbird_errors::evolved_success(response.clone()))
        } else {
            Ok(songbird_errors::evolved_success(MockResponse {
                success: false,
                data: serde_json::json!({"error": "No mock response configured"}),
                delay_ms: None,
            }))
        }
    }

    /// Get request count for an endpoint
    ///
    /// # Errors
    /// Returns an error if the request count cannot be retrieved.
    pub async fn get_request_count(&self, endpoint: &str) -> SongbirdResult<u32> {
        let counts = self.request_counts.read().await;
        Ok(*counts.get(endpoint).unwrap_or(&0))
    }

    /// Reset all mock data
    ///
    /// # Errors
    /// Returns an error if the reset operation fails.
    pub async fn reset(&self) -> SongbirdResult<()> {
        let mut responses = self.responses.write().await;
        let mut counts = self.request_counts.write().await;

        responses.clear();
        counts.clear();
        Ok(())
    }

    /// Clear request counts
    ///
    /// # Errors
    /// Returns an error if the clear operation fails.
    pub async fn clear_counts(&self) -> SongbirdResult<()> {
        let mut counts = self.request_counts.write().await;
        counts.clear();
        Ok(())
    }
}

impl Default for NetworkMockManager {
    fn default() -> Self {
        Self::new()
    }
}
