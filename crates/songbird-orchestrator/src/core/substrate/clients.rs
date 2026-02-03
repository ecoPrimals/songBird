//! Substrate client implementations

use songbird_http_client::IpcHttpClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use songbird_types::{NetworkError, Result, SongbirdError};

use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::connection_pool::ConnectionPool;

/// compute_provider client for compute and container operations with connection pooling
#[derive(Debug, Clone)]
pub struct compute_providerClient {
    /// Client field
    pub client: Arc<IpcHttpClient>,
    /// Endpoint field
    pub endpoint: String,
    /// Circuit Breaker field
    pub circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    /// Connection Pool field
    pub connection_pool: Arc<RwLock<ConnectionPool>>,
}

impl compute_providerClient {
    // DEAD CODE: Corrupted reqwest implementation removed during ecoBin v2.0 migration
    // This section had malformed syntax from incomplete previous edits
    // TODO: If needed, implement using IpcHttpClient via Unix sockets
    /*
    /// Create new compute_provider client with performance optimizations
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
    pub async fn new() -> Result<Self, SongbirdError> {
        // Removed corrupted reqwest code
        // See ecoPrimals/sessions/feb-2026/reqwest-removal/ for migration docs
        unimplemented!("compute_providerClient requires IpcHttpClient migration")
    }
    */
    
    // Placeholder implementation to satisfy module structure
    #[must_use]
    pub async fn new(endpoint: String) -> Result<Self, SongbirdError> {
        let circuit_breaker = CircuitBreaker::new(5, Duration::from_secs(30));
        let connection_pool = ConnectionPool::new(10).await?;

        Ok(Self {
            client: Arc::new(IpcHttpClient::new().await?),
            endpoint,
            circuit_breaker: Arc::new(RwLock::new(circuit_breaker)),
            connection_pool: Arc::new(RwLock::new(connection_pool)),
        })
    }

    /// Make a request to the compute_provider service
    ///
    /// # Errors
    ///
    /// Returns error if request fails or circuit breaker is open
    pub async fn request(&self, payload: serde_json::Value) -> Result<serde_json::Value, SongbirdError> {
        // Check circuit breaker
        {
            let mut cb = self.circuit_breaker.write().await;
            if !cb.allow_request() {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    message: "Circuit breaker is open".to_string(),
                    endpoint: Some(self.endpoint.clone()),
                    port: None,
                    protocol: Some("HTTP".to_string()),
                })));
            }
        }

        // Get client from connection pool or use default
        let pool_client = {
            let mut pool = self.connection_pool.write().await;
            pool.get_client()
        };

        // Make the request (use pooled or default client)
        let result = self.make_http_request(pool_client.as_deref(), payload).await;

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

    /// Make HTTP request to compute_provider endpoint
    async fn make_http_request(
        &self,
        pool_client: Option<&IpcHttpClient>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, SongbirdError> {
        // Use pooled client if available, otherwise use default
        let client = pool_client.unwrap_or(self.client.as_ref());

        let response = client
            .post(&self.endpoint)
            .await
            .json(&payload)
            .map_err(|e| {
                SongbirdError::Network(Box::new(NetworkError {
                    message: format!("Failed to prepare request: {}", e),
                    endpoint: Some(self.endpoint.clone()),
                    port: None,
                    protocol: Some("HTTP".to_string()),
                }))
            })?
            .send()
            .await
            .map_err(|e| {
                SongbirdError::Network(Box::new(NetworkError {
                    message: format!("Request failed: {}", e),
                    endpoint: Some(self.endpoint.clone()),
                    port: None,
                    protocol: Some("HTTP".to_string()),
                }))
            })?;

        if response.is_success() {
            let body: serde_json::Value = response.json().await.map_err(|e| {
                SongbirdError::Network(Box::new(NetworkError {
                    message: format!("Failed to parse response: {}", e),
                    endpoint: Some(self.endpoint.clone()),
                    port: None,
                    protocol: Some("HTTP".to_string()),
                }))
            })?;
            Ok(body)
        } else {
            Err(SongbirdError::Network(Box::new(NetworkError {
                message: format!("HTTP error: {}", response.status()),
                endpoint: Some(self.endpoint.clone()),
                port: None,
                protocol: Some("HTTP".to_string()),
            })))
        }
    }"

    /// Get client health status
    ///
    /// # Errors
    ///
    /// Returns Ok(false) if health check fails, Ok(true) if successful
    pub async fn health_check(&self) -> Result<bool, SongbirdError> {
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
