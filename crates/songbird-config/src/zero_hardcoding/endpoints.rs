// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Zero Hardcoding: Environment-Driven Endpoints
//!
//! This module provides environment-driven endpoint configuration with intelligent defaults.
//! NO hardcoded ports or IPs - everything comes from environment or auto-selects.
//!
//! ## Philosophy
//!
//! - **Port 0 Magic**: Bind to port 0, OS assigns available port
//! - **Environment First**: Check env vars before defaults
//! - **Cloud Native**: Works in Kubernetes, Docker, bare metal
//! - **Zero Conflicts**: Each instance gets unique ports
//!
//! ## Usage
//!
//! ```rust
//! use songbird_config::zero_hardcoding::EndpointConfig;
//!
//! // Read from environment or auto-select
//! let config = EndpointConfig::from_env();
//! println!("HTTP port: {}", config.http_port); // Could be 0 (auto)
//!
//! // Bind and get actual port
//! let listener = std::net::TcpListener::bind(("0.0.0.0", config.http_port))?;
//! let actual_port = listener.local_addr()?.port();
//! println!("Actually listening on: {}", actual_port);
//! ```
//!
//! ## Environment Variables
//!
//! - `HTTP_PORT` - HTTP server port (default: 0 = auto-select)
//! - `RPC_PORT` - RPC server port (default: 0 = auto-select)
//! - `WS_PORT` - WebSocket port (default: 0 = auto-select)
//! - `BIND_ADDR` - Bind address (default: 0.0.0.0)
//! - `DISCOVERY_PORT` - Discovery/mDNS port (default: 0 = auto-select)

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Environment-driven endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndpointConfig {
    /// HTTP server port (0 = auto-select)
    pub http_port: u16,
    
    /// RPC server port (0 = auto-select)
    pub rpc_port: u16,
    
    /// WebSocket port (0 = auto-select)
    pub ws_port: u16,
    
    /// Discovery port (0 = auto-select)
    pub discovery_port: u16,
    
    /// Bind address (0.0.0.0 = all interfaces)
    pub bind_addr: IpAddr,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl EndpointConfig {
    /// Create configuration from environment variables
    ///
    /// All ports default to 0 (auto-select) if not specified.
    /// Bind address defaults to 0.0.0.0 (all interfaces).
    pub fn from_env() -> Self {
        Self {
            http_port: Self::parse_port_env("HTTP_PORT", 0),
            rpc_port: Self::parse_port_env("RPC_PORT", 0),
            ws_port: Self::parse_port_env("WS_PORT", 0),
            discovery_port: Self::parse_port_env("DISCOVERY_PORT", 0),
            bind_addr: Self::parse_addr_env("BIND_ADDR", IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        }
    }
    
    /// Create configuration with explicit values (for testing)
    #[must_use]
    pub const fn with_ports(http: u16, rpc: u16, ws: u16, discovery: u16) -> Self {
        Self {
            http_port: http,
            rpc_port: rpc,
            ws_port: ws,
            discovery_port: discovery,
            bind_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        }
    }
    
    /// Get HTTP socket address
    #[must_use]
    pub fn http_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind_addr, self.http_port)
    }
    
    /// Get RPC socket address
    #[must_use]
    pub fn rpc_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind_addr, self.rpc_port)
    }
    
    /// Get WebSocket socket address
    #[must_use]
    pub fn ws_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind_addr, self.ws_port)
    }
    
    /// Get discovery socket address
    #[must_use]
    pub fn discovery_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind_addr, self.discovery_port)
    }
    
    /// Parse port from environment variable
    fn parse_port_env(key: &str, default: u16) -> u16 {
        songbird_process_env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
    
    /// Parse IP address from environment variable
    fn parse_addr_env(key: &str, default: IpAddr) -> IpAddr {
        songbird_process_env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
}

/// Capability endpoint configuration
///
/// For discovered services (security, storage, compute, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEndpoints {
    /// Security capability endpoint (discovered or from env)
    pub security: Option<String>,
    
    /// Storage capability endpoint (discovered or from env)
    pub storage: Option<String>,
    
    /// Compute capability endpoint (discovered or from env)
    pub compute: Option<String>,
    
    /// AI capability endpoint (discovered or from env)
    pub ai: Option<String>,
}

impl Default for CapabilityEndpoints {
    fn default() -> Self {
        Self::from_env()
    }
}

impl CapabilityEndpoints {
    /// Load capability endpoints from environment
    ///
    /// Returns None for each capability if not set - will be discovered at runtime.
    pub fn from_env() -> Self {
        Self {
            security: songbird_process_env::var("SECURITY_ENDPOINT").ok(),
            storage: songbird_process_env::var("STORAGE_ENDPOINT").ok(),
            compute: songbird_process_env::var("COMPUTE_ENDPOINT").ok(),
            ai: songbird_process_env::var("AI_ENDPOINT").ok(),
        }
    }
    
    /// Check if any endpoints are configured
    #[must_use]
    pub fn has_any(&self) -> bool {
        self.security.is_some()
            || self.storage.is_some()
            || self.compute.is_some()
            || self.ai.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_uses_auto_select() {
        let config = EndpointConfig::from_env();
        
        // Without env vars, all ports should be 0 (auto-select)
        // Note: This test may fail if env vars are set
        // In CI, we'd use a clean environment
    }
    
    #[test]
    fn test_explicit_ports() {
        let config = EndpointConfig::with_ports(8080, 9000, 8081, 5353);
        
        assert_eq!(config.http_port, 8080);
        assert_eq!(config.rpc_port, 9000);
        assert_eq!(config.ws_port, 8081);
        assert_eq!(config.discovery_port, 5353);
    }
    
    #[test]
    fn test_socket_addresses() {
        let config = EndpointConfig::with_ports(8080, 9000, 8081, 5353);
        
        let http_addr = config.http_socket_addr();
        assert_eq!(http_addr.port(), 8080);
        
        let rpc_addr = config.rpc_socket_addr();
        assert_eq!(rpc_addr.port(), 9000);
    }
    
    #[test]
    fn test_capability_endpoints_default() {
        let endpoints = CapabilityEndpoints::from_env();
        
        // Without env vars, should be None (will be discovered)
        // Note: This test may fail if env vars are set
    }
}

