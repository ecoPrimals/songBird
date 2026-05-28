// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🎯 Test Endpoint Fixtures
//!
//! **Zero-hardcoding test infrastructure**
//!
//! Provides dynamic endpoint generation for tests that:
//! - Respects environment variables
//! - Avoids port conflicts
//! - Demonstrates capability-based discovery
//! - Works across all test environments
//!
//! ## Usage
//!
//! ```rust
//! use songbird_test_utils::fixtures::endpoints::*;
//!
//! #[tokio::test]
//! async fn test_security_integration() {
//!     // Get endpoint for security capability
//!     let security_endpoint = test_endpoint("security");
//!     // Uses SECURITY_ENDPOINT env var or generates unique endpoint
//!     
//!     // Get dynamic port for capability
//!     let port = test_port("compute");
//!     // Uses COMPUTE_PORT env var or selects available port
//! }
//! ```

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::Mutex;

/// Global registry of allocated test ports to avoid conflicts
static PORT_REGISTRY: std::sync::LazyLock<Mutex<HashMap<String, u16>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Get test endpoint for a capability
///
/// Discovery order:
/// 1. `{CAPABILITY}_ENDPOINT` environment variable
/// 2. `TEST_{CAPABILITY}_ENDPOINT` environment variable
/// 3. Generated endpoint with dynamic port
///
/// # Examples
///
/// ```rust,ignore
/// use songbird_test_utils::fixtures::endpoints::test_endpoint;
///
/// // With environment variable:
/// // export SECURITY_ENDPOINT=http://localhost:9443
/// let endpoint = test_endpoint("security");
/// assert_eq!(endpoint, "http://localhost:9443");
///
/// // Without environment variable:
/// let endpoint = test_endpoint("compute");
/// // Returns: "http://127.0.0.1:PORT" with dynamically allocated PORT
/// ```
#[must_use]
pub fn test_endpoint(capability: &str) -> String {
    // Try environment variables first
    let env_key = format!("{}_ENDPOINT", capability.to_uppercase());
    if let Ok(endpoint) = songbird_process_env::var(&env_key) {
        return endpoint;
    }

    let test_env_key = format!("TEST_{}_ENDPOINT", capability.to_uppercase());
    if let Ok(endpoint) = songbird_process_env::var(&test_env_key) {
        return endpoint;
    }

    // Generate endpoint with dynamic port
    let port = test_port(capability);
    format!("http://127.0.0.1:{port}")
}

/// Get test port for a capability
///
/// Discovery order:
/// 1. `{CAPABILITY}_PORT` environment variable
/// 2. `TEST_{CAPABILITY}_PORT` environment variable
/// 3. Previously allocated port for this capability (cached)
/// 4. Dynamically allocated available port
///
/// Ports are cached per capability to ensure consistency within a test run.
///
/// # Examples
///
/// ```rust,ignore
/// use songbird_test_utils::fixtures::endpoints::test_port;
///
/// // With environment variable:
/// // export SECURITY_PORT=9443
/// let port = test_port("security");
/// assert_eq!(port, 9443);
///
/// // Without environment variable:
/// let port1 = test_port("compute");
/// let port2 = test_port("compute");
/// assert_eq!(port1, port2); // Same capability = same port (cached)
/// ```
pub fn test_port(capability: &str) -> u16 {
    // Try environment variables first
    let env_key = format!("{}_PORT", capability.to_uppercase());
    if let Ok(port_str) = songbird_process_env::var(&env_key)
        && let Ok(port) = port_str.parse()
    {
        return port;
    }

    let test_env_key = format!("TEST_{}_PORT", capability.to_uppercase());
    if let Ok(port_str) = songbird_process_env::var(&test_env_key)
        && let Ok(port) = port_str.parse()
    {
        return port;
    }

    // Atomically check-or-insert to avoid TOCTOU race between concurrent tests
    let mut registry = PORT_REGISTRY.lock().expect("PORT_REGISTRY lock poisoned");
    if let Some(&port) = registry.get(capability) {
        return port;
    }

    let port = allocate_available_port();
    registry.insert(capability.to_string(), port);
    port
}

/// Get test endpoint with specific port override
///
/// Useful when you need to force a specific port for testing.
///
/// # Examples
///
/// ```rust
/// use songbird_test_utils::fixtures::endpoints::test_endpoint_with_port;
///
/// let endpoint = test_endpoint_with_port("security", 9443);
/// assert_eq!(endpoint, "http://127.0.0.1:9443");
/// ```
#[must_use]
pub fn test_endpoint_with_port(capability: &str, port: u16) -> String {
    // Check if security capability should use HTTPS
    let protocol = if capability == "security" && port == 8443 {
        "https"
    } else {
        "http"
    };

    format!("{protocol}://127.0.0.1:{port}")
}

