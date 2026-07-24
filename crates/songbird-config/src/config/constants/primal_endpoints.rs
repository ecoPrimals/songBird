// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Primal endpoint resolution from env and deterministic defaults.

use super::bind_and_ports::get_port_range_start;
use songbird_types::error_helpers::SafeEnv;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Universal primal endpoint discovery - works with any primal name
#[deprecated(
    since = "0.2.1",
    note = "use capability-based discovery via CapabilityEndpointResolver instead"
)]
#[must_use]
#[allow(deprecated, reason = "legacy shim delegates to sibling helpers in deprecated module")]
pub fn get_primal_endpoint(primal_name: &str) -> String {
    tracing::warn!(
        primal_name = %primal_name,
        "get_primal_endpoint is deprecated; use capability-based discovery via CapabilityEndpointResolver instead"
    );

    // First try primal-specific environment variable
    let env_var = format!("{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = SafeEnv::get(&env_var) {
        return endpoint;
    }

    // Try generic primal endpoint pattern
    let generic_env = format!("PRIMAL_{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = SafeEnv::get(&generic_env) {
        return endpoint;
    }

    // Calculate default endpoint based on environment and primal name
    calculate_default_primal_endpoint(primal_name)
}

/// Calculate default endpoint for any primal based on naming conventions
fn calculate_default_primal_endpoint(primal_name: &str) -> String {
    let base_port = get_port_range_start();
    let primal_offset = calculate_primal_port_offset(primal_name);
    let port = base_port + primal_offset;

    let host = if SafeEnv::get("KUBERNETES_SERVICE_HOST").is_ok() {
        // Kubernetes service discovery pattern
        format!("{}-service", primal_name.to_lowercase())
    } else if SafeEnv::get("DOCKER_HOST").is_ok() || SafeEnv::get("CONTAINER").is_ok() {
        // Docker container pattern
        primal_name.to_lowercase()
    } else {
        // Local development pattern
        crate::constants::network::DEFAULT_HOST.to_string()
    };

    let protocol = if should_use_tls_for_primal(primal_name) {
        "https"
    } else {
        "http"
    };

    format!("{protocol}://{host}:{port}")
}

/// Calculate port offset for any primal name using consistent hashing
fn calculate_primal_port_offset(primal_name: &str) -> u16 {
    // Use consistent hashing to assign port offsets
    // This ensures the same primal name always gets the same offset
    let mut hasher = DefaultHasher::new();
    primal_name.hash(&mut hasher);
    let hash = hasher.finish();

    // Map hash to reasonable port offset (0-999)
    (hash % 1000) as u16
}

/// Determine if primal should use TLS based on environment and naming
fn should_use_tls_for_primal(primal_name: &str) -> bool {
    // Check primal-specific TLS setting
    let tls_env = format!("{}_USE_TLS", primal_name.to_uppercase());
    if SafeEnv::get_bool(&tls_env, false) {
        return true;
    }

    // Security-related primals default to TLS in production
    let is_security_primal = primal_name.to_lowercase().contains("security")
        || primal_name.to_lowercase().contains("auth")
        || primal_name.to_lowercase().contains("crypto");

    match SafeEnv::get("SONGBIRD_ENV").as_deref() {
        Ok("production") => true,
        Ok("staging") => is_security_primal,
        _ => false, // Development default
    }
}

/// Get all configured primal names from environment
#[must_use]
pub fn get_configured_primal_names() -> Vec<String> {
    let mut primal_names = Vec::new();

    // Scan for primal-specific environment variables (overlay + OS, matching [`SafeEnv`])
    for (key, _value) in songbird_process_env::vars() {
        if key.ends_with("_ENDPOINT") && !key.starts_with("SONGBIRD_") {
            let primal_name = key.trim_end_matches("_ENDPOINT").to_lowercase();
            if !primal_names.contains(&primal_name) {
                primal_names.push(primal_name);
            }
        }

        // Also check PRIMAL_*_ENDPOINT pattern
        if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
            // Safe: we already checked starts_with and ends_with above
            if let Some(primal_part) =
                key.strip_prefix("PRIMAL_").and_then(|s| s.strip_suffix("_ENDPOINT"))
            {
                let primal_name = primal_part.to_lowercase();
                if !primal_names.contains(&primal_name) {
                    primal_names.push(primal_name);
                }
            }
        }
    }

    // If no primals configured, return empty list for pure discovery mode
    primal_names
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    deprecated,
    reason = "test assertions; exercises deprecated name-based endpoint resolution"
)]
mod tests {
    use super::*;
    use songbird_test_utils::ScopedEnv;

    #[tokio::test]
    async fn get_primal_endpoint_prefers_primal_specific_env() {
        let _e = ScopedEnv::remove_and_set_many(
            [
                "PRIMAL_SBTESTPRIMALA_ENDPOINT",
                "KUBERNETES_SERVICE_HOST",
                "DOCKER_HOST",
                "CONTAINER",
            ],
            [("SBTESTPRIMALA_ENDPOINT", "http://explicit-one:1111")],
        )
        .await;
        assert_eq!(get_primal_endpoint("sbtestprimala"), "http://explicit-one:1111");
    }

    #[tokio::test]
    async fn get_primal_endpoint_falls_back_to_primal_underscore_pattern() {
        let _e = ScopedEnv::remove_and_set_many(
            ["SBTESTPRIMALB_ENDPOINT", "KUBERNETES_SERVICE_HOST", "DOCKER_HOST", "CONTAINER"],
            [("PRIMAL_SBTESTPRIMALB_ENDPOINT", "http://explicit-two:2222")],
        )
        .await;
        assert_eq!(get_primal_endpoint("sbtestprimalb"), "http://explicit-two:2222");
    }

    #[tokio::test]
    async fn get_primal_endpoint_default_is_stable_for_same_name() {
        let _e = ScopedEnv::remove_multiple([
            "SBTESTPRIMALSTABLE_ENDPOINT",
            "PRIMAL_SBTESTPRIMALSTABLE_ENDPOINT",
            "KUBERNETES_SERVICE_HOST",
            "DOCKER_HOST",
            "CONTAINER",
            "SONGBIRD_ENV",
        ])
        .await;
        let a = get_primal_endpoint("sbtestprimalstable");
        let b = get_primal_endpoint("sbtestprimalstable");
        assert_eq!(a, b);
        assert!(a.starts_with("http://"), "expected non-TLS dev default, got {a}");
        assert!(a.contains(':'));
    }

    #[tokio::test]
    async fn get_primal_endpoint_production_uses_https_scheme() {
        let _e = ScopedEnv::remove_and_set_many(
            [
                "SBTESTPRIMALPROD_ENDPOINT",
                "PRIMAL_SBTESTPRIMALPROD_ENDPOINT",
                "KUBERNETES_SERVICE_HOST",
                "DOCKER_HOST",
                "CONTAINER",
            ],
            [("SONGBIRD_ENV", "production")],
        )
        .await;
        let ep = get_primal_endpoint("sbtestprimalprod");
        assert!(ep.starts_with("https://"), "production defaults to TLS, got {ep}");
    }

    #[tokio::test]
    async fn get_configured_primal_names_picks_up_endpoint_env_keys() {
        let key = "SBTESTPRIMALNAMEDXYZ_ENDPOINT";
        let _e = ScopedEnv::remove_and_set_many([key], [(key, "http://unused:1")]).await;
        let names = get_configured_primal_names();
        assert!(
            names.contains(&String::from("sbtestprimalnamedxyz")),
            "expected scan to find {key}, got {names:?}"
        );
    }
}
