// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-based port allocation with zero hardcoding
//!
//! # Philosophy
//! - **No hardcoded port numbers** in production code
//! - **Capability-based allocation** (port determined by capability, not service name)
//! - **OS-managed allocation** for ephemeral/test ports
//! - **Environment-aware** port ranges
//! - **Collision-free** port assignment
//!
//! # Modern Approach
//! Instead of hardcoding "orchestrator uses 8080", we:
//! 1. Let OS assign available ports (development/test)
//! 2. Use capability-based port ranges (production)
//! 3. Discover actual ports at runtime
//! 4. Register with discovery service

use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::ops::Range;

use super::hosts_evolved::Environment;

/// Port allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortStrategy {
    /// OS assigns an available port (ephemeral)
    OsAssigned,

    /// Fixed port from environment variable
    Fixed(u16),

    /// Allocated from a capability-specific range
    CapabilityRange {
        /// Start of the port range (inclusive)
        start: u16,
        /// End of the port range (exclusive)
        end: u16,
    },
}

/// Port allocator with capability-based assignment
#[derive(Debug, Clone)]
pub struct PortAllocator {
    /// Strategy for port allocation
    strategy: PortStrategy,
}

impl PortAllocator {
    /// Create a new port allocator for the current environment
    #[must_use]
    pub fn new() -> Self {
        let environment = Environment::detect();
        let strategy = Self::strategy_for_environment(environment);

        Self {
            strategy,
        }
    }

    /// Determine allocation strategy based on environment
    const fn strategy_for_environment(env: Environment) -> PortStrategy {
        match env {
            // Development/Test: Let OS assign to avoid conflicts
            Environment::Development | Environment::Test => PortStrategy::OsAssigned,

            // Production: Use capability-based ranges
            Environment::Production | Environment::Staging => PortStrategy::CapabilityRange {
                start: 8000,
                end: 8999,
            },
        }
    }

    /// Allocate a port for a specific capability
    ///
    /// # Modern Approach
    /// - **Development/Test**: OS assigns an available port
    /// - **Production**: Allocate from capability-specific range
    /// - **Collision Detection**: Verify port is actually available
    ///
    /// # Returns
    /// A bound TCP listener on the allocated port
    ///
    /// # Errors
    /// Returns error if no ports are available in the capability range
    pub fn allocate_for_capability(&self, capability: &str) -> Result<TcpListener, std::io::Error> {
        match self.strategy {
            PortStrategy::OsAssigned => {
                // OS assigns port (bind to 0)
                TcpListener::bind("0.0.0.0:0")
            }

            PortStrategy::Fixed(port) => {
                // Try fixed port
                TcpListener::bind(("0.0.0.0", port))
            }

            PortStrategy::CapabilityRange {
                start,
                end,
            } => {
                // Allocate from capability-specific range
                let range = self.capability_range(capability, start..end);
                self.find_available_port_in_range(range)
            }
        }
    }

    /// Get capability-specific port range
    ///
    /// # Capability Ranges (Production)
    /// - `orchestration`: 8000-8099
    /// - `discovery`: 8100-8199
    /// - `messaging`: 8200-8299
    /// - `storage`: 8300-8399
    /// - `compute`: 8400-8499
    /// - `security`: 8500-8599
    /// - `monitoring`: 8600-8699
    /// - `federation`: 8700-8799
    /// - `other`: 8800-8899
    ///
    /// # Examples
    /// ```
    /// use songbird_config::defaults::ports_evolved::PortAllocator;
    ///
    /// let allocator = PortAllocator::new();
    /// let range = allocator.capability_range("storage", 8000..9000);
    /// assert_eq!(range, 8300..8400);
    /// ```
    #[must_use]
    pub fn capability_range(&self, capability: &str, _default_range: Range<u16>) -> Range<u16> {
        match capability {
            "orchestration" => 8000..8100,
            "discovery" => 8100..8200,
            "messaging" => 8200..8300,
            "storage" => 8300..8400,
            "compute" => 8400..8500,
            "security" => 8500..8600,
            "monitoring" => 8600..8700,
            "federation" => 8700..8800,
            _ => 8800..8900, // Default range for unknown capabilities
        }
    }

