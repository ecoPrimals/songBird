/// Performance testing utilities
///
/// Provides benchmarking, performance measurement, and load testing
/// utilities for performance-critical components.
use songbird_types::errors::SongbirdResult;
use songbird_types::SongbirdError;
use std::time::{Duration, Instant};

/// Performance measurement context
pub struct PerformanceMeasurement {
    pub operation_name: String,
    pub start_time: Instant,
    pub measurements: Vec<Duration>,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
}

impl PerformanceMeasurement {
    #[must_use]
    pub fn new(operation_name: &str) -> Self {
        Self {
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            measurements: Vec::new(),
            min_duration: None,
            max_duration: None,
        }
    }

    /// Record a single measurement
    pub fn record(&mut self, duration: Duration) {
        self.measurements.push(duration);

        self.min_duration = Some(self.min_duration.map_or(duration, |min| min.min(duration)));

        self.max_duration = Some(self.max_duration.map_or(duration, |max| max.max(duration)));
    }

    /// Get average duration
    #[must_use]
    pub fn average_duration(&self) -> Option<Duration> {
        if self.measurements.is_empty() {
            return None;
        }

        #[allow(clippy::cast_possible_truncation)]
        let total_nanos: u64 = self.measurements.iter().map(|d| d.as_nanos() as u64).sum();

        #[allow(clippy::cast_possible_truncation)]
        let len = self.measurements.len() as u64;
        Some(Duration::from_nanos(total_nanos / len))
    }

    /// Get percentile duration
    #[must_use]
    pub fn percentile_duration(&self, percentile: f32) -> Option<Duration> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut sorted = self.measurements.clone();
        sorted.sort();

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = ((percentile / 100.0) * sorted.len() as f32) as usize;
        let index = index.min(sorted.len() - 1);

        Some(sorted[index])
    }

    /// Print performance summary
    pub fn print_summary(&self) {
        println!("Performance Summary for {}:", self.operation_name);
        println!("  Measurements: {}", self.measurements.len());

        if let Some(avg) = self.average_duration() {
            println!("  Average: {avg:?}");
        }

        if let Some(min) = self.min_duration {
            println!("  Min: {min:?}");
        }

        if let Some(max) = self.max_duration {
            println!("  Max: {max:?}");
        }

        if let Some(p95) = self.percentile_duration(95.0) {
            println!("  95th percentile: {p95:?}");
        }

        if let Some(p99) = self.percentile_duration(99.0) {
            println!("  99th percentile: {p99:?}");
        }
    }
}

/// Benchmark a function multiple times
///
/// # Errors
///
/// Returns an error if any iteration of the operation fails.
pub async fn benchmark_async<F, Fut, T>(
    name: &str,
    iterations: usize,
    mut operation: F,
) -> SongbirdResult<PerformanceMeasurement>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = SongbirdResult<T>>,
{
    let mut measurement = PerformanceMeasurement::new(name);

    for i in 0..iterations {
        let start = Instant::now();

        operation().await.map_err(|e| {
            SongbirdError::service("benchmark", format!("Benchmark iteration {i} failed: {e}"))
        })?;

        let duration = start.elapsed();
        measurement.record(duration);
    }

    Ok(measurement)
}

/// Benchmark a synchronous function multiple times
///
/// # Errors
///
/// Returns an error if any iteration of the operation fails.
pub fn benchmark_sync<F, T>(
    name: &str,
    iterations: usize,
    mut operation: F,
) -> SongbirdResult<PerformanceMeasurement>
where
    F: FnMut() -> SongbirdResult<T>,
{
    let mut measurement = PerformanceMeasurement::new(name);

    for i in 0..iterations {
        let start = Instant::now();

        operation().map_err(|e| {
            SongbirdError::service("benchmark", format!("Benchmark iteration {i} failed: {e}"))
        })?;

        let duration = start.elapsed();
        measurement.record(duration);
    }

    Ok(measurement)
}

/// Load testing utilities
pub struct LoadTester {
    pub concurrent_users: usize,
    pub test_duration: Duration,
    pub ramp_up_duration: Duration,
}

impl LoadTester {
    #[must_use]
    pub fn new(concurrent_users: usize, test_duration: Duration) -> Self {
        Self {
            concurrent_users,
            test_duration,
            ramp_up_duration: Duration::from_secs(10),
        }
    }

