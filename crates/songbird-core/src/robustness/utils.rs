//! Utility functions for robustness patterns

use super::config::RobustnessConfig;
use super::error_types::RetryableError;
use songbird_errors::SongbirdError;
use std::time::Duration;

/// Create a default robustness configuration
pub fn create_default_config() -> RobustnessConfig {
    RobustnessConfig::default()
}

/// Calculate backoff delay for retries with jitter
pub fn calculate_backoff_delay(
    attempt: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    backoff_multiplier: f64,
    enable_jitter: bool,
    jitter_percentage: f64,
) -> Duration {
    let delay_ms = (base_delay_ms as f64 * backoff_multiplier.powi(attempt as i32))
        .min(max_delay_ms as f64);
    
    let final_delay_ms = if enable_jitter {
        add_jitter(delay_ms, jitter_percentage)
    } else {
        delay_ms
    };
    
    Duration::from_millis(final_delay_ms as u64)
}

/// Add jitter to a delay to prevent thundering herd
fn add_jitter(delay_ms: f64, jitter_percentage: f64) -> f64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let jitter_range = delay_ms * jitter_percentage;
    let jitter: f64 = rng.gen_range(-jitter_range..=jitter_range);
    (delay_ms + jitter).max(0.0)
}

/// Check if an error is retryable based on configuration
pub fn is_retryable_error(_error: &SongbirdError, retryable_errors: &[RetryableError]) -> bool {
    // This is a simplified implementation
    // In a real implementation, you would match the error against retryable types
    !retryable_errors.is_empty() // Placeholder logic
}

/// Calculate percentile from a sorted list of values
pub fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }
    
    let index = (percentile / 100.0 * (sorted_values.len() - 1) as f64) as usize;
    Some(sorted_values[index.min(sorted_values.len() - 1)])
}

/// Calculate moving average of response times
pub fn calculate_moving_average(values: &[Duration], window_size: usize) -> Option<Duration> {
    if values.is_empty() {
        return None;
    }
    
    let start_index = values.len().saturating_sub(window_size);
    let recent_values = &values[start_index..];
    
    let sum: Duration = recent_values.iter().sum();
    let avg_nanos = sum.as_nanos() / recent_values.len() as u128;
    
    Some(Duration::from_nanos(avg_nanos as u64))
}

/// Check if a duration exceeds a threshold by a percentage
pub fn exceeds_threshold(value: Duration, threshold: Duration, percentage: f64) -> bool {
    let threshold_with_buffer = threshold.as_nanos() as f64 * (1.0 + percentage / 100.0);
    value.as_nanos() as f64 > threshold_with_buffer
} 