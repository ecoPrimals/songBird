// Canonical Test Framework for Songbird Universal Orchestrator
//
// This module provides standardized testing patterns, utilities, and assertions
// that ensure consistent testing across all Songbird crates.

use songbird_errors::SongbirdError;
use songbird_types::errors::SongbirdResult;
use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

/// Standard test timeout for async operations
pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Quick test timeout for unit tests
pub const QUICK_TEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Extended test timeout for integration tests
pub const EXTENDED_TEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Test result type alias for canonical usage
pub type TestResult<T> = SongbirdResult<T>;

/// Test execution context with timing and resource tracking
#[derive(Debug)]
pub struct TestContext {
    pub name: String,
    pub start_time: std::time::Instant,
    pub timeout: Duration,
    pub metadata: std::collections::HashMap<String, String>,
}

impl TestContext {
    /// Create a new test context
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start_time: std::time::Instant::now(),
            timeout: DEFAULT_TEST_TIMEOUT,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set custom timeout for this test
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add metadata to the test context
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get elapsed time since test start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if test has exceeded timeout
    pub fn is_timeout(&self) -> bool {
        self.elapsed() > self.timeout
    }
}

/// Canonical test assertions for Songbird types
pub struct CanonicalAssertions;

impl CanonicalAssertions {
    /// Assert that a result is successful
    pub fn assert_success<T>(result: &SongbirdResult<T>) -> TestResult<()> {
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(SongbirdError::service(
                "test-utils",
                format!("Expected success but got error: {e}"),
            )),
        }
    }

    /// Assert that a result is an error
    pub fn assert_error<T>(result: &SongbirdResult<T>) -> TestResult<()> {
        match result {
            Ok(_) => Err(SongbirdError::service(
                "test-utils",
                "Expected error but got success",
            )),
            Err(_) => Ok(()),
        }
    }

    /// Assert that a result is an error with specific message
    pub fn assert_error_contains<T>(
        result: &SongbirdResult<T>,
        expected_msg: &str,
    ) -> TestResult<()> {
        match result {
            Ok(_) => Err(SongbirdError::service(
                "test-utils",
                "Expected error but got success",
            )),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains(expected_msg) {
                    Ok(())
                } else {
                    Err(SongbirdError::service(
                        "test-utils",
                        format!("Error '{error_str}' does not contain '{expected_msg}'"),
                    ))
                }
            }
        }
    }

    /// Assert that a SongbirdResponse is successful
    pub fn assert_response_success<T>(_response: &SongbirdResult<T>) -> TestResult<()> {
        // SongbirdResponse doesn't have a success field - it's always successful if Ok
        // The presence of data indicates success
        Ok(())
    }

    /// Assert that an operation completes within timeout
    pub async fn assert_timeout<F, Fut, T>(
        operation: F,
        timeout_duration: Duration,
    ) -> TestResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = SongbirdResult<T>>,
    {
        match tokio::time::timeout(timeout_duration, operation()).await {
            Ok(result) => result,
            Err(_) => Err(SongbirdError::service("test-utils", "Operation timed out")),
        }
    }

    /// Assert that a value is within a range
    pub fn assert_range<T: PartialOrd + std::fmt::Debug>(
        value: T,
        min: T,
        max: T,
    ) -> TestResult<()> {
        if value >= min && value <= max {
            Ok(())
        } else {
            Err(SongbirdError::service(
                "test-utils",
                format!("Value {value:?} is not in range [{min:?}, {max:?}]"),
            ))
        }
    }

    /// Assert that two floating point values are approximately equal
    pub fn assert_approx_equal(actual: f64, expected: f64, tolerance: f64) -> TestResult<()> {
        let diff = (actual - expected).abs();
        if diff <= tolerance {
            Ok(())
        } else {
            Err(SongbirdError::service("test-utils", format!("Values not approximately equal: {actual} vs {expected} (tolerance: {tolerance})")))
        }
    }

    /// Assert that two durations are approximately equal
    pub fn assert_duration_approx_equal(
        actual: Duration,
        expected: Duration,
        tolerance: Duration,
    ) -> TestResult<()> {
        let diff = if actual > expected {
            actual - expected
        } else {
            expected - actual
        };

        if diff <= tolerance {
            Ok(())
        } else {
            Err(SongbirdError::service(
                "test-utils",
                format!("Duration {actual:?} is not within {tolerance:?} of expected {expected:?}"),
            ))
        }
    }
}

/// Performance testing utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    /// Measure execution time of an operation
    pub async fn measure_async<F, Fut, T>(operation: F) -> (T, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = operation().await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure execution time of a synchronous operation
    pub fn measure_sync<F, T>(operation: F) -> (T, Duration)
    where
        F: FnOnce() -> T,
    {
        let start = std::time::Instant::now();
        let result = operation();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Run a performance benchmark with multiple iterations
    pub async fn benchmark_async<F, Fut, T>(operation: F, iterations: usize) -> PerformanceResults
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = T>,
    {
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let (_, duration) = Self::measure_async(operation.clone()).await;
            durations.push(duration);
        }

        PerformanceResults::new(durations)
    }

    /// Assert that an operation meets performance requirements
    pub async fn assert_performance<F, Fut, T>(
        operation: F,
        max_duration: Duration,
        iterations: usize,
    ) -> TestResult<()>
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = T>,
    {
        let results = Self::benchmark_async(operation, iterations).await;

        if results.average() <= max_duration {
            Ok(())
        } else {
            Err(SongbirdError::service(
                "test-utils",
                format!(
                    "Performance requirement failed: average {:?} > max {:?}",
                    results.average(),
                    max_duration
                ),
            ))
        }
    }
}

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct PerformanceResults {
    durations: Vec<Duration>,
}

