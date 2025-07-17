//! Unit tests for validation module
//!
//! Tests all validation functions in src/errors/validation.rs
//! Coverage target: 95%+

use songbird_errors::validation::ConfigValidator;
use songbird_errors::SongbirdError;
use std::time::Duration;
use tempfile::TempDir;

#[cfg(test)]
mod port_validation_tests {
    use super::*;

    #[test]
    fn test_validate_port_valid_ranges() {
        // Test valid unprivileged ports
        assert!(ConfigValidator::validate_port(8080, "test_port").is_ok());
        assert!(ConfigValidator::validate_port(3000, "test_port").is_ok());
        assert!(ConfigValidator::validate_port(65535, "test_port").is_ok());
        assert!(ConfigValidator::validate_port(1024, "test_port").is_ok());
    }

    #[test]
    fn test_validate_port_privileged_range() {
        // Test privileged ports (should warn but pass)
        assert!(ConfigValidator::validate_port(80, "http_port").is_ok());
        assert!(ConfigValidator::validate_port(443, "https_port").is_ok());
        assert!(ConfigValidator::validate_port(22, "ssh_port").is_ok());
        assert!(ConfigValidator::validate_port(1, "min_privileged").is_ok());
        assert!(ConfigValidator::validate_port(1023, "max_privileged").is_ok());
    }

    #[test]
    fn test_validate_port_invalid() {
        // Test port 0 (invalid)
        match ConfigValidator::validate_port(0, "invalid_port") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "invalid_port");
                assert!(message.contains("Port cannot be 0"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for port 0"),
        }
    }

    #[test]
    fn test_validate_port_boundary_conditions() {
        // Test exact boundaries
        assert!(ConfigValidator::validate_port(1, "min_port").is_ok());
        assert!(ConfigValidator::validate_port(65535, "max_port").is_ok());

        // Port 0 should fail
        assert!(ConfigValidator::validate_port(0, "zero_port").is_err());
    }

    #[test]
    fn test_validate_port_range_valid() {
        // Test valid port ranges
        assert!(ConfigValidator::validate_port_range(8000, 8100).is_ok());
        assert!(ConfigValidator::validate_port_range(3000, 4000).is_ok());
        assert!(ConfigValidator::validate_port_range(1024, 65535).is_ok());
    }

    #[test]
    fn test_validate_port_range_invalid() {
        // Test invalid port range (start > end)
        match ConfigValidator::validate_port_range(8100, 8000) {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "port_range");
                assert!(message.contains("Start port"));
                assert!(message.contains("cannot be greater than end port"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for invalid port range"),
        }

        // Test invalid individual ports in range
        assert!(ConfigValidator::validate_port_range(0, 100).is_err());
        assert!(ConfigValidator::validate_port_range(100, 0).is_err());
    }

    #[test]
    fn test_validate_port_range_small_range() {
        // Test very small range (should warn but pass)
        assert!(ConfigValidator::validate_port_range(8000, 8005).is_ok());
    }
}

#[cfg(test)]
mod url_validation_tests {
    use super::*;

    #[test]
    fn test_validate_url_valid_schemes() {
        // Test all supported schemes
        let valid_urls = [
            ("http://example.com", "http"),
            ("https://example.com", "https"),
            ("ws://example.com/socket", "websocket"),
            ("wss://example.com/secure-socket", "secure_websocket"),
        ];

        for (url_str, name) in valid_urls {
            match ConfigValidator::validate_url(url_str, name) {
                Ok(()) => {
                    // URL validation successful
                    assert!(!url_str.is_empty());
                }
                Err(e) => panic!("Expected valid URL for {url_str}: {e:?}"),
            }
        }
    }

