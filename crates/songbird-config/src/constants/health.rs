//! Health monitoring constants
//!
//! Centralized constants for health checking and monitoring across the Songbird ecosystem.

use std::time::Duration;

/// Default health check interval
pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Default health check timeout
pub const DEFAULT_CHECK_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum consecutive failed health checks before marking unhealthy
pub const MAX_FAILED_CHECKS: u32 = 3;

/// Health check retry interval when failed
pub const RETRY_INTERVAL: Duration = Duration::from_secs(5);

/// Minimum time between health status updates
pub const MIN_UPDATE_INTERVAL: Duration = Duration::from_secs(1);

/// Default health endpoint path
pub const DEFAULT_HEALTH_ENDPOINT: &str = "/health";

/// Health check user agent
pub const HEALTH_CHECK_USER_AGENT: &str = "Songbird-Health-Check/1.0";

/// Maximum health check response size in bytes
pub const MAX_HEALTH_RESPONSE_SIZE: usize = 1024;

/// Health check connection timeout
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Health check read timeout
pub const READ_TIMEOUT: Duration = Duration::from_secs(5);
