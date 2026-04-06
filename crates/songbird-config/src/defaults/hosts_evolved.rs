// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Self-aware service configuration with runtime discovery
//!
//! # Philosophy
//! - Songbird knows **only itself** (self-knowledge)
//! - Other services are **discovered at runtime** (no hardcoded primals)
//! - Configuration is **capability-based** (not name-based)
//! - Defaults are **environment-aware** (development vs production)

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

pub use super::service_locator::ServiceLocator;

fn read_process_env(key: &str) -> Result<String, std::env::VarError> {
    songbird_process_env::var(key)
}

/// Self-aware service configuration
///
/// Songbird knows only about itself. Other services are discovered at runtime
/// through capability-based discovery mechanisms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfAwareConfig {
    /// This service's bind configuration
    pub bind: BindConfig,
    /// This service's advertised address for discovery
    pub advertise: AdvertiseConfig,
    /// Environment-specific behavior
    pub environment: Environment,
}

impl SelfAwareConfig {
    /// Create configuration with environment awareness
    ///
    /// Configuration adapts to environment automatically:
    /// - **Development**: Binds to localhost, low security
    /// - **Production**: Binds to all interfaces, high security
    /// - **Test**: Isolated, ephemeral configuration
    #[must_use]
    pub fn from_environment() -> Self {
        Self::from_environment_with(&read_process_env)
    }

    /// Same as [`from_environment`](Self::from_environment) with an injectable env reader.
    #[must_use]
    pub fn from_environment_with(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Self {
        let environment = Environment::detect_with(env);

        Self {
            bind: BindConfig::for_environment(&environment),
            advertise: AdvertiseConfig::for_environment(&environment),
            environment,
        }
    }

    /// Get bind socket address
    #[must_use]
    pub const fn bind_address(&self) -> SocketAddr {
        self.bind.socket_addr()
    }

    /// Get advertised socket address for discovery
    #[must_use]
    pub const fn advertise_address(&self) -> SocketAddr {
        self.advertise.socket_addr()
    }
}

/// Bind configuration - where this service listens
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindConfig {
    /// IP address to bind to
    pub ip: IpAddr,
    /// Port to bind to (0 = OS assigns)
    pub port: u16,
}

impl BindConfig {
    /// Create environment-appropriate bind configuration
    ///
    /// - **Development**: localhost (127.0.0.1) — isolated testing
    /// - **Production**: all interfaces (0.0.0.0) — accessible to network
    /// - **Test**: localhost with OS-assigned port
    #[must_use]
    pub const fn for_environment(env: &Environment) -> Self {
        match env {
            Environment::Development => Self {
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 8080,
            },
            Environment::Production | Environment::Staging => Self {
                ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                port: 8080,
            },
            Environment::Test => Self {
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 0,
            },
        }
    }