/// Allocate an available port from the OS
///
/// Uses OS port allocation by binding to port 0 and retrieving the assigned port.
/// This ensures we get an available port without conflicts.
fn allocate_available_port() -> u16 {
    // Bind to port 0 to let OS choose an available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to ephemeral port");

    // Listener is dropped here, freeing the port for actual use
    listener.local_addr().expect("Failed to get local address").port()
}

/// Clear port registry (useful for test isolation)
///
/// Call this in test setup/teardown if you need fresh port allocation.
///
/// # Examples
///
/// ```rust,no_run
/// use songbird_test_utils::fixtures::endpoints::{clear_port_registry, test_port};
///
/// fn example() {
///     clear_port_registry();
///     let port = test_port("isolated");
///     // Fresh allocation
/// }
/// ```
pub fn clear_port_registry() {
    let mut registry = PORT_REGISTRY.lock().expect("lock poisoned");
    registry.clear();
}

/// Get test bind address for capability
///
/// Returns `127.0.0.1:PORT` where PORT is determined via `test_port()`.
///
/// # Examples
///
/// ```rust
/// use songbird_test_utils::fixtures::endpoints::test_bind_address;
///
/// let addr = test_bind_address("discovery");
/// // Returns: "127.0.0.1:PORT"
/// ```
#[must_use]
pub fn test_bind_address(capability: &str) -> String {
    let port = test_port(capability);
    format!("127.0.0.1:{port}")
}

/// Get test socket address for capability
///
/// Returns a parsed `SocketAddr` ready for binding.
///
/// # Examples
///
/// ```rust
/// use songbird_test_utils::fixtures::endpoints::test_socket_addr;
/// use std::net::TcpListener;
///
/// let addr = test_socket_addr("http");
/// let listener = TcpListener::bind(addr).unwrap();
/// ```
#[must_use]
pub fn test_socket_addr(capability: &str) -> std::net::SocketAddr {
    let port = test_port(capability);
    format!("127.0.0.1:{port}").parse().expect("Failed to parse socket address")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_generates_valid_url() {
        // ✅ Concurrent-safe: no env var mutation, just verify the function works
        let endpoint = test_endpoint("test_security_gen");
        assert!(endpoint.starts_with("http://"));
        assert!(endpoint.contains("127.0.0.1"));
    }

    #[test]
    fn test_port_generates_valid_port() {
        let port = test_port("test_compute_gen_v2");
        assert!(port > 0);
    }

    #[test]
    fn test_port_allocation_is_cached() {
        let capability = "cache_stability_isolated_v4";
        let port1 = test_port(capability);
        let registry =
            super::PORT_REGISTRY.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let cached = registry.get(capability).copied();
        drop(registry);
        assert_eq!(cached, Some(port1), "Port should be cached in registry after allocation");
    }

    #[test]
    fn test_different_capabilities_get_different_ports() {
        let port1 = test_port("diff_cap_v3_alpha_x7f");
        let port2 = test_port("diff_cap_v3_beta_y8g");
        assert_ne!(port1, port2, "Different capabilities should get different ports");
    }

    #[test]
    fn test_endpoint_with_port_override() {
        let endpoint = test_endpoint_with_port("storage", 8888);
        assert_eq!(endpoint, "http://127.0.0.1:8888");
    }

    #[test]
    fn test_security_uses_https() {
        let endpoint = test_endpoint_with_port("security", 8443);
        assert!(endpoint.starts_with("https://"), "Security should use HTTPS");
    }

    #[test]
    fn test_bind_address_format() {
        let addr = test_bind_address("bind_fmt_unique_v2");
        assert!(addr.starts_with("127.0.0.1:"), "Bind address should start with 127.0.0.1:");
    }

    #[test]
    fn test_socket_addr_parseable() {
        let addr = test_socket_addr("sockaddr_unique_v2");
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert!(addr.port() > 0, "Port should be allocated");
    }

    #[test]
    fn test_clear_registry_does_not_panic() {
        let _port = test_port("clear_test_isolated_v2");
        clear_port_registry();
        let port2 = test_port("clear_test_isolated_v2");
        assert!(port2 > 0);
    }
}