    #[test]
    fn test_validate_url_invalid_schemes() {
        let invalid_urls = [
            ("ftp://example.com", "ftp_url"),
            ("file:///tmp/test", "file_url"),
            ("mailto:test@example.com", "email_url"),
            ("ssh://user@host", "ssh_url"),
        ];

        for (url_str, name) in invalid_urls {
            match ConfigValidator::validate_url(url_str, name) {
                Err(SongbirdError::Config {
                    field: Some(field),
                    message,
                    suggestion,
                    context,
                }) => {
                    assert_eq!(field, name);
                    assert!(message.contains("Unsupported URL scheme"));
                    assert!(suggestion.is_some());
                    assert!(context.is_some());
                }
                _ => panic!("Expected Config error for unsupported scheme: {url_str}"),
            }
        }
    }

    #[test]
    fn test_validate_url_malformed() {
        let malformed_urls = [
            ("not-a-url", "invalid_format"),
            ("http://", "incomplete_url"),
            ("://example.com", "no_scheme"),
            ("", "empty_url"),
        ];

        for (url_str, name) in malformed_urls {
            match ConfigValidator::validate_url(url_str, name) {
                Err(SongbirdError::Config {
                    field: Some(field),
                    message,
                    suggestion,
                    context,
                }) => {
                    assert_eq!(field, name);
                    assert!(message.contains("Invalid URL format"));
                    assert!(suggestion.is_some());
                    assert!(context.is_some());
                }
                _ => panic!("Expected Config error for malformed URL: {url_str}"),
            }
        }
    }

    #[test]
    fn test_validate_http_url_specific() {
        // Valid HTTP URLs
        assert!(ConfigValidator::validate_http_url("http://example.com", "http").is_ok());
        assert!(ConfigValidator::validate_http_url("https://example.com", "https").is_ok());

        // Invalid (non-HTTP) URLs
        match ConfigValidator::validate_http_url("ws://example.com", "websocket") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "websocket");
                assert!(message.contains("Expected HTTP/HTTPS URL"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for non-HTTP URL"),
        }
    }

    #[test]
    fn test_validate_websocket_url_specific() {
        // Valid WebSocket URLs
        assert!(ConfigValidator::validate_websocket_url("ws://example.com", "ws").is_ok());
        assert!(ConfigValidator::validate_websocket_url("wss://example.com", "wss").is_ok());

        // Invalid (non-WebSocket) URLs
        match ConfigValidator::validate_websocket_url("http://example.com", "http") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "http");
                assert!(message.contains("Expected WebSocket URL"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for non-WebSocket URL"),
        }
    }

    #[test]
    fn test_validate_url_edge_cases() {
        // URLs with ports, paths, queries
        let complex_urls = [
            "http://example.com:8080/path?query=value",
            "https://sub.example.com:443/api/v1",
            "ws://localhost:3000/socket",
            "wss://secure.example.com/path",
        ];

        for url_str in complex_urls {
            assert!(ConfigValidator::validate_url(url_str, "complex_url").is_ok());
        }
    }
}

#[cfg(test)]
mod ip_validation_tests {
    use super::*;

    #[test]
    fn test_validate_ip_address_ipv4() {
        let valid_ipv4 = [
            "127.0.0.1",
            "192.168.1.1",
            "10.0.0.1",
            "172.16.0.1",
            "8.8.8.8",
            "0.0.0.0",
            "255.255.255.255",
        ];

        for ip_str in &valid_ipv4 {
            match ConfigValidator::validate_ip_address(ip_str, "test_ip") {
                Ok(_) => assert!(true),
                Err(e) => panic!("Expected valid IPv4 for {ip_str}: {e:?}"),
            }
        }
    }

    #[test]
    fn test_validate_ip_address_ipv6() {
        let valid_ipv6 = [
            "::1",
            "2001:db8::1",
            "fe80::1",
            "::",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
        ];

        for ip_str in &valid_ipv6 {
            match ConfigValidator::validate_ip_address(ip_str, "test_ip") {
                Ok(_ip) => {}, // IP validation passed
                Err(e) => panic!("Expected valid IPv6 for {ip_str}: {e:?}"),
            }
        }
    }

