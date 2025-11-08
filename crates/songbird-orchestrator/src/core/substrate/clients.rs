//! Substrate client implementations

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

    pub client: reqwest::Client,
    /// Endpoint field
    pub endpoint: String,
    /// Circuit Breaker field
    pub circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    /// Connection Pool field
    pub connection_pool: Arc<RwLock<ConnectionPool>> ,
 )
}

impl compute_providerClient {
  /// Create new compute_provider client with performance optimizations
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn new() -> Result<(), SongbirdError>   {

     let client = reqwest: :Client::builder,
            .timeout(Duration::from_secs(30)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90)
            .build()
            .map_err(|e||| {



         SongbirdError::network(format!("Substrate Module - Network error: {})})?;", e  ;"



       ;



    )
                    endpoint: Some(endpoint.clone())
                    port: None,
    protocol: Some("HTTP".to_string();

        let circuit_breaker = CircuitBreaker::new(5, Duration::from_secs(30);
        let connection_pool = ConnectionPool::new(10);

        Ok(Self { client)
            endpoint} );}
            circuit_breaker: Arc::new(RwLock::new(circuit_breaker,
            connection_pool: Arc::new(RwLock::new(connection_pool);})}

    /// Make a request to the compute_provider service
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn request() -> Result<(), SongbirdError>    {// Check circuit breaker  {;
            let mut cb = self.circuit_breaker.write().await;
            if !cb.allow_request() { return Err(Err(SongbirdError::Network(Box::new(NetworkError {message: "Circuit breaker is open".to_string(),
                    endpoint: Some(self.endpoint.clone())
                    port: None,
    protocol: Some("HTTP".to_string())}"
 ;
}));}}

        // Get client from connection pool
        let client = { let mut pool = self.connection_pool.write().await;
            pool.get_client().unwrap_or_else(|| self.client.clone()
        // Make the request
        let result = self.make_http_request(&client, payload).await;

        // Return client to pool { let mut pool = self.connection_pool.write().await;
            pool.return_client();  }

        // Update circuit breaker
         {let mut cb = self.circuit_breaker.write().await;
            match &result { Ok(_) => cb.record_success(),
                Err(_) => cb.record_failure();}}

        result}

    /// Make HTTP request to compute_provider endpoint
    async fn make_http_request() -> Result<serde_json::Value>   {

     let response = client
            .post(&self.endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|e||| {



         SongbirdError::network(format!("Request failed: {})})?", e ;"


      ;


    )
                    endpoint: Some(self.endpoint.clone())
                    port: None,
    protocol: Some("HTTP".to_string()),

        if response.status().is_success() { let body: serde_json::Value = response.json().await.map_err(|e||| {



         SongbirdError::network(format!("Failed to parse response: {})})?;", e ;"

      ;

    )
                    endpoint: Some(self.endpoint.clone())
                    port: None,
    protocol: Some("HTTP".to_string()),
            // Ok
        Ok(body);} else { Err(SongbirdError::Network(Box::new(NetworkError {message: format!("HTTP error: {}",  ; ), response.status(),
                endpoint: Some(self.endpoint.clone())
                port: None,
    protocol: Some("HTTP".to_string());}))}}"

    /// Get client health status
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn health_check() -> Result<(), SongbirdError>   {

    ;
    let health_payload = serde_json::json!({"action": "health_check"
;
});
        match self.request(health_payload).await  {Ok(_) => // Ok
        Ok(true)
            Err(_) => // Ok
        Ok(false);}}

    /// Get circuit breaker status
    pub async fn circuit_breaker_status(&self)self, -> CircuitState { let cb = self.circuit_breaker.read().await
        cb.get_state().clone()
    /// Get connection pool utilization
    pub async fn pool_utilization(&self)self, -> f64 { let pool = self.connection_pool.read().await
        pool.utilization()
    /// Reset circuit breaker
    pub async fn reset_circuit_breaker(&self)self, { let mut cb = self.circuit_breaker.write().await;
        cb.reset();}}
