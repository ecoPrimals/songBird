//! Service management constants
//!
//! Constants for service lifecycle, discovery, and management across the Songbird ecosystem.

use std::time::Duration;

// Startup and shutdown timeouts are defined in core.rs

/// Default service restart timeout
pub const DEFAULT_RESTART_TIMEOUT: Duration = Duration::from_secs(90);

/// Default service discovery interval
pub const DEFAULT_DISCOVERY_INTERVAL: Duration = Duration::from_secs(30);

/// Default service registration TTL
pub const DEFAULT_REGISTRATION_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Maximum service name length
pub const MAX_SERVICE_NAME_LENGTH: usize = 64;

/// Maximum service description length
pub const MAX_SERVICE_DESCRIPTION_LENGTH: usize = 256;

// Service check interval is defined in core.rs

/// Maximum number of service instances per type
pub const MAX_SERVICE_INSTANCES: usize = 100;

/// Default service port range start
pub const DEFAULT_SERVICE_PORT_START: u16 = 8000;

/// Default service port range end
pub const DEFAULT_SERVICE_PORT_END: u16 = 8999;

/// Service discovery timeout
pub const SERVICE_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum service metadata size in bytes
pub const MAX_SERVICE_METADATA_SIZE: usize = 4096; // 4KB

/// Default service priority (lower numbers = higher priority)
pub const DEFAULT_SERVICE_PRIORITY: u8 = 100;

/// Maximum number of service dependencies
pub const MAX_SERVICE_DEPENDENCIES: usize = 10;
