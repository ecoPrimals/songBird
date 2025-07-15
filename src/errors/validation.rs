//! Configuration Validation Module
//!
//! Validation framework for configuration settings

use crate::errors::{Result, SongbirdError};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use url::Url;

/// Configuration validation utilities
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate port number is in valid range
    pub fn validate_port(port: u16, name: &str) -> Result<()> {
        match port {
            0 => Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Port cannot be 0".to_string(),
            }),
            1..=1023 => {
                tracing::warn!(
                    "Port {} is in privileged range (1-1023), may require elevated permissions",
                    port
                );
                Ok(())
            }
            1024..=65535 => Ok(()),
        }
    }

    /// Validate port range for configuration
    pub fn validate_port_range(start_port: u16, end_port: u16) -> Result<()> {
        if start_port > end_port {
            return Err(SongbirdError::Config {
                field: Some("port_range".to_string()),
                message: format!(
                    "Start port {start_port} cannot be greater than end port {end_port}"
                ),
            });
        }

        Self::validate_port(start_port, "start_port")?;
        Self::validate_port(end_port, "end_port")?;

        if end_port - start_port < 10 {
            tracing::warn!(
                "Port range is very small ({} ports), consider expanding for flexibility",
                end_port - start_port + 1
            );
        }

        Ok(())
    }

    /// Validate URL format
    pub fn validate_url(url_str: &str, name: &str) -> Result<Url> {
        let url = Url::parse(url_str).map_err(|e| SongbirdError::Config {
            field: Some(name.to_string()),
            message: format!("Invalid URL format: {e}"),
        })?;

        // Check for supported schemes
        match url.scheme() {
            "http" | "https" | "ws" | "wss" => Ok(url),
            scheme => Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!(
                    "Unsupported URL scheme '{scheme}', supported: http, https, ws, wss"
                ),
            }),
        }
    }

    /// Validate HTTP/HTTPS URL specifically
    pub fn validate_http_url(url_str: &str, name: &str) -> Result<Url> {
        let url = Self::validate_url(url_str, name)?;

        match url.scheme() {
            "http" | "https" => Ok(url),
            scheme => Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Expected HTTP/HTTPS URL, got scheme '{scheme}'"),
            }),
        }
    }

    /// Validate WebSocket URL specifically
    pub fn validate_websocket_url(url_str: &str, name: &str) -> Result<Url> {
        let url = Self::validate_url(url_str, name)?;

        match url.scheme() {
            "ws" | "wss" => Ok(url),
            scheme => Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Expected WebSocket URL (ws/wss), got scheme '{scheme}'"),
            }),
        }
    }

    /// Validate IP address format
    pub fn validate_ip_address(ip_str: &str, name: &str) -> Result<IpAddr> {
        ip_str.parse::<IpAddr>().map_err(|e| SongbirdError::Config {
            field: Some(name.to_string()),
            message: format!("Invalid IP address format: {e}"),
        })
    }

    /// Validate socket address (IP:port combination)
    pub fn validate_socket_address(addr_str: &str, name: &str) -> Result<SocketAddr> {
        addr_str
            .parse::<SocketAddr>()
            .map_err(|e| SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Invalid socket address format: {e}"),
            })
    }

    /// Validate timeout value bounds
    pub fn validate_timeout(
        timeout_ms: u64,
        name: &str,
        min_ms: u64,
        max_ms: u64,
    ) -> Result<Duration> {
        if timeout_ms < min_ms {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Timeout {timeout_ms} ms is below minimum {min_ms} ms"),
            });
        }

        if timeout_ms > max_ms {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Timeout {timeout_ms} ms exceeds maximum {max_ms} ms"),
            });
        }

        if timeout_ms > 300_000 {
            // 5 minutes
            tracing::warn!(
                "Timeout {} ms is very long (>5 minutes), consider reducing",
                timeout_ms
            );
        }

        Ok(Duration::from_millis(timeout_ms))
    }

    /// Validate connection timeout specifically
    pub fn validate_connection_timeout(timeout_ms: u64) -> Result<Duration> {
        Self::validate_timeout(timeout_ms, "connection_timeout", 100, 60_000) // 100ms to 60s
    }

    /// Validate request timeout specifically
    pub fn validate_request_timeout(timeout_ms: u64) -> Result<Duration> {
        Self::validate_timeout(timeout_ms, "request_timeout", 1_000, 300_000) // 1s to 5min
    }

    /// Validate health check interval
    pub fn validate_health_check_interval(interval_ms: u64) -> Result<Duration> {
        Self::validate_timeout(interval_ms, "health_check_interval", 1_000, 300_000)
        // 1s to 5min
    }

    /// Validate retry configuration
    pub fn validate_retry_config(max_retries: u32, retry_delay_ms: u64) -> Result<()> {
        if max_retries > 10 {
            return Err(SongbirdError::Config {
                field: Some("max_retries".to_string()),
                message: format!("Max retries {max_retries} exceeds reasonable limit of 10"),
            });
        }

        Self::validate_timeout(retry_delay_ms, "retry_delay", 10, 30_000)?; // 10ms to 30s
        Ok(())
    }

    /// Validate thread pool size
    pub fn validate_thread_pool_size(size: usize, name: &str) -> Result<()> {
        if size == 0 {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Thread pool size cannot be 0".to_string(),
            });
        }

        let cpu_count = num_cpus::get();
        if size > cpu_count * 4 {
            tracing::warn!(
                "Thread pool size {} is much larger than CPU count {} * 4, may cause overhead",
                size,
                cpu_count
            );
        }

        Ok(())
    }

    /// Validate buffer size
    pub fn validate_buffer_size(
        size: usize,
        name: &str,
        min_size: usize,
        max_size: usize,
    ) -> Result<()> {
        if size < min_size {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Buffer size {size} is below minimum {min_size}"),
            });
        }

        if size > max_size {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Buffer size {size} exceeds maximum {max_size}"),
            });
        }

        // Check if size is power of 2 for optimal performance
        if !size.is_power_of_two() {
            tracing::warn!("Buffer size {} is not a power of 2, consider using power of 2 for better performance", size);
        }

        Ok(())
    }

    /// Validate memory limit
    pub fn validate_memory_limit(limit_mb: u64) -> Result<()> {
        if limit_mb < 64 {
            return Err(SongbirdError::Config {
                field: Some("memory_limit".to_string()),
                message: "Memory limit cannot be less than 64 MB".to_string(),
            });
        }

        // Get system memory to provide warnings
        if let Ok(sys_info) = sys_info::mem_info() {
            let total_memory_mb = sys_info.total / 1024; // Convert KB to MB

            if limit_mb > total_memory_mb {
                return Err(SongbirdError::Config {
                    field: Some("memory_limit".to_string()),
                    message: format!(
                        "Memory limit {limit_mb} MB exceeds system memory {total_memory_mb} MB"
                    ),
                });
            }

            if limit_mb > total_memory_mb / 2 {
                tracing::warn!(
                    "Memory limit {} MB is more than half of system memory {} MB",
                    limit_mb,
                    total_memory_mb
                );
            }
        }

        Ok(())
    }

    /// Validate percentage value (0-100)
    pub fn validate_percentage(value: f64, name: &str) -> Result<()> {
        if !(0.0..=100.0).contains(&value) {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Percentage {value} must be between 0.0 and 100.0"),
            });
        }

        Ok(())
    }

    /// Validate rate (requests per second)
    pub fn validate_rate_limit(rate: f64, name: &str) -> Result<()> {
        if rate <= 0.0 {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: "Rate limit must be positive".to_string(),
            });
        }

        if rate > 100_000.0 {
            tracing::warn!(
                "Rate limit {} is very high (>100k/s), ensure system can handle this load",
                rate
            );
        }

        Ok(())
    }

    /// Validate file path exists and is accessible
    pub fn validate_file_path(
        path: &str,
        name: &str,
        must_exist: bool,
    ) -> Result<std::path::PathBuf> {
        let path_buf = std::path::PathBuf::from(path);

        if must_exist && !path_buf.exists() {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("File path '{path}' does not exist"),
            });
        }

        if let Some(parent) = path_buf.parent() {
            if !parent.exists() {
                return Err(SongbirdError::Config {
                    field: Some(name.to_string()),
                    message: format!("Parent directory '{}' does not exist", parent.display()),
                });
            }
        }

        Ok(path_buf)
    }

    /// Validate directory path exists and is writable
    pub fn validate_directory_path(
        path: &str,
        name: &str,
        create_if_missing: bool,
    ) -> Result<std::path::PathBuf> {
        let path_buf = std::path::PathBuf::from(path);

        if !path_buf.exists() {
            if create_if_missing {
                std::fs::create_dir_all(&path_buf).map_err(|e| SongbirdError::Config {
                    field: Some(name.to_string()),
                    message: format!("Failed to create directory '{path}': {e}"),
                })?;
            } else {
                return Err(SongbirdError::Config {
                    field: Some(name.to_string()),
                    message: format!("Directory '{path}' does not exist"),
                });
            }
        }

        if !path_buf.is_dir() {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Path '{path}' is not a directory"),
            });
        }

        // Test write permissions
        let test_file = path_buf.join(".write_test");
        if let Err(e) = std::fs::write(&test_file, "test") {
            return Err(SongbirdError::Config {
                field: Some(name.to_string()),
                message: format!("Directory '{path}' is not writable: {e}"),
            });
        }
        let _ = std::fs::remove_file(test_file); // Clean up test file

        Ok(path_buf)
    }

    /// Comprehensive configuration validation (simplified version)
    pub fn validate_basic_config() -> Result<()> {
        // Basic validation without complex config dependencies
        Self::validate_port(8080, "default_port")?;
        Self::validate_timeout(30000, "default_timeout", 1000, 300000)?;
        Self::validate_thread_pool_size(num_cpus::get(), "worker_threads")?;

        tracing::info!("Basic configuration validation completed successfully");
        Ok(())
    }
}
