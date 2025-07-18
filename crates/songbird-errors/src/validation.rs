//! Configuration validation utilities with proper error handling

use crate::songbird_errors::{Result, SongbirdError};
use std::path::Path;
use tracing::warn;
use url::Url;

/// Configuration validator with comprehensive error handling
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate port number
    ///
    /// # Errors
    ///
    /// Returns an error if the port is 0 or invalid
    pub fn validate_port(port: u16, name: &str) -> Result<()> {
        match port {
            0 => Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Port cannot be 0".to_string(),
                context: Some("port_validation".to_string()),
                suggestion: Some("Use a port number between 1024 and 65535".to_string()),
            }),
            1..=1023 => {
                tracing::warn!(
                    "Port {} is in privileged range (1-1023), may require elevated permissions",
                    port
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Validate port range
    ///
    /// # Errors
    ///
    /// Returns an error if the port range is invalid
    pub fn validate_port_range(start: u16, end: u16) -> Result<()> {
        if start >= end {
            return Err(SongbirdError::Config {
                field: Some("port_range".to_string()),
                message: format!("Start port {start} cannot be greater than end port {end}"),
                context: Some("port_range_validation".to_string()),
                suggestion: Some("Ensure start port is less than end port".to_string()),
            });
        }

        // Validate that neither start nor end port is 0
        if start == 0 {
            return Err(SongbirdError::Config {
                field: Some("port_range".to_string()),
                message: "Start port cannot be 0".to_string(),
                context: Some("port_range_validation".to_string()),
                suggestion: Some("Use a port greater than 0".to_string()),
            });
        }

        if end == 0 {
            return Err(SongbirdError::Config {
                field: Some("port_range".to_string()),
                message: "End port cannot be 0".to_string(),
                context: Some("port_range_validation".to_string()),
                suggestion: Some("Use a port greater than 0".to_string()),
            });
        }

        if end - start < 10 {
            warn!(
                "Port range {start}-{end} is very small, consider expanding for better performance"
            );
        }

        Ok(())
    }

    /// Validate URL
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is malformed
    pub fn validate_url(url_str: &str, name: &str) -> Result<()> {
        let url = Url::parse(url_str).map_err(|_| SongbirdError::Config {
            field: Some(name.to_string()),
            message: "Invalid URL format".to_string(),
            context: Some("url_validation".to_string()),
            suggestion: Some("Provide a valid URL with proper scheme and host".to_string()),
        })?;

        match url.scheme() {
            "http" | "https" | "ws" | "wss" => Ok(()),
            scheme => Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Unsupported URL scheme: {scheme}"),
                context: Some("url_scheme_validation".to_string()),
                suggestion: Some("Use http:// or https:// scheme".to_string()),
            }),
        }
    }

    /// Validate WebSocket URL
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket URL is malformed
    pub fn validate_websocket_url(url_str: &str, name: &str) -> Result<()> {
        let url = Url::parse(url_str).map_err(|e| SongbirdError::Config {
            field: Some(name.to_string()),
            message: format!("Invalid WebSocket URL: {e}"),
            context: Some("websocket_url_validation".to_string()),
            suggestion: Some(
                "Provide a valid WebSocket URL with ws:// or wss:// scheme".to_string(),
            ),
        })?;

        match url.scheme() {
            "ws" | "wss" => Ok(()),
            _ => Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Expected WebSocket URL".to_string(),
                context: Some("websocket_scheme_validation".to_string()),
                suggestion: Some("Use ws:// or wss:// scheme for WebSocket URLs".to_string()),
            }),
        }
    }

    /// Validate HTTP URL
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP URL is malformed
    pub fn validate_http_url(url_str: &str, name: &str) -> Result<()> {
        let url = Url::parse(url_str).map_err(|e| SongbirdError::Config {
            field: Some(name.to_string()),
            message: format!("Invalid HTTP URL: {e}"),
            context: Some("http_url_validation".to_string()),
            suggestion: Some(
                "Provide a valid HTTP URL with http:// or https:// scheme".to_string(),
            ),
        })?;

        match url.scheme() {
            "http" | "https" => Ok(()),
            _ => Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Expected HTTP/HTTPS URL".to_string(),
                context: Some("http_scheme_validation".to_string()),
                suggestion: Some("Use http:// or https:// scheme for HTTP URLs".to_string()),
            }),
        }
    }

    /// Validate IP address
    ///
    /// # Errors
    ///
    /// Returns an error if the IP address is malformed
    pub fn validate_ip_address(ip_str: &str, name: &str) -> Result<()> {
        ip_str
            .parse::<std::net::IpAddr>()
            .map_err(|_| SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Invalid IP address format".to_string(),
                context: Some("ip_address_validation".to_string()),
                suggestion: Some("Provide a valid IPv4 or IPv6 address".to_string()),
            })?;
        Ok(())
    }

    /// Validate socket address
    ///
    /// # Errors
    ///
    /// Returns an error if the socket address is malformed
    pub fn validate_socket_address(addr_str: &str, name: &str) -> Result<()> {
        addr_str
            .parse::<std::net::SocketAddr>()
            .map_err(|_| SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Invalid socket address format".to_string(),
                context: Some("socket_address_validation".to_string()),
                suggestion: Some("Provide a valid socket address in format IP:PORT".to_string()),
            })?;
        Ok(())
    }

    /// Validate timeout
    ///
    /// # Errors
    ///
    /// Returns an error if the timeout is invalid
    pub fn validate_timeout(timeout_ms: u64, name: &str, min_ms: u64, max_ms: u64) -> Result<()> {
        if timeout_ms < min_ms {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Timeout {timeout_ms}ms is below minimum {min_ms}ms"),
                context: Some("timeout_validation".to_string()),
                suggestion: Some(format!("Use a timeout between {min_ms}ms and {max_ms}ms")),
            });
        }

        if timeout_ms > max_ms {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Timeout {timeout_ms}ms exceeds maximum {max_ms}ms"),
                context: Some("timeout_validation".to_string()),
                suggestion: Some(format!("Use a timeout between {min_ms}ms and {max_ms}ms")),
            });
        }

        Ok(())
    }

    /// Validate connection timeout
    ///
    /// # Errors
    ///
    /// Returns an error if the connection timeout is invalid
    pub fn validate_connection_timeout(timeout_ms: u64) -> Result<()> {
        Self::validate_timeout(timeout_ms, "connection_timeout", 100, 60000)?;
        Ok(())
    }

    /// Validate request timeout
    ///
    /// # Errors
    ///
    /// Returns an error if the request timeout is invalid
    pub fn validate_request_timeout(timeout_ms: u64) -> Result<()> {
        Self::validate_timeout(timeout_ms, "request_timeout", 1000, 300000)?;
        Ok(())
    }

    /// Validate retry configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the retry configuration is invalid
    pub fn validate_retry_config(max_retries: u32, retry_delay_ms: u64) -> Result<()> {
        if max_retries > 10 {
            return Err(SongbirdError::Config {
                field: Some("max_retries".to_string()),
                message: format!("Max retries {max_retries} exceeds reasonable limit"),
                context: Some("retry_validation".to_string()),
                suggestion: Some("Use between 0 and 10 retry attempts".to_string()),
            });
        }

        Self::validate_timeout(retry_delay_ms, "retry_delay", 100, 10000)?;
        Ok(())
    }

    /// Validate thread pool size
    ///
    /// # Errors
    ///
    /// Returns an error if the thread pool size is invalid
    pub fn validate_thread_pool_size(size: usize, name: &str) -> Result<()> {
        if size == 0 {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Thread pool size cannot be 0".to_string(),
                context: Some("thread_pool_validation".to_string()),
                suggestion: Some("Use at least 1 thread".to_string()),
            });
        }

        if size > 1000 {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Thread pool size {size} is too large"),
                context: Some("thread_pool_validation".to_string()),
                suggestion: Some("Use between 1 and 1000 threads".to_string()),
            });
        }

        Ok(())
    }

    /// Validate memory limit
    ///
    /// # Errors
    ///
    /// Returns an error if the memory limit is invalid
    pub fn validate_memory_limit(limit_mb: u64, name: &str) -> Result<()> {
        if limit_mb < 64 {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Memory limit cannot be less than 64 MB".to_string(),
                context: Some("memory_limit_validation".to_string()),
                suggestion: Some("Use at least 64MB".to_string()),
            });
        }

        if limit_mb > 1024 * 1024 {
            warn!("Memory limit {limit_mb}MB is very high, ensure system has enough memory");
        }

        Ok(())
    }

    /// Validate buffer size
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer size is invalid
    pub fn validate_buffer_size(size: usize, name: &str) -> Result<()> {
        if size < 1024 {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Buffer size {size} below minimum"),
                context: Some("buffer_size_validation".to_string()),
                suggestion: Some("Use at least 1024 bytes".to_string()),
            });
        }

        if size > 65536 {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Buffer size {size} exceeds maximum"),
                context: Some("buffer_size_validation".to_string()),
                suggestion: Some("Use at most 64KB".to_string()),
            });
        }

        Ok(())
    }

    /// Validate percentage
    ///
    /// # Errors
    ///
    /// Returns an error if the percentage is invalid
    pub fn validate_percentage(value: f64, name: &str) -> Result<()> {
        if !(0.0..=100.0).contains(&value) {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Percentage {value} must be between 0.0 and 100.0"),
                context: Some("percentage_validation".to_string()),
                suggestion: Some("Use a value between 0.0 and 100.0".to_string()),
            });
        }
        Ok(())
    }

    /// Validate rate limit
    ///
    /// # Errors
    ///
    /// Returns an error if the rate limit is invalid
    pub fn validate_rate_limit(rate: f64, name: &str) -> Result<()> {
        if rate <= 0.0 {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Rate limit must be positive".to_string(),
                context: Some("rate_limit_validation".to_string()),
                suggestion: Some("Use a positive rate limit value".to_string()),
            });
        }

        if rate > 100_000.0 {
            warn!("Rate limit {rate} is very high, ensure system can handle this load");
        }

        Ok(())
    }

    /// Validate file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file path is invalid
    pub fn validate_file_path(path_str: &str, name: &str) -> Result<()> {
        let path = Path::new(path_str);

        if !path.exists() {
            // Check if parent directory exists
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    return Err(SongbirdError::Config {
                        field: Some(name.to_string()),
                        message: format!("Parent directory does not exist: {}", parent.display()),
                        context: Some("file_path_validation".to_string()),
                        suggestion: Some("Ensure the parent directory exists".to_string()),
                    });
                }
            }
            
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("File does not exist: {path_str}"),
                context: Some("file_path_validation".to_string()),
                suggestion: Some("Ensure the file exists and is accessible".to_string()),
            });
        }

        if !path.is_file() {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Path is not a file: {path_str}"),
                context: Some("file_path_validation".to_string()),
                suggestion: Some("Provide a path to a file, not a directory".to_string()),
            });
        }

        Ok(())
    }

    /// Validate directory path
    ///
    /// # Errors
    ///
    /// Returns an error if the directory path is invalid
    pub fn validate_directory_path(path_str: &str, name: &str) -> Result<()> {
        let path = Path::new(path_str);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(SongbirdError::Config {
                    field: Some(name.to_string()),
                    message: format!("Parent directory does not exist: {}", parent.display()),
                    context: Some("directory_path_validation".to_string()),
                    suggestion: Some("Ensure the parent directory exists".to_string()),
                });
            }
        }

        if !path.exists() {
            // Try to create the directory
            std::fs::create_dir_all(path).map_err(|e| SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Could not create directory: {e}"),
                context: Some("directory_path_validation".to_string()),
                suggestion: Some("Check write permissions for the directory path".to_string()),
            })?;
        } else if !path.is_dir() {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Path is not a directory: {path_str}"),
                context: Some("directory_path_validation".to_string()),
                suggestion: Some("Provide a path to a directory, not a file".to_string()),
            });
        }

        Ok(())
    }

    /// Validate configuration as a whole
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid
    pub fn validate_configuration() -> Result<()> {
        // Validate default timeout
        Self::validate_timeout(30000, "default_timeout", 1000, 300000)?;

        // Validate other common values
        Self::validate_port(8080, "default_port")?;
        Self::validate_thread_pool_size(4, "default_threads")?;
        Self::validate_memory_limit(1024, "default_memory")?;
        Self::validate_buffer_size(8192, "default_buffer")?;
        Self::validate_percentage(80.0, "default_threshold")?;
        Self::validate_rate_limit(1000.0, "default_rate")?;

        Ok(())
    }
}

// Helper function for testing
#[cfg(test)]
pub fn validate_test_config() -> Result<()> {
    use std::env;
    use std::fs;

    // Create a temporary file for testing
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join("test_config.toml");
    fs::write(&temp_file, "test content").map_err(|e| SongbirdError::Config {
        field: Some("test_file".to_string()),
        message: format!("Failed to create test file: {e}"),
        context: Some("test_setup".to_string()),
        suggestion: Some("Check write permissions in temp directory".to_string()),
    })?;

    // Validate the file
    ConfigValidator::validate_file_path(temp_file.to_str().unwrap(), "test_file")?;

    // Clean up
    fs::remove_file(&temp_file).map_err(|e| SongbirdError::Config {
        field: Some("test_cleanup".to_string()),
        message: format!("Failed to clean up test file: {e}"),
        context: Some("test_cleanup".to_string()),
        suggestion: Some("Check if file exists and is writable".to_string()),
    })?;

    Ok(())
}