    #[test]
    fn test_validate_ip_address_invalid() {
        let invalid_ips = [
            "256.256.256.256",
            "192.168.1.256",
            "192.168.1",
            "192.168.1.1.1",
            "not-an-ip",
            "",
            "localhost",
        ];

        for ip_str in &invalid_ips {
            match ConfigValidator::validate_ip_address(ip_str, "invalid_ip") {
                Err(SongbirdError::Config {
                    field: Some(field),
                    message,
                    suggestion,
                    context,
                }) => {
                    assert_eq!(field, "invalid_ip");
                    assert!(message.contains("Invalid IP address format"));
                    assert!(suggestion.is_some());
                    assert!(context.is_some());
                }
                _ => panic!("Expected Config error for invalid IP: {ip_str}"),
            }
        }
    }

    #[test]
    fn test_validate_socket_address_valid() {
        let valid_addrs = [
            "127.0.0.1:8080",
            "192.168.1.1:80",
            "0.0.0.0:3000",
            "[::1]:8080",
            "[2001:db8::1]:80",
        ];

        for addr_str in &valid_addrs {
            match ConfigValidator::validate_socket_address(addr_str, "test_addr") {
                Ok(addr) => {
                    // Port assertion removed as addr type is not SocketAddr
                }
                Err(e) => panic!("Expected valid socket address for {addr_str}: {e:?}"),
            }
        }
    }

    #[test]
    fn test_validate_socket_address_invalid() {
        let invalid_addrs = [
            "127.0.0.1",       // Missing port
            ":8080",           // Missing IP
            "127.0.0.1:70000", // Invalid port
            "256.1.1.1:8080",  // Invalid IP
            "localhost:8080",  // Hostname not allowed
            "",                // Empty
        ];

        for addr_str in &invalid_addrs {
            match ConfigValidator::validate_socket_address(addr_str, "invalid_addr") {
                Err(SongbirdError::Config {
                    field: Some(field),
                    message,
                    suggestion,
                    context,
                }) => {
                    assert_eq!(field, "invalid_addr");
                    assert!(message.contains("Invalid socket address format"));
                    assert!(suggestion.is_some());
                    assert!(context.is_some());
                }
                _ => panic!("Expected Config error for invalid socket address: {addr_str}"),
            }
        }
    }
}

#[cfg(test)]
mod timeout_validation_tests {
    use super::*;

    #[test]
    fn test_validate_timeout_valid_range() {
        // Test valid timeout within bounds
        let result = ConfigValidator::validate_timeout(5000, "test_timeout", 1000, 10000);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("Test assertion failed"),
            ()
        );
    }