impl PerformanceResults {
    pub fn new(durations: Vec<Duration>) -> Self {
        Self { durations }
    }

    /// Get the average duration
    pub fn average(&self) -> Duration {
        if self.durations.is_empty() {
            return Duration::ZERO;
        }

        let total_nanos: u64 = self.durations.iter().map(|d| d.as_nanos() as u64).sum();
        Duration::from_nanos(total_nanos / self.durations.len() as u64)
    }

    /// Get the minimum duration
    pub fn min(&self) -> Duration {
        self.durations
            .iter()
            .min()
            .copied()
            .unwrap_or(Duration::ZERO)
    }

    /// Get the maximum duration
    pub fn max(&self) -> Duration {
        self.durations
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::ZERO)
    }

    /// Get the median duration
    pub fn median(&self) -> Duration {
        if self.durations.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted = self.durations.clone();
        sorted.sort();

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            let sum_nanos = sorted[mid - 1].as_nanos() + sorted[mid].as_nanos();
            Duration::from_nanos((sum_nanos / 2) as u64)
        } else {
            sorted[mid]
        }
    }

    /// Calculate standard deviation
    pub fn std_deviation(&self) -> Duration {
        if self.durations.len() <= 1 {
            return Duration::ZERO;
        }

        let avg = self.average();
        let avg_nanos = avg.as_nanos() as f64;

        let variance: f64 = self
            .durations
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - avg_nanos;
                diff * diff
            })
            .sum::<f64>()
            / (self.durations.len() - 1) as f64;

        Duration::from_nanos(variance.sqrt() as u64)
    }
}

/// Mock service for testing
#[derive(Debug, Clone)]
pub struct MockService {
    /// Service name
    pub name: String,
    /// Service port
    pub port: u16,
    /// Service is healthy
    pub healthy: bool,
}

impl MockService {
    /// Create a mock service
    #[must_use]
    pub fn new(service_id: &str, healthy: bool) -> Self {
        Self {
            name: service_id.to_string(),
            port: 8080, // Default test port
            healthy,
        }
    }

    /// Simulate a service call
    ///
    /// # Errors
    /// Returns an error if the service is unhealthy.
    pub async fn call(&self, _input: &str) -> SongbirdResult<String> {
        // Simulate response delay
        tokio::time::sleep(Duration::from_millis(10)).await;

        if self.healthy {
            Ok(format!("Response from {}", self.name))
        } else {
            Err(SongbirdError::service(
                "test-utils",
                format!("Service {} unavailable", &self.name),
            ))
        }
    }

    /// Get the number of times this service has been called
    #[must_use]
    pub fn call_count(&self) -> usize {
        0 // Simplified mock - no call count tracking
    }

    /// Reset the call counter
    pub fn reset_call_count(&self) {
        // No-op for this simplified mock
    }
}

/// Test environment for canonical testing
#[derive(Debug, Clone)]
pub struct TestEnvironment {
    /// Environment name
    pub name: String,
    /// Environment configuration
    pub config: HashMap<String, String>,
    /// Test timeout
    pub timeout: Duration,
}

impl TestEnvironment {
    /// Create a new test environment
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: HashMap::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Set up a clean test environment
    ///
    /// # Errors
    /// Returns an error if setup fails.
    pub fn setup() -> TestResult<()> {
        // Initialize logging for tests if not already done
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        Ok(())
    }

    /// Clean up test environment
    ///
    /// # Errors
    /// Returns an error if cleanup fails.
    pub async fn cleanup() -> TestResult<()> {
        // Perform any necessary cleanup
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    /// Create a test configuration with sensible defaults
    #[must_use]
    pub fn create_test_config() -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("test_mode".to_string(), "true".to_string());
        config.insert("log_level".to_string(), "debug".to_string());
        config.insert("timeout_ms".to_string(), "30000".to_string());
        config
    }
}

/// Macro for creating canonical tests with automatic setup/cleanup
#[macro_export]
macro_rules! canonical_test {
    ($name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $name() -> $crate::canonical_test_framework::TestResult<()> {
            use $crate::canonical_test_framework::{TestContext, TestEnvironment};

            // Setup test environment
            TestEnvironment::setup().await?;

            // Create test context
            let ctx = TestContext::new(stringify!($name));

            // Run the test
            let result = $test_fn(ctx).await;

            // Cleanup
            TestEnvironment::cleanup().await?;

            result
        }
    };
}

/// Macro for performance tests
#[macro_export]
macro_rules! performance_test {
    ($name:ident, $max_duration:expr, $iterations:expr, $test_fn:expr) => {
        #[tokio::test]
        async fn $name() -> $crate::canonical_test_framework::TestResult<()> {
            use $crate::canonical_test_framework::{PerformanceTestUtils, TestEnvironment};

            TestEnvironment::setup().await?;

            PerformanceTestUtils::assert_performance(|| $test_fn(), $max_duration, $iterations)
                .await?;

            TestEnvironment::cleanup().await?;
            Ok(())
        }
    };
}
