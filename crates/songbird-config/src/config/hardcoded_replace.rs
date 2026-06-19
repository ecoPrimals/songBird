// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Convenience functions for replacing hardcoded values in production code.
//!
//! Each function in this module wraps a field from [`super::get_config()`],
//! providing zero-config defaults that can be overridden via environment
//! variables. Use these instead of inline magic numbers/strings.

use super::{Duration, IpAddr, get_config};
use std::sync::Arc;

/// Replace hardcoded bind address.
#[must_use]
pub fn bind_address() -> IpAddr {
    get_config().network.bind_address
}

/// Replace hardcoded orchestrator endpoint.
#[must_use]
pub fn orchestrator_endpoint() -> Arc<str> {
    Arc::clone(&get_config().network.orchestrator_endpoint)
}

/// Replace hardcoded gaming endpoint.
#[must_use]
pub fn gaming_endpoint() -> Arc<str> {
    Arc::clone(&get_config().network.gaming_endpoint)
}

/// Replace hardcoded security provider endpoint.
#[must_use]
pub fn security_provider_endpoint() -> Arc<str> {
    Arc::clone(&get_config().primals.security_provider_endpoint)
}

/// Replace hardcoded storage provider endpoint.
#[must_use]
pub fn storage_provider_endpoint() -> Arc<str> {
    Arc::clone(&get_config().primals.storage_provider_endpoint)
}

/// Replace hardcoded connection timeout.
#[must_use]
pub fn connection_timeout() -> Duration {
    get_config().timeouts.connection_timeout
}

/// Replace hardcoded request timeout.
#[must_use]
pub fn request_timeout() -> Duration {
    get_config().timeouts.request_timeout
}

/// Replace hardcoded health check timeout.
#[must_use]
pub fn health_check_timeout() -> Duration {
    get_config().timeouts.health_check_timeout
}

/// Replace hardcoded buffer size.
#[must_use]
pub fn large_buffer_size() -> usize {
    get_config().performance.large_buffer_size
}

/// Replace hardcoded STUN servers.
#[must_use]
pub fn stun_servers() -> Vec<String> {
    get_config().network.stun_servers.clone()
}

/// Replace hardcoded federation endpoints.
#[must_use]
pub fn federation_endpoints() -> Vec<String> {
    get_config().federation.cluster_endpoints.clone()
}

/// Replace hardcoded compute capability endpoint.
#[must_use]
pub fn compute_provider_endpoint() -> Arc<str> {
    Arc::clone(&get_config().primals.compute_provider_endpoint)
}

/// Replace hardcoded AI / neural capability endpoint.
#[must_use]
pub fn ai_provider_endpoint() -> Arc<str> {
    Arc::clone(&get_config().primals.ai_provider_endpoint)
}

/// Replace hardcoded capability-discovery endpoint list.
#[must_use]
pub fn primal_discovery_endpoints() -> Vec<String> {
    get_config().primals.discovery_endpoints.clone()
}

/// Replace hardcoded broadcast ports.
#[must_use]
pub fn federation_broadcast_ports() -> Vec<u16> {
    get_config().federation.broadcast_ports.clone()
}

/// Replace hardcoded discovery ports.
#[must_use]
pub fn federation_discovery_ports() -> Vec<u16> {
    get_config().federation.discovery_ports.clone()
}

/// Get production-ready bind address (0.0.0.0 vs localhost).
#[must_use]
pub fn production_bind_address() -> IpAddr {
    if songbird_process_env::var("SONGBIRD_ENVIRONMENT").unwrap_or_default() == "production" {
        get_config().network.production_bind_address
    } else {
        get_config().network.bind_address
    }
}

/// Format endpoint with configurable IP and port.
///
/// Discovery order:
/// 1. `{CAPABILITY}_ENDPOINT` environment variable
/// 2. `{CAPABILITY}_PORT` environment variable + bind address
/// 3. Auto-select port (0) for dynamic allocation
#[must_use]
pub fn format_endpoint(capability: &str, port_override: Option<u16>) -> Arc<str> {
    let env_key_endpoint = format!("{}_ENDPOINT", capability.to_uppercase());
    if let Ok(endpoint) = songbird_process_env::var(&env_key_endpoint) {
        return Arc::from(endpoint);
    }

    let config = get_config();
    let ip =
        if songbird_process_env::var("SONGBIRD_ENVIRONMENT").unwrap_or_default() == "production" {
            config.network.production_bind_address
        } else {
            config.network.bind_address
        };

    let env_key_port = format!("{}_PORT", capability.to_uppercase());
    let port = port_override
        .or_else(|| songbird_process_env::var(&env_key_port).ok().and_then(|p| p.parse().ok()))
        .unwrap_or(0);

    let protocol = if port == songbird_types::defaults::ports::DEFAULT_HTTPS_PORT
        || capability == "security"
    {
        "https"
    } else {
        "http"
    };
    Arc::from(format!("{protocol}://{ip}:{port}"))
}

/// Format service endpoint with path.
#[must_use]
pub fn format_service_endpoint(service: &str, path: &str, port_override: Option<u16>) -> String {
    let base = format_endpoint(service, port_override);
    format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))
}

/// Replace hardcoded gaming port.
#[must_use]
pub fn gaming_port() -> u16 {
    get_config().network.gaming_port_range.start
}

/// Replace hardcoded timeout configuration.
#[must_use]
pub fn timeout_config() -> super::TimeoutConfig {
    get_config().timeouts.clone()
}
