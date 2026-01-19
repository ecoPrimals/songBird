//! Mock HTTP API server for testing HTTP gateway
//!
//! Provides a lightweight mock implementation of external HTTP APIs
//! (OpenAI, HuggingFace, etc.) for testing the HTTP gateway without
//! requiring real API keys or making actual HTTP calls.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

/// Mock HTTP API server
pub struct MockHttpApi {
    port: u16,
    responses: Arc<RwLock<HashMap<String, Value>>>,
    rate_limit: Arc<RwLock<Option<(usize, std::time::Duration)>>>,
    request_count: Arc<RwLock<usize>>,
}

impl MockHttpApi {
    /// Create a new mock HTTP API
    pub fn new(port: u16) -> Self {
        Self {
            port,
            responses: Arc::new(RwLock::new(HashMap::new())),
            rate_limit: Arc::new(RwLock::new(None)),
            request_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Register a mock response for a specific path
    pub async fn register_response(&self, path: &str, response: Value) {
        let mut responses = self.responses.write().await;
        responses.insert(path.to_string(), response);
    }

    /// Set rate limit (requests, per duration)
    pub async fn set_rate_limit(&self, requests: usize, per: std::time::Duration) {
        let mut rate_limit = self.rate_limit.write().await;
        *rate_limit = Some((requests, per));
    }

    /// Start the mock HTTP server
    pub async fn start(self: Arc<Self>) -> Result<()> {
        let responses = self.responses.clone();
        let rate_limit = self.rate_limit.clone();
        let request_count = self.request_count.clone();

        let route = warp::post().and(warp::path::full()).and(warp::body::json()).and_then(
            move |path: warp::path::FullPath, body: Value| {
                let responses = responses.clone();
                let rate_limit = rate_limit.clone();
                let request_count = request_count.clone();

                async move {
                    // Check rate limit
                    {
                        let mut count = request_count.write().await;
                        *count += 1;

                        if let Some((limit, _)) = *rate_limit.read().await {
                            if *count > limit {
                                return Ok::<_, warp::Rejection>(warp::reply::with_status(
                                    warp::reply::json(&json!({"error": "Rate limited"})),
                                    warp::http::StatusCode::TOO_MANY_REQUESTS,
                                ));
                            }
                        }
                    }

                    // Find response
                    let responses = responses.read().await;
                    let path_str = path.as_str();

                    if let Some(response) = responses.get(path_str) {
                        Ok(warp::reply::with_status(
                            warp::reply::json(response),
                            warp::http::StatusCode::OK,
                        ))
                    } else {
                        Ok(warp::reply::with_status(
                            warp::reply::json(&json!({"error": "Not found"})),
                            warp::http::StatusCode::NOT_FOUND,
                        ))
                    }
                }
            },
        );

        warp::serve(route).run(([127, 0, 0, 1], self.port)).await;
        Ok(())
    }

    /// Get request count
    pub async fn get_request_count(&self) -> usize {
        *self.request_count.read().await
    }

    /// Reset request count
    pub async fn reset_request_count(&self) {
        let mut count = self.request_count.write().await;
        *count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_http_api_creation() {
        let mock = MockHttpApi::new(8888);
        assert_eq!(mock.port, 8888);
    }

    #[tokio::test]
    async fn test_mock_http_api_register_response() {
        let mock = MockHttpApi::new(8889);
        let response = json!({"result": "success"});
        mock.register_response("/test", response.clone()).await;

        let responses = mock.responses.read().await;
        assert_eq!(responses.get("/test"), Some(&response));
    }
}