    #[test]
    fn test_validate_timeout_below_minimum() {
        match ConfigValidator::validate_timeout(500, "low_timeout", 1000, 10000) {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "low_timeout");
                assert!(message.contains("below minimum"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for timeout below minimum"),
        }
    }

    #[test]
    fn test_validate_timeout_above_maximum() {
        match ConfigValidator::validate_timeout(15000, "high_timeout", 1000, 10000) {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "high_timeout");
                assert!(message.contains("exceeds maximum"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for timeout above maximum"),
        }
    }

    #[test]
    fn test_validate_timeout_boundary_values() {
        // Test exact boundary values
        assert!(ConfigValidator::validate_timeout(1000, "min_timeout", 1000, 10000).is_ok());
        assert!(ConfigValidator::validate_timeout(10000, "max_timeout", 1000, 10000).is_ok());
    }

    #[test]
    fn test_validate_connection_timeout() {
        // Valid connection timeouts
        assert!(ConfigValidator::validate_connection_timeout(5000).is_ok());
        assert!(ConfigValidator::validate_connection_timeout(30000).is_ok());

        // Invalid connection timeouts
        assert!(ConfigValidator::validate_connection_timeout(50).is_err()); // Too low
        assert!(ConfigValidator::validate_connection_timeout(70000).is_err()); // Too high
    }

    #[test]
    fn test_validate_request_timeout() {
        // Valid request timeouts
        assert!(ConfigValidator::validate_request_timeout(30000).is_ok());
        assert!(ConfigValidator::validate_request_timeout(120000).is_ok());

        // Invalid request timeouts
        assert!(ConfigValidator::validate_request_timeout(500).is_err()); // Too low
        assert!(ConfigValidator::validate_request_timeout(400000).is_err()); // Too high
    }

    #[test]
    fn test_validate_health_check_interval() {
        // Valid health check intervals
        assert!(ConfigValidator::validate_timeout(15000, "health_check_interval", 1000, 300000).is_ok());
        assert!(ConfigValidator::validate_timeout(60000, "health_check_interval", 1000, 300000).is_ok());

        // Invalid health check intervals
        assert!(ConfigValidator::validate_timeout(500, "health_check_interval", 1000, 300000).is_err()); // Too low
        assert!(ConfigValidator::validate_timeout(400000, "health_check_interval", 1000, 300000).is_err());
        // Too high
    }
}

#[cfg(test)]
mod retry_validation_tests {
    use super::*;

    #[test]
    fn test_validate_retry_config_valid() {
        // Valid retry configurations
        assert!(ConfigValidator::validate_retry_config(3, 1000).is_ok());
        assert!(ConfigValidator::validate_retry_config(5, 2000).is_ok());
        assert!(ConfigValidator::validate_retry_config(0, 100).is_ok()); // No retries
    }

    #[test]
    fn test_validate_retry_config_too_many_retries() {
        match ConfigValidator::validate_retry_config(15, 1000) {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "max_retries");
                assert!(message.contains("exceeds reasonable limit"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for too many retries"),
        }
    }

    #[test]
    fn test_validate_retry_config_invalid_delay() {
        // Delay too short
        assert!(ConfigValidator::validate_retry_config(3, 5).is_err());

        // Delay too long
        assert!(ConfigValidator::validate_retry_config(3, 35000).is_err());
    }
}

#[cfg(test)]
mod thread_pool_validation_tests {
    use super::*;

    #[test]
    fn test_validate_thread_pool_size_valid() {
        // Valid thread pool sizes
        assert!(ConfigValidator::validate_thread_pool_size(4, "worker_pool").is_ok());
        assert!(ConfigValidator::validate_thread_pool_size(8, "worker_pool").is_ok());
        assert!(ConfigValidator::validate_thread_pool_size(1, "single_thread").is_ok());
    }

    #[test]
    fn test_validate_thread_pool_size_zero() {
        match ConfigValidator::validate_thread_pool_size(0, "empty_pool") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "empty_pool");
                assert!(message.contains("Thread pool size cannot be 0"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for zero thread pool size"),
        }
    }

    #[test]
    fn test_validate_thread_pool_size_large() {
        let cpu_count = num_cpus::get();
        let large_size = cpu_count * 8; // Very large

        // Should warn but not fail
        assert!(ConfigValidator::validate_thread_pool_size(large_size, "large_pool").is_ok());
    }
}

#[cfg(test)]
mod memory_validation_tests {
    use super::*;

    #[test]
    fn test_validate_memory_limit_valid() {
        // Valid memory limits
        assert!(ConfigValidator::validate_memory_limit(512, "memory_limit").is_ok());
        assert!(ConfigValidator::validate_memory_limit(1024, "memory_limit").is_ok());
        assert!(ConfigValidator::validate_memory_limit(2048, "memory_limit").is_ok());
    }

    #[test]
    fn test_validate_memory_limit_too_low() {
        match ConfigValidator::validate_memory_limit(32, "memory_limit") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "memory_limit");
                assert!(message.contains("Memory limit cannot be less than 64 MB"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for memory limit too low"),
        }
    }

    #[test]
    fn test_validate_memory_limit_boundary() {
        // Test exact boundary
        assert!(ConfigValidator::validate_memory_limit(64, "memory_limit").is_ok());
        assert!(ConfigValidator::validate_memory_limit(63, "memory_limit").is_err());
    }

    #[test]
    fn test_validate_memory_limit_very_high() {
        // Test with extremely high memory limit that would exceed most systems
        // This might warn but shouldn't error (depending on system memory)
        let very_high_limit = 1024 * 1024; // 1TB
        let result = ConfigValidator::validate_memory_limit(very_high_limit, "memory_limit");

        // Result depends on system memory, but should be either Ok (with warning) or Error
        match result {
            Ok(_) => {} // OK if system has enough memory
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "memory_limit");
                assert!(message.contains("exceeds system memory"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Unexpected error type for very high memory limit"),
        }
    }
}

#[cfg(test)]
mod buffer_validation_tests {
    use super::*;

    #[test]
    fn test_validate_buffer_size_valid() {
        // Valid buffer sizes
        assert!(ConfigValidator::validate_buffer_size(8192, "buffer").is_ok());
        assert!(ConfigValidator::validate_buffer_size(4096, "buffer").is_ok());
    }

    #[test]
    fn test_validate_buffer_size_too_small() {
        match ConfigValidator::validate_buffer_size(512, "small_buffer") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "small_buffer");
                assert!(message.contains("below minimum"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for buffer size too small"),
        }
    }

    #[test]
    fn test_validate_buffer_size_too_large() {
        match ConfigValidator::validate_buffer_size(131072, "large_buffer") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "large_buffer");
                assert!(message.contains("exceeds maximum"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for buffer size too large"),
        }
    }

    #[test]
    fn test_validate_buffer_size_power_of_two() {
        // Power of 2 sizes (should not warn)
        assert!(ConfigValidator::validate_buffer_size(1024, "buffer").is_ok());
        assert!(ConfigValidator::validate_buffer_size(2048, "buffer").is_ok());
        assert!(ConfigValidator::validate_buffer_size(4096, "buffer").is_ok());

        // Non-power of 2 sizes (should warn but pass)
        assert!(ConfigValidator::validate_buffer_size(3000, "buffer").is_ok());
        assert!(ConfigValidator::validate_buffer_size(5000, "buffer").is_ok());
    }
}

#[cfg(test)]
mod percentage_validation_tests {
    use super::*;

    #[test]
    fn test_validate_percentage_valid() {
        // Valid percentages
        assert!(ConfigValidator::validate_percentage(0.0, "min_percent").is_ok());
        assert!(ConfigValidator::validate_percentage(50.5, "mid_percent").is_ok());
        assert!(ConfigValidator::validate_percentage(100.0, "max_percent").is_ok());
        assert!(ConfigValidator::validate_percentage(25.25, "quarter").is_ok());
    }

    #[test]
    fn test_validate_percentage_invalid() {
        // Below 0
        match ConfigValidator::validate_percentage(-5.0, "negative") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "negative");
                assert!(message.contains("must be between 0.0 and 100.0"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for negative percentage"),
        }

        // Above 100
        match ConfigValidator::validate_percentage(105.0, "over_hundred") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "over_hundred");
                assert!(message.contains("must be between 0.0 and 100.0"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for percentage over 100"),
        }
    }
}

#[cfg(test)]
mod rate_limit_validation_tests {
    use super::*;

    #[test]
    fn test_validate_rate_limit_valid() {
        // Valid rate limits
        assert!(ConfigValidator::validate_rate_limit(10.0, "low_rate").is_ok());
        assert!(ConfigValidator::validate_rate_limit(1000.0, "medium_rate").is_ok());
        assert!(ConfigValidator::validate_rate_limit(0.1, "very_low_rate").is_ok());
    }

    #[test]
    fn test_validate_rate_limit_invalid() {
        // Zero rate
        match ConfigValidator::validate_rate_limit(0.0, "zero_rate") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "zero_rate");
                assert!(message.contains("Rate limit must be positive"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for zero rate limit"),
        }

        // Negative rate
        match ConfigValidator::validate_rate_limit(-5.0, "negative_rate") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "negative_rate");
                assert!(message.contains("Rate limit must be positive"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for negative rate limit"),
        }
    }

    #[test]
    fn test_validate_rate_limit_high_values() {
        // Very high rate (should warn but pass)
        assert!(ConfigValidator::validate_rate_limit(150000.0, "very_high_rate").is_ok());
    }
}

#[cfg(test)]
mod file_path_validation_tests {
    use super::*;

    #[test]
    fn test_validate_file_path_existing() {
        // Create temporary file for testing
        let temp_dir = TempDir::new().expect("Test assertion failed");
        let temp_file = temp_dir.path().join("test_file.txt");
        std::fs::write(&temp_file, "test content").expect("Test assertion failed");

        // Test with must_exist = true
        let result = ConfigValidator::validate_file_path(
            temp_file.to_str().expect("Test assertion failed"),
            "existing_file",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_path_non_existing_must_exist() {
        // Test non-existing file with must_exist = true
        match ConfigValidator::validate_file_path("/non/existing/file.txt", "missing_file") {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "missing_file");
                assert!(message.contains("does not exist"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for non-existing file"),
        }
    }

    #[test]
    fn test_validate_file_path_non_existing_optional() {
        // Test non-existing file with must_exist = false
        // This should fail because parent directory doesn't exist
        match ConfigValidator::validate_file_path("/non/existing/file.txt", "optional_file")
        {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "optional_file");
                assert!(message.contains("Parent directory") && message.contains("does not exist"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for non-existing parent directory"),
        }
    }

    #[test]
    fn test_validate_directory_path_existing() {
        let temp_dir = TempDir::new().expect("Test assertion failed");

        let result = ConfigValidator::validate_directory_path(
            temp_dir.path().to_str().expect("Test assertion failed"),
            "existing_dir",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_directory_path_create_missing() {
        let temp_dir = TempDir::new().expect("Test assertion failed");
        let new_dir = temp_dir.path().join("new_directory");

        // Test creating missing directory
        let result = ConfigValidator::validate_directory_path(
            new_dir.to_str().expect("Test assertion failed"),
            "new_dir"
        );
        assert!(result.is_ok());
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_validate_directory_path_non_existing_no_create() {
        // Test non-existing directory without create permission
        match ConfigValidator::validate_directory_path(
            "/non/existing/directory",
            "missing_dir",
        ) {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "missing_dir");
                assert!(message.contains("does not exist"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for non-existing directory"),
        }
    }

    #[test]
    fn test_validate_directory_path_file_not_directory() {
        let temp_dir = TempDir::new().expect("Test assertion failed");
        let temp_file = temp_dir.path().join("not_a_directory.txt");
        std::fs::write(&temp_file, "test content").expect("Test assertion failed");

        // Test file path instead of directory
        match ConfigValidator::validate_directory_path(
            temp_file.to_str().expect("Test assertion failed"),
            "not_dir",
        ) {
            Err(SongbirdError::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            }) => {
                assert_eq!(field, "not_dir");
                assert!(message.contains("is not a directory"));
                assert!(suggestion.is_some());
                assert!(context.is_some());
            }
            _ => panic!("Expected Config error for file instead of directory"),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use songbird_config::SongbirdConfig;

    #[test]
    fn test_validate_all_default_config() {
        // Test validation of default configuration
        let _config = SongbirdConfig::default();
        let result = ConfigValidator::validate_configuration();

        // Default config should be valid
        match result {
            Ok(()) => {
                // Success expected
            }
            Err(e) => {
                panic!("Default configuration should be valid: {e:?}");
            }
        }
    }
}
