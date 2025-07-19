//! Configuration validator tests

use songbird_errors::validation::ConfigValidator;

#[test]
fn test_config_validator_timeout() {
    // Valid timeout
    let result = ConfigValidator::validate_timeout(5000, "connection_timeout");
    assert!(result.is_ok());

    // Timeout too short
    let result = ConfigValidator::validate_timeout(100, "short_timeout");
    assert!(result.is_err());

    // Timeout too long
    let result = ConfigValidator::validate_timeout(300000, "long_timeout");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_port() {
    // Valid port
    let result = ConfigValidator::validate_port(8080, "http_port");
    assert!(result.is_ok());

    // Valid high port
    let result = ConfigValidator::validate_port(65535, "max_port");
    assert!(result.is_ok());

    // Invalid port (0)
    let result = ConfigValidator::validate_port(0, "zero_port");
    assert!(result.is_err());

    // Invalid port (out of range)
    let result = ConfigValidator::validate_port(70000, "invalid_port");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_url() {
    // Valid HTTP URL
    let result = ConfigValidator::validate_url("http://example.com", "api_url");
    assert!(result.is_ok());

    // Valid HTTPS URL
    let result = ConfigValidator::validate_url("https://secure.example.com", "secure_url");
    assert!(result.is_ok());

    // Invalid URL format
    let result = ConfigValidator::validate_url("not-a-url", "invalid_url");
    assert!(result.is_err());

    // Empty URL
    let result = ConfigValidator::validate_url("", "empty_url");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_retry_config() {
    // Valid retry config
    let result = ConfigValidator::validate_retry_config(3, 1000);
    assert!(result.is_ok());

    // Too many retries
    let result = ConfigValidator::validate_retry_config(15, 1000);
    assert!(result.is_err());

    // Invalid retry delay
    let result = ConfigValidator::validate_retry_config(3, 50000);
    assert!(result.is_err());
}

#[test]
fn test_config_validator_thread_pool_size() {
    // Valid thread pool size
    let result = ConfigValidator::validate_thread_pool_size(4, "worker_threads");
    assert!(result.is_ok());

    // Invalid thread pool size (0)
    let result = ConfigValidator::validate_thread_pool_size(0, "zero_threads");
    assert!(result.is_err());

    // Very large thread pool (should warn but succeed)
    let result = ConfigValidator::validate_thread_pool_size(100, "many_threads");
    assert!(result.is_ok());
}

#[test]
fn test_config_validator_buffer_size() {
    // Valid buffer size
    let result = ConfigValidator::validate_buffer_size(8192, "buffer");
    assert!(result.is_ok());

    // Buffer too small
    let result = ConfigValidator::validate_buffer_size(512, "small_buffer");
    assert!(result.is_err());

    // Buffer too large
    let result = ConfigValidator::validate_buffer_size(131072, "large_buffer");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_memory_limit() {
    // Valid memory limit
    let result = ConfigValidator::validate_memory_limit(512, "memory_limit");
    assert!(result.is_ok());

    // Memory limit too small
    let result = ConfigValidator::validate_memory_limit(32, "low_memory_limit");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_percentage() {
    // Valid percentage
    let result = ConfigValidator::validate_percentage(75.5, "cpu_usage");
    assert!(result.is_ok());

    // Valid edge cases
    let result = ConfigValidator::validate_percentage(0.0, "min_percent");
    assert!(result.is_ok());

    let result = ConfigValidator::validate_percentage(100.0, "max_percent");
    assert!(result.is_ok());

    // Invalid percentage (negative)
    let result = ConfigValidator::validate_percentage(-10.0, "negative_percent");
    assert!(result.is_err());

    // Invalid percentage (over 100)
    let result = ConfigValidator::validate_percentage(150.0, "over_hundred");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_rate_limit() {
    // Valid rate limit
    let result = ConfigValidator::validate_rate_limit(100.0, "requests_per_second");
    assert!(result.is_ok());

    // Invalid rate limit (negative)
    let result = ConfigValidator::validate_rate_limit(-5.0, "negative_rate");
    assert!(result.is_err());

    // Invalid rate limit (zero)
    let result = ConfigValidator::validate_rate_limit(0.0, "zero_rate");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_connection_pool() {
    // Valid connection pool size
    let result = ConfigValidator::validate_connection_pool_size(10, "db_pool");
    assert!(result.is_ok());

    // Invalid connection pool size (0)
    let result = ConfigValidator::validate_connection_pool_size(0, "empty_pool");
    assert!(result.is_err());

    // Very large connection pool
    let result = ConfigValidator::validate_connection_pool_size(1000, "huge_pool");
    assert!(result.is_ok()); // Should warn but be valid
}

#[test]
fn test_config_validator_disk_space() {
    // Valid disk space requirement
    let result = ConfigValidator::validate_disk_space(1024, "storage_requirement");
    assert!(result.is_ok());

    // Invalid disk space (0)
    let result = ConfigValidator::validate_disk_space(0, "no_storage");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_network_interface() {
    // Valid network interfaces
    let result = ConfigValidator::validate_network_interface("eth0", "primary_interface");
    assert!(result.is_ok());

    let result = ConfigValidator::validate_network_interface("lo", "loopback_interface");
    assert!(result.is_ok());

    // Invalid network interface
    let result = ConfigValidator::validate_network_interface("", "empty_interface");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_log_level() {
    // Valid log levels
    let result = ConfigValidator::validate_log_level("info", "application_log_level");
    assert!(result.is_ok());

    let result = ConfigValidator::validate_log_level("debug", "debug_log_level");
    assert!(result.is_ok());

    let result = ConfigValidator::validate_log_level("error", "error_log_level");
    assert!(result.is_ok());

    // Invalid log level
    let result = ConfigValidator::validate_log_level("invalid", "bad_log_level");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_encryption_algorithm() {
    // Valid encryption algorithms
    let result = ConfigValidator::validate_encryption_algorithm("AES-256", "data_encryption");
    assert!(result.is_ok());

    let result = ConfigValidator::validate_encryption_algorithm("ChaCha20", "stream_encryption");
    assert!(result.is_ok());

    // Invalid encryption algorithm
    let result = ConfigValidator::validate_encryption_algorithm("MD5", "weak_encryption");
    assert!(result.is_err());
} 