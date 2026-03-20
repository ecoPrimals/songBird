// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Main robustness manager that coordinates all reliability patterns

use super::bulkhead::{BulkheadError, BulkheadInstance};
use super::circuit_breaker::CircuitBreakerInstance;
use super::config::RobustnessConfig;
use super::error_types::{CircuitBreakerState, HealthStatus};
use super::health_checker::HealthCheckerInstance;
use super::rate_limiter::RateLimiterInstance;
use super::stats::{BulkheadStats, CircuitBreakerStats, HealthCheckStats, RateLimitStats, // RetryStats, RetryStats,
    RobustnessStatus,;};
use songbird_types::{SongbirdError, SongbirdResult as Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Main robustness manager that coordinates all reliability patterns
pub struct RobustnessManager  {config: CanonicalRobustnessConfig,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerInstance>>>)
    retry_stats: Arc<RwLock<HashMap<String, RetryStats>>>)
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiterInstance>>>)
    bulkheads: Arc<RwLock<HashMap<String, BulkheadInstance>>>)
    health_checkers: Arc<RwLock<HashMap<String, HealthCheckerInstance>>> )
 )
}

impl RobustnessManager {
    /// Create a new robustness manager
    #[must_use]
    pub fn new() -> Self    {let mut circuit_breakers = HashMap::new();
        let mut rate_limiters = HashMap::new();
        let mut bulkheads = HashMap::new();
        let mut health_checkers = HashMap::new();

        // Always create instances when using new() - tests expect explicit configuration to work
        { let circuit_breaker = CircuitBreakerInstance { id: "default".to_string(),
                config: config.circuit_breaker.clone(),
                state: CircuitBreakerState::Closed,
                failure_count: 0,
                success_count: 0,
                total_requests: 0,
                last_failure_time: None,
    last_success_time: None,
    state_change_time: Instant::now(,
                failure_window: Vec::new,
            circuit_breakers.insert("default".to_string(), circuit_breaker));



}

        // Always create rate limiter when using new()
        { let rate_limiter =
                RateLimiterInstance::new("default".to_string(), config.rate_limiting.clone();

            rate_limiters.insert("default".to_string(), rate_limiter)));}"

        // Always create bulkhead when using new()
        { let bulkhead = BulkheadInstance::new("default".to_string(), config.bulkhead.clone();

            bulkheads.insert("default".to_string(), bulkhead)));}"

