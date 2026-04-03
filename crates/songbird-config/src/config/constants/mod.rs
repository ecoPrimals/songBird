// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration Constants and Defaults (DEPRECATED)
//!
//! ⚠️ **CONSOLIDATION COMPLETE - MIGRATION REQUIRED** (November 8, 2025)
//!
//! This module has **92+ active uses** and has been superseded by `canonical::constants`.
//! All constants and functions have been migrated to the canonical module with identical APIs.
//!
//! ## Migration Path
//! ```rust,ignore
//! // ❌ OLD (deprecated - ALL MIGRATED as of Nov 10, 2025):
//! use songbird_config::config::constants::get_bind_address;
//! use songbird_config::config::constants::network::DEFAULT_HOST;
//!
//! // ✅ NEW (all code now uses this):
//! use songbird_config::canonical::constants::get_bind_address;
//! use songbird_config::canonical::constants::network::DEFAULT_HOST;
//! ```
//!
//! ## Migration Complete ✅
//! - ✅ All 98 references migrated (November 10, 2025)
//! - ✅ Zero deprecation warnings
//! - ✅ Build passing
//! - ✅ Tests passing
//!
//! ## What Was Consolidated
//! - 740 duplicate lines eliminated
//! - Identical API maintained (drop-in replacement)
//! - Single source of truth established
//! - `network::*` submodule fully migrated
//!
//! **Status**: ✅ Migration complete - file kept for external backward compatibility\
//! **Timeline**: Can be removed in v0.3.0 (Q2 2026) once external uses confirmed migrated\
//! **Urgency**: NONE - All internal uses successfully migrated

#![deprecated(since = "0.2.0", note = "Use songbird_config::canonical::constants instead")]
#![allow(missing_docs, reason = "deprecated shim; canonical module holds the documented API")]

mod bind_and_ports;
mod connection_and_tuning;
mod network_extras;
mod paths;
mod primal_endpoints;

pub mod health;
pub mod network;
pub mod resources;
pub mod services;

pub use bind_and_ports::{
    DEFAULT_BIND_ADDRESS, DEFAULT_CONFIG_PATH, DEFAULT_LOCALHOST, LOCALHOST_IPV4, get_bind_address,
    get_common_primal_ports, get_port_range_end, get_port_range_start,
};
pub use connection_and_tuning::{
    DEFAULT_CACHE_TTL, DEFAULT_EVALUATION_TIMEOUT, DEFAULT_METRICS_INTERVAL, enable_zero_copy,
    get_batch_size, get_buffer_pool_size, get_connection_timeout_ms, get_log_level,
    get_max_connections, get_worker_threads,
};
pub use network_extras::{
    default_bind_address, default_discovery_port, default_subnet, external_address,
    find_primals_with_capability, get_dashboard_port, get_default_bind_address, node_id,
    protocol_port_mappings,
};
pub use paths::{get_cache_dir, get_config_dir, get_data_dir, get_log_dir, get_temp_dir};
pub use primal_endpoints::{get_configured_primal_names, get_primal_endpoint};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::{
        external_address, get_cache_dir, get_config_dir, get_data_dir, get_log_dir, get_temp_dir,
        health, protocol_port_mappings, resources, services,
    };

    #[test]
    fn test_get_bind_address() {
        let addr = super::get_bind_address();
        assert!(!addr.is_empty());
    }

    #[test]
    fn string_constants_are_documented_non_empty() {
        assert_eq!(super::LOCALHOST_IPV4, "127.0.0.1");
        assert!(!super::DEFAULT_CONFIG_PATH.is_empty());
        assert!(!super::DEFAULT_BIND_ADDRESS.is_empty());
    }

    #[test]
    fn protocol_port_mappings_includes_core_protocols() {
        let m = protocol_port_mappings();
        assert_eq!(m.get("udp"), Some(&6112));
        assert_eq!(m.get("tcp"), Some(&6113));
        assert_eq!(m.get("websocket"), Some(&8080));
        assert_eq!(m.get("secure_websocket"), Some(&8443));
    }

    #[test]
    fn health_module_default_intervals() {
        assert_eq!(health::DEFAULT_CHECK_INTERVAL.as_secs(), 30);
        assert_eq!(health::DEFAULT_CHECK_TIMEOUT.as_secs(), 5);
    }

    #[test]
    fn resources_module_defaults() {
        assert_eq!(resources::DEFAULT_CLEANUP_INTERVAL.as_secs(), 300);
        assert!((resources::DEFAULT_MAX_MEMORY_USAGE - 0.8).abs() < f64::EPSILON);
        assert!((resources::DEFAULT_MAX_CPU_USAGE - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn services_module_shutdown_and_startup() {
        assert_eq!(services::DEFAULT_SHUTDOWN_TIMEOUT.as_secs(), 30);
        assert_eq!(services::DEFAULT_STARTUP_TIMEOUT.as_secs(), 60);
    }

    #[test]
    fn default_subnet_and_external_address_are_non_empty() {
        assert!(!super::default_subnet().is_empty());
        assert!(!external_address().is_empty());
    }

    #[test]
    fn directory_helpers_return_non_empty_strings() {
        assert!(!get_log_dir().is_empty());
        assert!(!get_cache_dir().is_empty());
        assert!(!get_data_dir().is_empty());
        assert!(!get_config_dir().is_empty());
        assert!(!get_temp_dir().is_empty());
    }

    #[test]
    fn duration_constants_are_sensible() {
        assert_eq!(super::DEFAULT_CACHE_TTL.as_secs(), 300);
        assert_eq!(super::DEFAULT_EVALUATION_TIMEOUT.as_secs(), 30);
        assert_eq!(super::DEFAULT_METRICS_INTERVAL.as_secs(), 60);
    }

    #[test]
    fn network_submodule_host_constants() {
        assert_eq!(super::network::DEFAULT_HOST, "localhost");
        assert_eq!(super::network::DEFAULT_HOST_V4, "127.0.0.1");
        assert_eq!(super::network::PRODUCTION_BIND_ADDRESS, "0.0.0.0");
    }
}