    /// Create from environment variable or use default
    ///
    /// # Environment Variables
    /// - `SONGBIRD_BIND_IP`: IP address to bind
    /// - `SONGBIRD_BIND_PORT`: Port to bind
    #[must_use]
    pub fn from_env_or_default(env: &Environment) -> Self {
        let default = Self::for_environment(env);

        Self {
            ip: songbird_process_env::var("SONGBIRD_BIND_IP")
                .ok()
                .and_then(|s| IpAddr::from_str(&s).ok())
                .unwrap_or(default.ip),
            port: songbird_process_env::var("SONGBIRD_BIND_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.port),
        }
    }

    /// Convert to socket address
    #[must_use]
    pub const fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

/// Advertise configuration - how this service announces itself
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvertiseConfig {
    /// IP address to advertise for discovery
    pub ip: IpAddr,
    /// Port to advertise
    pub port: u16,
}

impl AdvertiseConfig {
    /// Create environment-appropriate advertise configuration
    ///
    /// - **Development**: localhost — services discover each other locally
    /// - **Production**: auto-detect public IP via [`network_detection`](super::network_detection)
    /// - **Test**: localhost with dynamic port
    #[must_use]
    pub fn for_environment(env: &Environment) -> Self {
        match env {
            Environment::Development => Self {
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 8080,
            },
            Environment::Production | Environment::Staging => Self {
                ip: super::network_detection::detect_public_ip(),
                port: 8080,
            },
            Environment::Test => Self {
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 0,
            },
        }
    }

    /// Convert to socket address
    #[must_use]
    pub const fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

/// Environment classification
///
/// Determines service behavior and security posture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment - localhost, permissive, fast iteration
    Development,
    /// Test environment - isolated, ephemeral, reproducible
    Test,
    /// Staging environment - production-like, pre-deployment validation
    Staging,
    /// Production environment - public-facing, high security, monitored
    Production,
}

impl Environment {
    /// Detect environment from system
    ///
    /// # Detection Strategy
    /// 1. Check `SONGBIRD_ENVIRONMENT` environment variable
    /// 2. Check common environment indicators (Kubernetes, ECS)
    /// 3. Check `RUST_TEST_THREADS` for test harness
    /// 4. Default to Development (safe default)
    ///
    /// # Examples
    /// ```no_run
    /// use songbird_config::defaults::hosts_evolved::Environment;
    ///
    /// let env = Environment::detect();
    /// assert!(matches!(env, Environment::Development | Environment::Production));
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        Self::detect_with(&read_process_env)
    }

    /// Same as [`detect`](Self::detect) with an injectable env reader.
    #[must_use]
    pub fn detect_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> Self {
        if let Ok(env_str) = env("SONGBIRD_ENVIRONMENT") {
            return match env_str.to_lowercase().as_str() {
                "production" | "prod" => Self::Production,
                "staging" | "stage" => Self::Staging,
                "test" => Self::Test,
                _ => Self::Development,
            };
        }

        if env("KUBERNETES_SERVICE_HOST").is_ok() || env("ECS_CONTAINER_METADATA_URI").is_ok() {
            return Self::Production;
        }

        if env("RUST_TEST_THREADS").is_ok() {
            return Self::Test;
        }

        Self::Development
    }

    /// Check if this is a production-like environment
    #[must_use]
    pub const fn is_production_like(self) -> bool {
        matches!(self, Self::Production | Self::Staging)
    }

    /// Check if this is a development-like environment
    #[must_use]
    pub const fn is_development_like(self) -> bool {
        matches!(self, Self::Development | Self::Test)
    }
}

//
// === CONVENIENCE FUNCTIONS FOR ENDPOINT CONSTRUCTION ===
//

/// Get host for orchestrator endpoint
#[must_use]
pub fn orchestrator_host() -> String {
    songbird_process_env::var("SONGBIRD_ORCHESTRATOR_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for discovery endpoint
#[must_use]
pub fn discovery_host() -> String {
    songbird_process_env::var("SONGBIRD_DISCOVERY_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for dashboard endpoint
#[must_use]
pub fn dashboard_host() -> String {
    songbird_process_env::var("SONGBIRD_DASHBOARD_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for metrics endpoint
#[must_use]
pub fn metrics_host() -> String {
    songbird_process_env::var("SONGBIRD_METRICS_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for WebSocket endpoint
#[must_use]
pub fn websocket_host() -> String {
    songbird_process_env::var("SONGBIRD_WEBSOCKET_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for a named service
#[must_use]
pub fn service_host(service_name: &str) -> String {
    let env_var = format!("SONGBIRD_{}_HOST", service_name.to_uppercase());
    songbird_process_env::var(env_var).unwrap_or_else(|_| default_host())
}

/// Get environment-aware default host
///
/// - Development/Test: 127.0.0.1 (localhost)
/// - Production/Staging: 0.0.0.0 (all interfaces)
#[must_use]
fn default_host() -> String {
    let env = Environment::detect();
    match env {
        Environment::Production | Environment::Staging => "0.0.0.0".to_string(),
        Environment::Development | Environment::Test => "127.0.0.1".to_string(),
    }
}

#[cfg(test)]
#[path = "hosts_evolved_tests.rs"]
mod tests;