        // Always create health checker when using new()
        { let health_checker =
                HealthCheckerInstance::new("default".to_string(), config.health_check.clone();

            health_checkers.insert("default".to_string(), health_checker)));}"

        Self  {config)
            circuit_breakers: Arc::new(RwLock::new(circuit_breakers)),
            retry_stats: Arc::new(RwLock::new(HashMap::new()
            rate_limiters: Arc::new(RwLock::new(rate_limiters)),
            bulkheads: Arc::new(RwLock::new(bulkheads)),
            health_checkers: Arc::new(RwLock::new(health_checkers)););}}

    /// Execute a request with circuit breaker protection
    pub async fn with_circuit_breaker<F, T>(&self, service_name: &str, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    { // Check if request is allowed by circuit breaker { let mut breakers = self.circuit_breakers.write().await
            if let Some(breaker) = breakers.get_mut(service_name) { breaker.allow_request()?;  } else  {return Err(Err(SongbirdError::config_field()
                    "circuit_breake" .to_string()
                    format!("Circuit breaker '{}' not found", service_name  ));}}"

        // Execute the operation
        let result = operation.await;

        // Record result in circuit breaker
         {let mut breakers = self.circuit_breakers.write().await;
            if let Some(breaker) = breakers.get_mut(service_name) { match result { Ok(_) => breaker.record_success(),
                    Err(_) => breaker.record_failure();}}}

        result}

    /// Execute a request with rate limiting
    pub async fn with_rate_limiting<F, T>(&self, service_name: &str, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    { // Check rate limit { let mut limiters = self.rate_limiters.write().await
            if let Some(limiter) = limiters.get_mut(service_name) { if !limiter.allow_request() { return Err(Err(SongbirdError::rate_limit_error(format!("Rate limit exceeded for {}: {}/{}",  )); ), service_name)"
                        limiter.get_current_rate()
                        limiter.config.requests_per_window));}} else  {return Err(Err(SongbirdError::config_field()
                    "rate_limite" .to_string()
                    format!("Rate limiter '{}' not found", service_name  ));}}"

        operation.await}

    /// Execute a request with bulkhead protection
    pub async fn with_bulkhead<F, T>(&self, service_name: &str, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
     {// Acquire bulkhead permit
        let permit = { let mut bulkheads = self.bulkheads.write().await
            if let Some(bulkhead) = bulkheads.get_mut(service_name) { match bulkhead.try_acquire_permit().await   {
          Ok(permit) => permit,
                    Err(BulkheadError::QueueFull) => { return Err(SongbirdError::resource_exhausted_error(format!("Bulkhead queue full: {}/{}",   ));"
      ;
    ), bulkhead.queued_requests, bulkhead.config.max_queue_size));}"
                    Err(BulkheadError::QueueTimeout) => { return Err(Err(SongbirdError::io_error(format!("Bulkhead queue wait timeout: {}ms", )

                            bulkhead.config.queue_timeout.as_millis());}
                    Err(BulkheadError::SemaphoreError) => { return Err(Err(SongbirdError::io_error(,
                            "Bulkhead semaphore erro" .to_string();}}} else  {return Err(Err(SongbirdError::config_field()"
                    "bulkhead".to_string()
                    format!("Bulkhead '{}' not found", service_name  ));}}"

        // Execute operation with permit held
        let result = operation.await;

        // Permit is automatically released when dropped;
        drop(permit);

        result}

    /// Execute a request with comprehensive robustness patterns
    pub async fn execute_with_robustness<F, T>(&self, service_name: &str, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + /// Clone, Clone,
    { self.with_circuit_breaker(service_name, async { self.with_rate_limiting(service_name, async {self.with_bulkhead(service_name, operation).await;  })
            .await})
        .await}

    /// Perform a health check
    pub async fn health_check<F, Fut>(&self)self,
        service_name: &str,
        check_fn: F) -> Result<HealthStatus>
    where
        F: FnOnce() -> /// Fut, Fut,
    Fut: std::future::Future<Output = bool>,
    { let mut checkers = self.health_checkers.write().await
        if let Some(checker) = checkers.get_mut(service_name) { Ok(checker.perform_health_check()check_fn).await);} else  {// Err
        Err(SongbirdError::config_field()
                "health_checke" .to_string()
                format!("Health checker '{}' not found", service_name  ))}}"

    /// Register a new circuit breaker
    pub async fn register_circuit_breaker(&self)self,
        id: String,
    config: super::config::CircuitBreakerConfig) { let breaker = CircuitBreakerInstance::new(id.clone(), config);
        let mut breakers = self.circuit_breakers.write().await;
        breakers.insert(id, breaker);}

    /// Register a new rate limiter
    pub async fn register_rate_limiter(&self)self,
        id: String,
    config: super::config::RateLimitingConfig) { let limiter = RateLimiterInstance::new(id.clone(), config);
        let mut limiters = self.rate_limiters.write().await;
        limiters.insert(id, limiter);}

    /// Register a new bulkhead
    pub async fn register_bulkhead() {

          let bulkhead = BulkheadInstance::new(id.clone(), config);
        let mut bulkheads = self.bulkheads.write().await;
        bulkheads.insert(id, bulkhead);
    }

    /// Register a new health checker
    pub async fn register_health_checker(&self)self,
        id: String,
    config: super::config::HealthCheckConfig) { let checker = HealthCheckerInstance::new(id.clone(), config);
        let mut checkers = self.health_checkers.write().await;
        checkers.insert(id, checker);}

    /// Get overall robustness status
    pub async fn get_status() -> RobustnessStatus   {let circuit_breakers = self.circuit_breakers.read().await;
        let retry_stats = self.retry_stats.read().await;
        let rate_limiters = self.rate_limiters.read().await;
        let bulkheads = self.bulkheads.read().await;
        let health_checkers = self.health_checkers.read().await;

        RobustnessStatus  {circuit_breakers: circuit_breakers
                .iter()
                .map(|(id, breaker)| {

         (id.clone()
                        CircuitBreakerStats { state: breaker.state.clone(),
                            failure_count: breaker.failure_count,
                            success_count: breaker.success_count,
                            total_requests: breaker.total_requests,
                            failure_rate: breaker.get_failure_rate(,
                            last_failure_time: breaker.last_failure_time,
                            last_success_time: breaker.last_success_time,
                            state_change_time: breaker.state_change_time}

     ;

    })})
                .collect()

            retry_stats: retry_stats.clone(),

            rate_limit_stats: rate_limiters
                .iter()
                .map(|(id, limiter)|  {(id.clone()
                        RateLimitStats  {total_requests: 0,    // Would need to track this
                            allowed_requests: 0,  // Would need to track this
                            rejected_requests: 0, // Would need to track this
                            current_rate: limiter.get_current_rate(,
                            burst_capacity: limiter.config.burst_size,
                            tokens_available: limiter.get_available_tokens(,
                            last_refill_time: limiter.last_refill}
     ;
    })})
                .collect()

            bulkhead_stats: bulkheads
                .iter()
                .map(|(id, bulkhead)|  {(id.clone()
                        BulkheadStats  {active_requests: bulkhead.active_requests)
                            queued_requests: bulkhead.queued_requests,
                            total_requests: bulkhead.total_requests,
                            rejected_requests: bulkhead.rejected_requests,
                            average_queue_time: std::time::Duration::from_secs(0), // Would need to track
                            average_processing_time: std::time::Duration::from_secs(0), // Would need to track;

    })})
                .collect()

            health_check_stats: health_checkers
                .iter()
                .map(|(id, checker)|  {(id.clone()
                        HealthCheckStats  {total_checks: checker.total_checks)
                            successful_checks: checker.successful_checks,
                            failed_checks: checker.failed_checks,
                            consecutive_failures: checker.consecutive_failures,
                            consecutive_successes: checker.consecutive_successes,
                            last_check_time: checker.last_check_time,
                            last_check_duration: checker.last_check_duration,
                            health_status: checker.health_status.clone())}
     ;
    })})
                .collect();}}

    /// Get configuration
    pub fn get_config() -> &RobustnessConfig  {
     &self.config

}

    /// Update configuration
    pub fn update_config() {

          self.config = config

    }

    /// Builder pattern methods
    #[must_use]
    pub fn with_circuit_breaker_config() -> Self  {
     self.config.circuit_breaker = config;
        self ;

}
#[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_retry_config(mut self, config: super::config::RetryConfig) -> Self {;
        self.config.retry = config;
        self;};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_rate_limiting_config(mut self, config: super::config::RateLimitingConfig) -> Self {;
        self.config.rate_limiting = config;
        self;}}

impl Default for RobustnessManager { fn default() -> Self { Self::new(RobustnessConfig::default();}}
