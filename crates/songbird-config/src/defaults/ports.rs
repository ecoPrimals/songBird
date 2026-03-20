// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Default port configuration with environment variable support
//!
//! # Environment Variables
//!
//! - `SONGBIRD_ORCHESTRATOR_PORT` - Orchestrator service port (default: 8080)
//! - `SONGBIRD_DISCOVERY_PORT` - Discovery service port (default: 8081)
//! - `SONGBIRD_DASHBOARD_PORT` - Dashboard UI port (default: 3000)
//! - `SONGBIRD_METRICS_PORT` - Metrics/observability port (default: 9090)
//! - `SONGBIRD_FEDERATION_PORT` - Federation coordination port (default: 8082)
//! - `SONGBIRD_WEBSOCKET_PORT` - WebSocket streaming port (default: 8080)

use std::env;

/// Get orchestrator service port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_ORCHESTRATOR_PORT` (default: 8080)
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::ports::orchestrator_port;
///
/// let port = orchestrator_port();
/// assert_eq!(port, 8080); // Or value from SONGBIRD_ORCHESTRATOR_PORT
/// ```
#[must_use]
pub fn orchestrator_port() -> u16 {
    env::var("SONGBIRD_ORCHESTRATOR_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080)
}

/// Get discovery service port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_DISCOVERY_PORT` (default: 8081)
#[must_use]
pub fn discovery_port() -> u16 {
    env::var("SONGBIRD_DISCOVERY_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8081)
}

/// Get dashboard UI port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_DASHBOARD_PORT` (default: 3000)
#[must_use]
pub fn dashboard_port() -> u16 {
    env::var("SONGBIRD_DASHBOARD_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000)
}

/// Get metrics/observability port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_METRICS_PORT` (default: 9090)
#[must_use]
pub fn metrics_port() -> u16 {
    env::var("SONGBIRD_METRICS_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(9090)
}

/// Get federation coordination port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_FEDERATION_PORT` (default: 8082)
#[must_use]
pub fn federation_port() -> u16 {
    env::var("SONGBIRD_FEDERATION_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8082)
}

/// Get WebSocket streaming port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_WEBSOCKET_PORT` (default: 8080)
#[must_use]
pub fn websocket_port() -> u16 {
    env::var("SONGBIRD_WEBSOCKET_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080)
}

/// Get gaming server port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_GAMING_PORT` (default: 6112 - `StarCraft` IPX)
#[must_use]
pub fn gaming_port() -> u16 {
    env::var("SONGBIRD_GAMING_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(6112)
}

/// Get health monitoring port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_HEALTH_PORT` (default: 8002)
#[must_use]
pub fn health_port() -> u16 {
    env::var("SONGBIRD_HEALTH_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8002)
}

/// Get `BearDog` security service port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_BEARDOG_PORT` (default: 8443)
#[must_use]
pub fn beardog_port() -> u16 {
    env::var("SONGBIRD_BEARDOG_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8443)
}

/// Get Toadstool load balancing service port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_TOADSTOOL_PORT` (default: 8001)
#[must_use]
pub fn toadstool_port() -> u16 {
    env::var("SONGBIRD_TOADSTOOL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8001)
}

/// Get Squirrel storage service port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_SQUIRREL_PORT` (default: 8002)
#[must_use]
pub fn squirrel_port() -> u16 {
    env::var("SONGBIRD_SQUIRREL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8002)
}

/// Get `NestGate` gateway service port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_NESTGATE_PORT` (default: 8003)
#[must_use]
pub fn nestgate_port() -> u16 {
    env::var("SONGBIRD_NESTGATE_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8003)
}

/// Get gaming port range start from environment or default
///
/// # Environment Variable
/// `SONGBIRD_GAMING_PORT_START` (default: 7000)
#[must_use]
pub fn gaming_port_range_start() -> u16 {
    env::var("SONGBIRD_GAMING_PORT_START").ok().and_then(|p| p.parse().ok()).unwrap_or(7000)
}

/// Get gaming port range end from environment or default
///
/// # Environment Variable
/// `SONGBIRD_GAMING_PORT_END` (default: 7100)
#[must_use]
pub fn gaming_port_range_end() -> u16 {
    env::var("SONGBIRD_GAMING_PORT_END").ok().and_then(|p| p.parse().ok()).unwrap_or(7100)
}

/// Get `StarCraft` specific port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_STARCRAFT_PORT` (default: 6112)
#[must_use]
pub fn starcraft_port() -> u16 {
    env::var("SONGBIRD_STARCRAFT_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(6112)
}

/// Get Age of Empires 2 port from environment or default
///
/// # Environment Variable
/// `SONGBIRD_AOE2_PORT` (default: 2300)
#[must_use]
pub fn aoe2_port() -> u16 {
    env::var("SONGBIRD_AOE2_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(2300)
}

/// Get Command & Conquer port range start from environment or default
///
/// # Environment Variable
/// `SONGBIRD_CNC_PORT_START` (default: 1234)
#[must_use]
pub fn cnc_port_range_start() -> u16 {
    env::var("SONGBIRD_CNC_PORT_START").ok().and_then(|p| p.parse().ok()).unwrap_or(1234)
}

/// Get Command & Conquer port range end from environment or default
///
/// # Environment Variable
/// `SONGBIRD_CNC_PORT_END` (default: 1240)
#[must_use]
pub fn cnc_port_range_end() -> u16 {
    env::var("SONGBIRD_CNC_PORT_END").ok().and_then(|p| p.parse().ok()).unwrap_or(1240)
}

/// Get tarpc high-performance RPC port from environment or default
///
/// tarpc provides binary RPC with ~50μs latency (100x faster than JSON-RPC!)
/// for native Rust client-to-server communication.
///
/// # Environment Variable
/// `SONGBIRD_TARPC_PORT` (default: 8091)
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::ports::tarpc_port;
///
/// let port = tarpc_port();
/// assert_eq!(port, 8091); // Or value from SONGBIRD_TARPC_PORT
/// ```
#[must_use]
pub fn tarpc_port() -> u16 {
    env::var("SONGBIRD_TARPC_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8091)
}

/// Get service port by name from environment or default
///
/// Supports dynamic port lookup for any service.
///
/// # Environment Variable Pattern
/// `SONGBIRD_{SERVICE}_PORT` where SERVICE is uppercase service name
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::ports::service_port;
///
/// let port = service_port("CUSTOM_SERVICE", 9000);
/// ```
#[must_use]
pub fn service_port(service_name: &str, default: u16) -> u16 {
    let env_var = format!("SONGBIRD_{}_PORT", service_name.to_uppercase());
    env::var(env_var).ok().and_then(|p| p.parse().ok()).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ports() {
        // Should return defaults when env vars not set
        assert_eq!(orchestrator_port(), 8080);
        assert_eq!(discovery_port(), 8081);
        assert_eq!(dashboard_port(), 3000);
        assert_eq!(metrics_port(), 9090);
        assert_eq!(federation_port(), 8082);
        assert_eq!(websocket_port(), 8080);
    }

    #[test]
    fn test_service_port() {
        let port = service_port("CUSTOM", 5000);
        assert_eq!(port, 5000); // Default when env var not set
    }
}
