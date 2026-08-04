# Error Recovery & Resilience Specification

**Status**: 🔴 Not Implemented  
**Priority**: Critical (Week 3)  
**Owner**: Songbird Core

---

## Overview

Songbird must gracefully handle failures and recover from errors, including network issues, tower crashes, and task failures.

---

## Requirements

### Functional Requirements

1. **Automatic Retry**
   - Configurable retry policies (max attempts, backoff strategy)
   - Exponential backoff with jitter
   - Retry only on transient errors
   - Track retry attempts

2. **Circuit Breakers**
   - Detect failing services
   - Open circuit after threshold failures
   - Half-open state for recovery testing
   - Automatic circuit reset

3. **Graceful Degradation**
   - Fall back to alternative towers
   - Fall back to alternative protocols
   - Reduced functionality if necessary
   - User notification of degradation

4. **Partial Success Handling**
   - Track completed vs failed sub-tasks
   - Return partial results
   - Offer retry for failed portions

### Non-Functional Requirements

- Retry decision latency < 10ms
- Circuit breaker overhead < 1ms
- Automatic recovery without human intervention
- Preserve task state during failures

---

## API Design

```rust
/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum retry attempts
    pub max_attempts: u32,
    
    /// Backoff strategy
    pub backoff: BackoffStrategy,
    
    /// Which errors to retry
    pub retry_on: Vec<ErrorType>,
    
    /// Maximum total retry time
    pub max_retry_duration: Duration,
}

/// Backoff strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed { delay: Duration },
    
    /// Exponential backoff (2^n * base_delay)
    Exponential { base_delay: Duration, max_delay: Duration },
    
    /// Exponential with jitter (random ±25%)
    ExponentialWithJitter { base_delay: Duration, max_delay: Duration },
}

/// Circuit breaker state
#[derive(Debug, Clone)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    
    /// Circuit is open, requests fail immediately
    Open { opened_at: DateTime<Utc>, failures: u32 },
    
    /// Circuit is half-open, testing recovery
    HalfOpen { test_started_at: DateTime<Utc> },
}

/// Circuit breaker
pub struct CircuitBreaker {
    service_name: String,
    state: Arc<RwLock<CircuitState>>,
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failures before opening circuit
    pub failure_threshold: u32,
    
    /// Time to wait before attempting recovery
    pub timeout_duration: Duration,
    
    /// Success threshold in half-open state
    pub half_open_successes: u32,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match self.state() {
            CircuitState::Open { .. } => {
                Err(Error::CircuitOpen)
            }
            CircuitState::Closed | CircuitState::HalfOpen { .. } => {
                match f.await {
                    Ok(result) => {
                        self.record_success().await;
                        Ok(result)
                    }
                    Err(e) => {
                        self.record_failure().await;
                        Err(e)
                    }
                }
            }
        }
    }
}

/// Error recovery manager
pub trait ErrorRecoveryManager: Send + Sync {
    /// Execute with retry
    async fn execute_with_retry<F, T>(
        &self,
        operation: F,
        policy: RetryPolicy,
    ) -> Result<T>
    where
        F: Fn() -> Future<Output = Result<T>>;
    
    /// Get circuit breaker for service
    fn get_circuit_breaker(&self, service: &str) -> Arc<CircuitBreaker>;
    
    /// Handle partial failure
    async fn handle_partial_failure(
        &self,
        task_id: TaskId,
        succeeded: Vec<SubTaskId>,
        failed: Vec<(SubTaskId, Error)>,
    ) -> Result<PartialResult>;
}
```

---

## Retry Decision Flow

```
┌─────────────────┐
│ Task Fails      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐      No      ┌─────────────────┐
│ Is Retryable?   ├──────────────►│ Mark as Failed  │
└────────┬────────┘               └─────────────────┘
         │ Yes
         ▼
┌─────────────────┐      Yes     ┌─────────────────┐
│ Max Attempts?   ├──────────────►│ Mark as Failed  │
└────────┬────────┘               └─────────────────┘
         │ No
         ▼
┌─────────────────┐
│ Calculate Delay │
│ (with backoff)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Wait for Delay  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Retry Task      │
└─────────────────┘
```

---

## Circuit Breaker State Machine

```
       ┌──────────┐
       │  Closed  │
       └────┬─────┘
            │
            │ Failure threshold reached
            ▼
       ┌──────────┐
       │   Open   │
       └────┬─────┘
            │
            │ Timeout expired
            ▼
       ┌──────────┐
       │ HalfOpen │
       └────┬─────┘
            │
            ├──► Success → Closed
            │
            └──► Failure → Open
```

---

## Implementation Plan

### Phase 1: Retry Logic (Day 1-2)
- [ ] Implement retry policies
- [ ] Exponential backoff with jitter
- [ ] Error classification (transient vs permanent)
- [ ] Retry attempt tracking

### Phase 2: Circuit Breakers (Day 2-3)
- [ ] Circuit breaker state machine
- [ ] Failure detection
- [ ] Timeout and recovery logic
- [ ] Per-service circuit breakers

### Phase 3: Graceful Degradation (Day 3-4)
- [ ] Fallback tower selection
- [ ] Fallback protocol selection
- [ ] Degraded mode detection
- [ ] User notifications

### Phase 4: Partial Success (Day 4-5)
- [ ] Sub-task tracking
- [ ] Partial result aggregation
- [ ] Selective retry
- [ ] Result merging

---

## Error Classification

```rust
pub enum ErrorType {
    /// Transient errors (retry)
    NetworkTimeout,
    NetworkConnectionRefused,
    ServiceUnavailable,
    RateLimited,
    
    /// Permanent errors (don't retry)
    InvalidInput,
    PermissionDenied,
    NotFound,
    QuotaExceeded,
}
```

---

## Success Criteria

- [ ] Transient errors automatically retried
- [ ] Circuit breakers prevent cascading failures
- [ ] Fallback to alternative towers works
- [ ] Partial results returned when possible
- [ ] Zero data loss during failures

---

## Testing Requirements

- Retry test (fail 3 times, succeed on 4th)
- Circuit breaker test (open after threshold)
- Chaos test (kill tower mid-task, recover)
- Network partition test
- Partial failure test (5/10 sub-tasks fail)

---

## Dependencies

- Task lifecycle (for state restoration)
- Resource management (for fallback tower selection)

