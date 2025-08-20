//! Resource management constants
//!
//! Constants for system resource limits, thresholds, and management policies.

use std::time::Duration;

// Memory threshold is defined in core.rs

/// Default CPU threshold percentage before throttling
pub const DEFAULT_CPU_THRESHOLD_PERCENT: f32 = 80.0;

/// Default disk usage threshold percentage
pub const DEFAULT_DISK_THRESHOLD_PERCENT: f32 = 85.0;

/// Maximum number of concurrent connections
pub const MAX_CONCURRENT_CONNECTIONS: usize = 1000;

/// Default connection pool size
pub const DEFAULT_CONNECTION_POOL_SIZE: usize = 10;

/// Resource monitoring interval
pub const RESOURCE_MONITORING_INTERVAL: Duration = Duration::from_secs(30);

/// Resource cleanup interval
pub const RESOURCE_CLEANUP_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes
pub const DEFAULT_CLEANUP_INTERVAL: Duration = RESOURCE_CLEANUP_INTERVAL;

/// Resource monitoring interval (alias for compatibility)
pub const DEFAULT_MONITORING_INTERVAL: Duration = RESOURCE_MONITORING_INTERVAL;

/// Leak detection interval
pub const DEFAULT_LEAK_DETECTION_INTERVAL: Duration = Duration::from_secs(600); // 10 minutes

/// Maximum resource age before cleanup
pub const DEFAULT_MAX_RESOURCE_AGE: Duration = Duration::from_secs(3600); // 1 hour

/// Resource tracking interval
pub const DEFAULT_TRACKING_INTERVAL: Duration = Duration::from_secs(60); // 1 minute

// Memory units for display are defined in core.rs

/// Default buffer size for I/O operations (renamed to avoid conflict)
pub const DEFAULT_RESOURCE_BUFFER_SIZE: usize = 8192; // 8KB

/// Maximum buffer size for I/O operations (renamed to avoid conflict)
pub const MAX_RESOURCE_BUFFER_SIZE: usize = 1_048_576; // 1MB

/// Default timeout for resource operations
pub const DEFAULT_RESOURCE_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum retry attempts for resource operations
pub const MAX_RESOURCE_RETRY_ATTEMPTS: u32 = 3;

/// Delay between resource operation retries
pub const RESOURCE_RETRY_DELAY: Duration = Duration::from_secs(1);
