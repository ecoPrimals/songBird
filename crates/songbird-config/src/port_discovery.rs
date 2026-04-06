// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Port Discovery - Dynamic Port Allocation
//!
//! Eliminates hardcoded ports by discovering available ports at runtime.
//! Provides intelligent port selection based on environment and availability.

use songbird_types::{SongbirdError, SongbirdResult};
use std::net::TcpListener;
use tracing::{debug, info, warn};

/// Discover an available port in the default range (8000-9000)
///
/// # Examples
/// ```no_run
/// use songbird_config::port_discovery::discover_available_port;
///
/// let port = discover_available_port();
/// println!("Using port: {}", port);
/// ```
#[must_use]
pub fn discover_available_port() -> u16 {
    discover_available_port_in_range(8000, 9000)
}

/// Discover an available port in a specific range
///
/// # Arguments
/// * `start` - Start of port range (inclusive)
/// * `end` - End of port range (exclusive)
///
/// # Examples
/// ```no_run
/// use songbird_config::port_discovery::discover_available_port_in_range;
///
/// let port = discover_available_port_in_range(3000, 4000);
/// ```
#[must_use]
pub fn discover_available_port_in_range(start: u16, end: u16) -> u16 {
    debug!("Discovering available port in range {start}-{end}");

    for port in start..end {
        if is_port_available(port) {
            info!("Discovered available port: {port}");
            return port;
        }
    }

    warn!("No available ports in range {start}-{end}, using start port");
    start
}

/// Check if a specific port is available
///
/// # Arguments
/// * `port` - Port number to check
///
/// # Returns
/// `true` if port is available, `false` otherwise
#[must_use]
pub fn is_port_available(port: u16) -> bool {
    is_port_available_on("0.0.0.0", port)
}

/// Check if a specific port is available on a given interface
///
/// # Arguments
/// * `interface` - Network interface address
/// * `port` - Port number to check
///
/// # Returns
/// `true` if port is available, `false` otherwise
#[must_use]
pub fn is_port_available_on(interface: &str, port: u16) -> bool {
    let addr = format!("{interface}:{port}");
    TcpListener::bind(&addr).is_ok()
}

/// Discover multiple available ports
///
/// # Arguments
/// * `count` - Number of ports to discover
///
/// # Returns
/// Vector of available port numbers
///
/// # Errors
/// Returns error if unable to discover enough ports
pub fn discover_multiple_ports(count: usize) -> SongbirdResult<Vec<u16>> {
    discover_multiple_ports_in_range(count, 8000, 9000)
}

/// Discover multiple available ports in a specific range
///
/// # Arguments
/// * `count` - Number of ports to discover
/// * `start` - Start of port range
/// * `end` - End of port range
///
/// # Returns
/// Vector of available port numbers
///
/// # Errors
/// Returns error if unable to discover enough ports
pub fn discover_multiple_ports_in_range(
    count: usize,
    start: u16,
    end: u16,
) -> SongbirdResult<Vec<u16>> {
    let mut ports = Vec::with_capacity(count);

    for port in start..end {
        if is_port_available(port) {
            ports.push(port);
            if ports.len() >= count {
                return Ok(ports);
            }
        }
    }

    if ports.is_empty() {
        Err(SongbirdError::network(format!(
            "Unable to discover any available ports in range {start}-{end}"
        )))
    } else {
        Ok(ports) // Return what we found, even if less than requested
    }
}

/// Get port for a specific service from environment or discovery
///
/// Checks `SONGBIRD_{SERVICE}_PORT` environment variable first,
/// then discovers an available port if not set.
///
/// # Arguments
/// * `service_name` - Name of the service (e.g., "orchestrator", "discovery")
///
/// # Examples
/// ```no_run
/// use songbird_config::port_discovery::get_service_port;
///
/// let port = get_service_port("orchestrator");
/// ```
#[must_use]
pub fn get_service_port(service_name: &str) -> u16 {
    let env_var = format!("SONGBIRD_{}_PORT", service_name.to_uppercase());

    songbird_process_env::var(&env_var).ok().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        debug!("No port in {env_var}, discovering available port");
        discover_available_port()
    })
}

/// Validate that a port is in the safe range
///
/// Ports below 1024 require special privileges on most systems.
/// This function checks if a port is in the safe, unprivileged range.
///
/// # Arguments
/// * `port` - Port number to validate
///
/// # Returns
/// `true` if port is safe to use without privileges
#[must_use]
pub fn is_safe_port(port: u16) -> bool {
    (1024..=65535).contains(&port)
}

/// Get the recommended port range for the current environment
///
/// Returns different ranges based on:
/// - User privileges
/// - Environment (production, staging, development)
/// - System configuration
#[must_use]
pub fn get_recommended_port_range() -> (u16, u16) {
    // Check if we can use privileged ports
    let can_use_privileged = songbird_process_env::var("SONGBIRD_ALLOW_PRIVILEGED_PORTS").is_ok();

    // Check environment
    let env =
        songbird_process_env::var("SONGBIRD_ENV").unwrap_or_else(|_| "development".to_string());

    match (can_use_privileged, env.as_str()) {
        (true, "production") => (80, 100),     // HTTP range
        (true, _) => (8080, 8100),             // Development with privileges
        (false, "production") => (8000, 8100), // Production safe range
        (false, "staging") => (8100, 8200),    // Staging range
        (false, "testing") => (8200, 8300),    // Testing range
        (false, _) => (8300, 8400),            // Development range
    }
}

/// Discover a port in the recommended range for current environment
#[must_use]
pub fn discover_port_for_environment() -> u16 {
    let (start, end) = get_recommended_port_range();
    discover_available_port_in_range(start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_available_port() {
        let port = discover_available_port();
        assert!((8000..9000).contains(&port));
    }

    #[test]
    fn test_discover_port_in_range() {
        let port = discover_available_port_in_range(9000, 9100);
        assert!((9000..9100).contains(&port));
    }

    #[test]
    fn test_safe_port_validation() {
        assert!(!is_safe_port(80)); // Privileged
        assert!(!is_safe_port(443)); // Privileged
        assert!(is_safe_port(8080)); // Safe
        assert!(is_safe_port(3000)); // Safe
        assert!(is_safe_port(65535)); // Safe (max)
    }

    #[test]
    fn test_discover_multiple_ports() {
        let result = discover_multiple_ports_in_range(3, 9500, 9600);
        assert!(result.is_ok());
        let ports = result.expect("Port discovery should succeed in test");
        assert!(!ports.is_empty());
        assert!(ports.len() <= 3);

        // Verify all ports are unique
        let unique: std::collections::HashSet<_> = ports.iter().collect();
        assert_eq!(unique.len(), ports.len());
    }

    #[test]
    fn test_recommended_port_range() {
        let (start, end) = get_recommended_port_range();
        assert!(start < end);
        assert!(start >= 80); // At least this low
        // Note: end is u16, so always <= 65535 (type guarantees this)
    }

    #[test]
    fn test_get_service_port_default() {
        // Should discover a port if env var not set
        let port = get_service_port("test_service");
        assert!(port >= 8000);
    }
}
