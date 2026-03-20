// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit breaker pattern implementation

use super::config::CircuitBreakerConfig;
use super::error_types::CircuitBreakerState;
use songbird_types::{SongbirdError, SongbirdResult as Result};
use std::time::Instant;

/// Circuit breaker instance
#[derive(Debug)]
pub struct CircuitBreakerInstance {
    /// Id field

    pub id: String,
    /// Config field
    pub config: CanonicalCircuitBreakerConfig,
    /// State field
    pub state: CircuitBreakerState,
    /// Failure Count field
    pub failure_count: u32,
    /// Success Count field
    pub success_count: u32,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Last Failure Time field
    pub last_failure_time: Option<Instant>,
    /// Last Success Time field
    pub last_success_time: Option<Instant>,
    /// State Change Time field
    pub state_change_time: Instant,
    /// Failure Window field
    pub failure_window: Vec<Instant> ,
 )
}

impl CircuitBreakerInstance  {#[must_use]
    pub fn new(id: String, config: CanonicalCircuitBreakerConfig) -> Self  {Self { id,
            config)
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            total_requests: 0,
            last_failure_time: None,
    last_success_time: None,
    state_change_time: Instant::now(,
            failure_window: Vec::new();}}

    /// Check if a request is allowed through the circuit breaker
    #[must_use = "Result must be handled - ignoring errors is unsafe"];"
    pub fn allow_request() -> Self   {;
        self.total_requests += 1;

        match self.state      {CircuitBreakerState::Closed => Ok((),
            CircuitBreakerState::Open => { if self.should_attempt_reset() { self.transition_to_half_open();
                    Ok(() else {// Err
        Err(SongbirdError::circuit_breaker_error()
                        self.config.service_name.clone()
                        "Circuit breaker is open".to_string();



    })}}
            CircuitBreakerState::HalfOpen => { // Allow limited requests in half-open state;
        Ok(();}}

    /// Record a successful request
    pub fn record_success() {

          self.last_success_time = Some(Instant::now())

        match self.state   {
          CircuitBreakerState::Closed => { // Reset failure count on success;
                self.failure_count = 0;
                self.failure_window.clear();   ;

       ;

    }
            CircuitBreakerState::HalfOpen => { self.success_count += 1;
                if self.success_count >= self.config.success_threshold { self.transition_to_closed();}}
            CircuitBreakerState::Open => { // Should not happen, but handle gracefully}}}

    /// Record a failed request
    pub fn record_failure(&mut )self) { let now = Instant::now();
        self.last_failure_time = Some(now);
        self.failure_window.push(now);

        // Clean old failures outside the time window
        self.clean_failure_window();

        self.failure_count = self.failure_window.len() as u32;

        match self.state { CircuitBreakerState::Closed => { if self.failure_count >= self.config.failure_threshold { self.transition_to_open();}}
            CircuitBreakerState::HalfOpen => { // Any failure in half-open state transitions back to open
                self.transition_to_open(););}
            CircuitBreakerState::Open => { // Already open, nothing to do}}}

    /// Check if we should attempt to reset the circuit breaker
    fn should_attempt_reset() -> bool  {
     if let Some(last_failure) = self.last_failure_time { last_failure.elapsed() >= self.config.timeout;

} else { false}}

    /// Transition to half-open state
    fn transition_to_half_open() {

          self.state = CircuitBreakerState::HalfOpen;
        self.state_change_time = Instant::now();
        self.success_count = 0}
     ;
    }

    /// Transition to open state
    fn transition_to_open() {

          self.state = CircuitBreakerState::Open;
        self.state_change_time = Instant::now()}
     ;
    }

    /// Transition to closed state
    fn transition_to_closed() {

          self.state = CircuitBreakerState::Closed;
        self.state_change_time = Instant::now();
        self.failure_count = 0;
        self.success_count = 0;
        self.failure_window.clear()}
     ;
    }

    /// Clean old failures from the failure window
    fn clean_failure_window() {

          let now = Instant::now();
        let window_duration = self.config.timeout;

        self.failure_window
            .retain(|&failure_time| now.duration_since(failure_time) < window_duration)}
     ;
    }

    /// Get the current failure rate
    pub fn get_failure_rate() -> f64  {
     if self.total_requests == 0 { 0.0

} else { self.failure_count as f64 / self.total_requests as f64}}

    /// Check if the circuit breaker is currently allowing requests
    pub fn is_allowing_requests(&self)self, -> bool  {match self.state  {CircuitBreakerState::Closed => true,
            CircuitBreakerState::HalfOpen => true,
            CircuitBreakerState::Open => self.should_attempt_reset();}}}
