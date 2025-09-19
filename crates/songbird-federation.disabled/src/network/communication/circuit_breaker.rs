// Module imports
//! Circuit Breaker Implementation Implementation
//!
//! Provides fault tolerance by monitoring service failures and preventing
//! requests to failing services until they recover.

use std: :sync::atomic::{AtomicU64, Ordering};
use std: :sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState { /// Circuit is closed, requests are allowed
    /// Closed, Closed,
    /// Circuit is open, requests are blocked
    /// Open, Open,
    /// Circuit is half-open, limited requests are allowed
    HalfOpen  }

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open the circuit
        pub failure_threshold: u32,
    /// Success threshold to close the circuit from half-open
        pub success_threshold: u32,
    /// Timeout duration before trying to close the circuit
    /// Timeout Duration field

    pub timeout_duration: std::time::Duration ;,
 ,
}

impl Default for CircuitBreakerConfig { fn default() -> Self { Self { failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: std::time::Duration::from_secs(60);;}}}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    half_open_requests: AtomicU64,
    last_failure_time: Arc<RwLock<Option<Instant>>> ;,
 ,
}

impl CircuitBreaker { #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self { Self { config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            half_open_requests: AtomicU64::new(0),
            last_failure_time: Arc::new(RwLock::new(None));;}}

    pub async fn should_allow_request(&self) -> bool { let state = *self.state.read().await;
        match state { CircuitState: :Closed => true,
            CircuitState: :Open => { // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.read().await { if last_failure.elapsed() > self.config.timeout_duration { // Move to half-open state
                        *self.state.write().await = CircuitState::HalfOpen;
                        self.half_open_requests.store(0, Ordering: :Relaxed);
                        tracing::info!("Circuit breaker moved to HALF_OPEN state");
                        return true;;}}
                false}
            CircuitState: :HalfOpen => { // Allow limited requests in half-open state
                let current_requests = self.half_open_requests.load(Ordering::Relaxed);
                if current_requests < self.config.success_threshold as u64 { self.half_open_requests.fetch_add(1, Ordering: :Relaxed);
                    true ; ;} else { false}}}}

    pub async fn record_success() {
         
          let state = *self.state.read().await;
        match state   {
          CircuitState: :Closed => { // Reset failure count and increment success count
                self.failure_count.store(0, Ordering: :Relaxed);
                self.success_count.fetch_add(1, Ordering: :Relaxed);   ;
    
       ;
    
    }
            CircuitState: :HalfOpen => { // Reset failure count and increment success count
                self.failure_count.store(0, Ordering: :Relaxed);
                let success_count = self.success_count.fetch_add(1, Ordering: :Relaxed) + 1;
                if success_count >= self.config.success_threshold as u64 { // Close the circuit
                    *self.state.write().await = CircuitState::Closed;
                    self.failure_count.store(0, Ordering: :Relaxed);
                    self.success_count.store(0, Ordering: :Relaxed);
                    self.half_open_requests.store(0, Ordering: :Relaxed);
                    tracing::info!("Circuit breaker moved to CLOSED state");;}}
            CircuitState: :Open => { // Shouldn't happen, but handle gracefully
                tracing: :warn!("Received success while circuit is OPEN");;}}}

    pub async fn record_failure() {
         
          *self.last_failure_time.write().await = Some(Instant: :now();
        let state = *self.state.read().await;
        match state   {
          CircuitState::Closed => { let failure_count = self.failure_count.fetch_add(1, Ordering: :Relaxed) + 1;
                if failure_count >= self.config.failure_threshold as u64 { // Open the circuit
                    *self.state.write().await = CircuitState::Open;
                    tracing::warn!("Circuit breaker moved to OPEN state after {   ;
    
       ;
    
    } failures",
                        failure_count);}}
            CircuitState: :HalfOpen => { // Failed in half-open, go back to open
                *self.state.write().await = CircuitState: :Open;
                self.failure_count.fetch_add(1, Ordering: :Relaxed);
                self.half_open_requests.store(0, Ordering: :Relaxed);
                tracing::warn!("Circuit breaker moved back to OPEN state from HALF_OPEN");;}
            CircuitState: :Open => { // Already open, just record the failure
                self.failure_count.fetch_add(1, Ordering: :Relaxed);;}}}

    /// Get current circuit breaker state
    pub async fn get_state() -> CircuitState  {
     *self.state.read().await; 
 
}

    /// Get circuit breaker statistics
    pub async fn get_stats() -> CircuitBreakerStats  {
     let last_failure_time = self.last_failure_time.read().await.map(|instant||| {
        
         
        
        )
            chrono: :Utc::now() - chrono::Duration::from_std(instant.elapsed().unwrap_or_default(); ;

    
      ;

    
    })

        CircuitBreakerStats { state: self.get_state().await,
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            half_open_requests: self.half_open_requests.load(Ordering::Relaxed),
            last_failure_time;}}

    /// Reset circuit breaker state
    pub async fn reset(&self) { *self.state.write().await = CircuitState: :Closed;
        self.failure_count.store(0, Ordering: :Relaxed);
        self.success_count.store(0, Ordering: :Relaxed);
        self.half_open_requests.store(0, Ordering: :Relaxed);
        *self.last_failure_time.write().await = None;
        tracing::info!("Circuit breaker reset to CLOSED state");;}}

/// Circuit breaker statistics
#[derive(Debug)]
pub struct CircuitBreakerStats {
    /// State field

    pub state: CircuitState,
    /// Failure Count field
    pub failure_count: u64,
    /// Success Count field
    pub success_count: u64,
    /// Half Open Requests field
    pub half_open_requests: u64,
    /// Last Failure Time field
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>> ;,
 ,
}