    #[must_use]
    pub fn with_ramp_up(mut self, ramp_up_duration: Duration) -> Self {
        self.ramp_up_duration = ramp_up_duration;
        self
    }

    /// Run a load test with concurrent operations
    ///
    /// # Errors
    ///
    /// Returns an error if the load test setup or execution fails.
    pub async fn run_load_test<F, Fut, T>(
        &self,
        name: &str,
        operation: F,
    ) -> SongbirdResult<LoadTestResults>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = SongbirdResult<T>> + Send + 'static,
        T: Send + 'static,
    {
        let mut results = LoadTestResults::new(name);
        let start_time = Instant::now();

        // Calculate ramp-up delay between users
        let ramp_up_delay = if self.concurrent_users > 1 {
            #[allow(clippy::cast_possible_truncation)]
            let users_minus_one = (self.concurrent_users as u32).saturating_sub(1);
            self.ramp_up_duration / users_minus_one
        } else {
            Duration::ZERO
        };

        let mut handles = Vec::new();

        // Spawn concurrent operations with ramp-up
        for i in 0..self.concurrent_users {
            let operation = operation.clone();
            let test_duration = self.test_duration;

            let handle = tokio::spawn(async move {
                let mut user_results = Vec::new();
                let user_start = Instant::now();

                while user_start.elapsed() < test_duration {
                    let op_start = Instant::now();
                    match operation().await {
                        Ok(_) => {
                            user_results.push(LoadTestSample {
                                duration: op_start.elapsed(),
                                success: true,
                                error: None,
                            });
                        }
                        Err(e) => {
                            user_results.push(LoadTestSample {
                                duration: op_start.elapsed(),
                                success: false,
                                error: Some(e.to_string()),
                            });
                        }
                    }

                    // Small delay to prevent overwhelming the system
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }

                user_results
            });

            handles.push(handle);

            // Ramp-up delay
            if i < self.concurrent_users - 1 {
                // Add delay between user spawns for realistic load patterns
            }
            tokio::time::sleep(ramp_up_delay).await;
        }

        // Collect results from all users
        for handle in handles {
            let user_samples = handle.await.map_err(|e| {
                SongbirdError::service("load-test ", format!("Load test task failed: {e}"))
            })?;

            results.samples.extend(user_samples);
        }

        results.total_duration = start_time.elapsed();
        results.calculate_metrics();

        Ok(results)
    }
}

/// Load test results
#[derive(Debug)]
pub struct LoadTestResults {
    pub test_name: String,
    pub samples: Vec<LoadTestSample>,
    pub total_duration: Duration,
    pub success_rate: f32,
    pub average_response_time: Duration,
    pub throughput_per_second: f32,
}

#[derive(Debug)]
pub struct LoadTestSample {
    pub duration: Duration,
    pub success: bool,
    pub error: Option<String>,
}

impl LoadTestResults {
    #[must_use]
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            samples: Vec::new(),
            total_duration: Duration::ZERO,
            success_rate: 0.0,
            average_response_time: Duration::ZERO,
            throughput_per_second: 0.0,
        }
    }

    fn calculate_metrics(&mut self) {
        if self.samples.is_empty() {
            return;
        }

        let successful_samples: Vec<_> = self.samples.iter().filter(|s| s.success).collect();

        #[allow(clippy::cast_precision_loss)]
        {
            self.success_rate = successful_samples.len() as f32 / self.samples.len() as f32;
        }

        if !successful_samples.is_empty() {
            #[allow(clippy::cast_possible_truncation)]
            let total_nanos: u64 =
                successful_samples.iter().map(|s| s.duration.as_nanos() as u64).sum();

            #[allow(clippy::cast_possible_truncation)]
            let len = successful_samples.len() as u64;
            self.average_response_time = Duration::from_nanos(total_nanos / len);
        }

        if self.total_duration.as_secs_f32() > 0.0 {
            #[allow(clippy::cast_precision_loss)]
            let throughput = successful_samples.len() as f32 / self.total_duration.as_secs_f32();
            self.throughput_per_second = throughput;
        }
    }

    pub fn print_summary(&self) {
        println!("Load Test Results for {}:", self.test_name);
        println!("  Total samples: {}", self.samples.len());
        println!("  Success rate: {:.2}%", self.success_rate * 100.0);
        println!("  Average response time: {:?}", self.average_response_time);
        println!("  Throughput: {:.2} ops/sec ", self.throughput_per_second);
        println!("  Test duration: {:?}", self.total_duration);
    }
}