    /// Find an available port in the given range
    ///
    /// Iterates through the range to find a port that can be bound.
    ///
    /// # Errors
    /// Returns error if no ports in the range can be bound
    pub fn find_available_port_in_range(
        &self,
        range: Range<u16>,
    ) -> Result<TcpListener, std::io::Error> {
        for port in range {
            if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)) {
                return Ok(listener);
            }
        }

        Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "No available ports in range"))
    }

    /// Get port from environment variable or allocate dynamically
    ///
    /// # Errors
    /// Returns error if port allocation fails
    pub fn port_from_env_or_allocate(
        &self,
        env_var: &str,
        capability: &str,
    ) -> Result<TcpListener, std::io::Error> {
        // Try environment variable first
        if let Ok(port_str) = songbird_process_env::var(env_var)
            && let Ok(port) = port_str.parse::<u16>()
        {
            return TcpListener::bind(("0.0.0.0", port));
        }

        // Fall back to capability-based allocation
        self.allocate_for_capability(capability)
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Service port configuration (capability-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    /// Port number
    pub port: u16,

    /// Capability this port serves
    pub capability: String,

    /// Whether this port is dynamically allocated
    pub dynamic: bool,

    /// Port range if using capability-based allocation
    pub range: Option<Range<u16>>,
}

impl ServicePort {
    /// Create a new service port configuration
    #[must_use]
    pub fn new(port: u16, capability: impl Into<String>) -> Self {
        Self {
            port,
            capability: capability.into(),
            dynamic: false,
            range: None,
        }
    }

    /// Create a dynamically allocated port
    #[must_use]
    pub fn dynamic(capability: impl Into<String>) -> Self {
        Self {
            port: 0, // Will be assigned
            capability: capability.into(),
            dynamic: true,
            range: None,
        }
    }

    /// Create a capability-range port
    #[must_use]
    pub fn capability_range(capability: impl Into<String>, range: Range<u16>) -> Self {
        Self {
            port: range.start, // Default to start of range
            capability: capability.into(),
            dynamic: true,
            range: Some(range),
        }
    }
}

/// Well-known port registry (for backwards compatibility)
///
/// # Deprecation Notice
/// These functions are provided for backwards compatibility only.
/// New code should use `PortAllocator` with capability-based allocation.
pub mod well_known {
    /// Default orchestrator port (backwards compatibility)
    ///
    /// # Deprecated
    /// Use `PortAllocator::allocate_for_capability("orchestration")` instead
    #[deprecated(since = "0.2.0", note = "Use PortAllocator with capability-based allocation")]
    #[must_use]
    pub const fn orchestrator() -> u16 {
        8080
    }

    /// Default discovery port (backwards compatibility)
    #[deprecated(since = "0.2.0", note = "Use PortAllocator with capability-based allocation")]
    #[must_use]
    pub const fn discovery() -> u16 {
        8081
    }

    /// Default dashboard port (backwards compatibility)
    #[deprecated(since = "0.2.0", note = "Use PortAllocator with capability-based allocation")]
    #[must_use]
    pub const fn dashboard() -> u16 {
        3000
    }

    /// Default metrics port (backwards compatibility)
    #[deprecated(since = "0.2.0", note = "Use PortAllocator with capability-based allocation")]
    #[must_use]
    pub const fn metrics() -> u16 {
        9090
    }

    /// Default gaming port (backwards compatibility)
    #[deprecated(since = "0.2.0", note = "Use PortAllocator with capability-based allocation")]
    #[must_use]
    pub const fn gaming() -> u16 {
        7777
    }

    /// Default websocket port (backwards compatibility)
    #[deprecated(since = "0.2.0", note = "Use PortAllocator with capability-based allocation")]
    #[must_use]
    pub const fn websocket() -> u16 {
        8082
    }

    /// Default security port (backwards compatibility)
    #[deprecated(since = "0.2.0", note = "Use PortAllocator with capability-based allocation")]
    #[must_use]
    pub const fn security() -> u16 {
        8444
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_allocator_creation() {
        let allocator = PortAllocator::new();
        // Environment field is now private, test behavior instead
        // Strategy should be OsAssigned for dev/test or CapabilityRange for prod/staging
        assert!(matches!(
            allocator.strategy,
            PortStrategy::OsAssigned | PortStrategy::CapabilityRange { .. }
        ));
    }

    #[test]
    fn test_capability_ranges() {
        let allocator = PortAllocator::new();

        let orchestration_range = allocator.capability_range("orchestration", 8000..9000);
        assert_eq!(orchestration_range, 8000..8100);

        let discovery_range = allocator.capability_range("discovery", 8000..9000);
        assert_eq!(discovery_range, 8100..8200);

        let unknown_range = allocator.capability_range("unknown", 8000..9000);
        assert_eq!(unknown_range, 8800..8900);
    }

    #[test]
    fn test_os_assigned_allocation() {
        let mut allocator = PortAllocator::new();
        allocator.strategy = PortStrategy::OsAssigned;

        // Should successfully allocate a port
        let listener = allocator.allocate_for_capability("test").expect("Should allocate port");
        let addr = listener.local_addr().expect("Should have address");

        // OS-assigned port should be non-zero
        assert!(addr.port() > 0);
    }

    #[test]
    fn test_service_port_creation() {
        let port = ServicePort::new(8080, "orchestration");
        assert_eq!(port.port, 8080);
        assert_eq!(port.capability, "orchestration");
        assert!(!port.dynamic);
    }

    #[test]
    fn test_dynamic_service_port() {
        let port = ServicePort::dynamic("discovery");
        assert_eq!(port.port, 0);
        assert_eq!(port.capability, "discovery");
        assert!(port.dynamic);
    }

    #[test]
    fn test_capability_range_port() {
        let port = ServicePort::capability_range("storage", 8300..8400);
        assert_eq!(port.port, 8300);
        assert_eq!(port.capability, "storage");
        assert!(port.dynamic);
        assert_eq!(port.range, Some(8300..8400));
    }

    #[test]
    #[allow(deprecated, reason = "calling deprecated API until migration completes")]
    fn test_well_known_ports_backwards_compat() {
        assert_eq!(well_known::orchestrator(), 8080);
        assert_eq!(well_known::discovery(), 8081);
        assert_eq!(well_known::dashboard(), 3000);
        assert_eq!(well_known::metrics(), 9090);
    }
}
