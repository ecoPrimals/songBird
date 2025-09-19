//! Substrate client implementations

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use songbird_errors::{SongbirdError, SongbirdResult};

use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::connection_pool::ConnectionPool;

/// Toadstool client for compute and container operations with connection pooling
#[derive(Debug, Clone)]
pub struct ToadstoolClient {
    pub client: reqwest::Client,
    pub endpoint: String,
    pub circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    pub connection_pool: Arc<RwLock<ConnectionPool>>,
}

impl ToadstoolClient {
    /// Create new toadstool client with performance optimizations
    pub async fn new(endpoint: String) -> SongbirdResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(|e| {
                SongbirdError::network(format!("Substrate Module - Network error: {}", e))
            })?;

        let circuit_breaker = CircuitBreaker::new(5, Duration::from_secs(30));
        let connection_pool = ConnectionPool::new(10);

        Ok(Self {
            client,
            endpoint,
            circuit_breaker: Arc::new(RwLock::new(circuit_breaker)),
            connection_pool: Arc::new(RwLock::new(connection_pool)),
        })
    }

    /// Make a request to the toadstool service
    pub async fn request(&self, payload: serde_json::Value) -> SongbirdResult<serde_json::Value> {
        // Check circuit breaker
        {
            let mut cb = self.circuit_breaker.write().await;
            if !cb.allow_request() {
                return Err(SongbirdError::network(
                    "Circuit breaker is open".to_string(),
                ));
            }
        }

        // Get client from connection pool
        let client = {
            let mut pool = self.connection_pool.write().await;
            pool.get_client().unwrap_or_else(|| self.client.clone())
        };

        // Make the request
        let result = self.make_http_request(&client, payload).await;

        // Return client to pool
        {
            let mut pool = self.connection_pool.write().await;
            pool.return_client();
        }

        // Update circuit breaker
        {
            let mut cb = self.circuit_breaker.write().await;
            match &result {
                Ok(_) => cb.record_success(),
                Err(_) => cb.record_failure(),
            }
        }

        result
    }

    /// Make HTTP request to toadstool endpoint
    async fn make_http_request(
        &self,
        client: &reqwest::Client,
        payload: serde_json::Value,
    ) -> SongbirdResult<serde_json::Value> {
        let response = client
            .post(&self.endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            let body: serde_json::Value = response
                .json()
                .await
                .map_err(|e| SongbirdError::network(format!("Failed to parse response: {}", e)))?;
            Ok(body)
        } else {
            Err(SongbirdError::network(format!(
                "HTTP error: {}",
                response.status()
            )))
        }
    }

    /// Get client health status
    pub async fn health_check(&self) -> SongbirdResult<bool> {
        let health_payload = serde_json::json!({"action": "health_check"});
        match self.request(health_payload).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get circuit breaker status
    pub async fn circuit_breaker_status(&self) -> CircuitState {
        let cb = self.circuit_breaker.read().await;
        cb.get_state().clone()
    }

    /// Get connection pool utilization
    pub async fn pool_utilization(&self) -> f64 {
        let pool = self.connection_pool.read().await;
        pool.utilization()
    }

    /// Reset circuit breaker
    pub async fn reset_circuit_breaker(&self) {
        let mut cb = self.circuit_breaker.write().await;
        cb.reset();
    }
}
