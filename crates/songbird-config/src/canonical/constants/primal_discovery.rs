// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Primal endpoint discovery and capability-based filtering.
//!
//! All primal resolution is identity-agnostic: we discover by capability,
//! not by hardcoded primal names.  Environment variables drive every lookup
//! via injectable readers so that tests never mutate the process environment.

use std::collections::HashMap;

use songbird_types::error_helpers::SafeEnv;

use super::{
    FALLBACK_CANONICAL_DISCOVERY_PORT, FALLBACK_CANONICAL_GAMING_PORT,
    FALLBACK_CANONICAL_ORCHESTRATOR_PORT, FALLBACK_CANONICAL_SECURITY_PORT,
    FALLBACK_PRODUCTION_HTTPS_PORT, FALLBACK_STAGING_HTTP_PORT, env_get_bool_with,
    get_bind_address, get_bind_address_with, get_port_range_start, read_process_env,
};

// ==================== PRIMAL CONFIGURATION ====================

/// Universal primal endpoint discovery - works with any primal name
#[must_use]
pub fn get_primal_endpoint(primal_name: &str) -> String {
    get_primal_endpoint_with(primal_name, &read_process_env)
}

/// Same as [`get_primal_endpoint`] with an injectable env reader.
#[must_use]
pub fn get_primal_endpoint_with(
    primal_name: &str,
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> String {
    let env_var = format!("{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = env(&env_var) {
        return endpoint;
    }

    let generic_env = format!("PRIMAL_{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = env(&generic_env) {
        return endpoint;
    }

    calculate_default_primal_endpoint(primal_name)
}

fn calculate_default_primal_endpoint(primal_name: &str) -> String {
    let base_port = get_port_range_start();
    let primal_offset = calculate_primal_port_offset(primal_name);
    let port = base_port + primal_offset;

    let host = if SafeEnv::get("KUBERNETES_SERVICE_HOST").is_ok() {
        format!("{}-service", primal_name.to_lowercase())
    } else if SafeEnv::get("DOCKER_HOST").is_ok() || SafeEnv::get("CONTAINER").is_ok() {
        primal_name.to_lowercase()
    } else {
        get_bind_address()
    };

    let protocol = if should_use_tls_for_primal(primal_name) {
        "https"
    } else {
        "http"
    };

    format!("{protocol}://{host}:{port}")
}

pub(crate) fn calculate_primal_port_offset(primal_name: &str) -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    primal_name.hash(&mut hasher);
    let hash = hasher.finish();

    (hash % 1000) as u16
}

fn get_primal_port_offset(primal_type: &str) -> u16 {
    calculate_primal_port_offset(primal_type)
}

fn should_use_tls_for_primal(primal_name: &str) -> bool {
    should_use_tls_for_primal_with(primal_name, &read_process_env)
}

fn should_use_tls_for_primal_with(
    primal_name: &str,
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> bool {
    let tls_env = format!("{}_USE_TLS", primal_name.to_uppercase());
    if env_get_bool_with(env, &tls_env, false) {
        return true;
    }

    let is_security_primal = primal_name.to_lowercase().contains("security")
        || primal_name.to_lowercase().contains("auth")
        || primal_name.to_lowercase().contains("crypto");

    match env("SONGBIRD_ENV").as_deref() {
        Ok("production") => true,
        Ok("staging") => is_security_primal,
        _ => false,
    }
}

/// Get all configured primal names from a snapshot of environment variables.
#[must_use]
pub fn get_configured_primal_names_in_env(env: &HashMap<String, String>) -> Vec<String> {
    let mut primal_names = Vec::new();

    for key in env.keys() {
        if key.ends_with("_ENDPOINT") && !key.starts_with("SONGBIRD_") {
            let primal_name = key.trim_end_matches("_ENDPOINT").to_lowercase();
            if !primal_names.contains(&primal_name) {
                primal_names.push(primal_name);
            }
        }

        if key.starts_with("PRIMAL_")
            && key.ends_with("_ENDPOINT")
            && let Some(primal_part) =
                key.strip_prefix("PRIMAL_").and_then(|s| s.strip_suffix("_ENDPOINT"))
        {
            let primal_name = primal_part.to_lowercase();
            if !primal_names.contains(&primal_name) {
                primal_names.push(primal_name);
            }
        }
    }

    primal_names
}

/// Get all configured primal names from environment
#[must_use]
pub fn get_configured_primal_names() -> Vec<String> {
    let map: HashMap<String, String> = songbird_process_env::vars().collect();
    get_configured_primal_names_in_env(&map)
}

/// Dynamically discovers enabled primals via `SONGBIRD_ENABLE_*` env vars
/// rather than hardcoding specific primal names.
#[must_use]
pub fn get_common_primal_ports() -> Vec<u16> {
    let map: HashMap<String, String> = songbird_process_env::vars().collect();
    get_common_primal_ports_from_env_map(&map)
}

/// Common primal ports from a snapshot of environment variables (concurrent-safe tests).
#[must_use]
pub fn get_common_primal_ports_from_env_map(env: &HashMap<String, String>) -> Vec<u16> {
    let default_computed = {
        let mut ports = Vec::new();
        let base_port = get_port_range_start();

        ports.push(base_port);

        for (key, value) in env {
            if let Some(primal_name) = key.strip_prefix("SONGBIRD_ENABLE_")
                && (value.eq_ignore_ascii_case("true") || value == "1")
            {
                let name = primal_name.to_lowercase();
                ports.push(base_port + get_primal_port_offset(&name));
            }
        }

        ports.into_iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
    };

    env.get("SONGBIRD_COMMON_PORTS")
        .cloned()
        .unwrap_or(default_computed)
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}

fn normalize_capability_env_key(capability: &str) -> String {
    capability.trim().to_lowercase().replace(['-', ' '], "_").to_uppercase()
}

fn normalize_capability_match_key(s: &str) -> String {
    s.trim().to_lowercase().replace(['-', ' '], "_")
}

fn trim_nonempty(s: &str) -> Option<&str> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

fn capability_env_for_primal_in(
    primal_lower: &str,
    env: &HashMap<String, String>,
) -> Option<String> {
    let u = primal_lower.to_uppercase();
    env.get(&format!("{u}_CAPABILITIES"))
        .cloned()
        .or_else(|| env.get(&format!("PRIMAL_{u}_CAPABILITIES")).cloned())
        .or_else(|| env.get(&format!("SONGBIRD_PRIMAL_{u}_CAPABILITIES")).cloned())
}

fn primal_declares_capability_in(
    primal_lower: &str,
    capability_query: &str,
    env: &HashMap<String, String>,
) -> bool {
    let Some(list) = capability_env_for_primal_in(primal_lower, env) else {
        return false;
    };
    let want = normalize_capability_match_key(capability_query);
    list.split(',').filter_map(trim_nonempty).map(normalize_capability_match_key).any(|c| c == want)
}

/// Filters by capability using an env snapshot (tests avoid mutating process environment).
#[must_use]
pub fn find_primals_with_capability_in_env(
    capability: &str,
    env: &HashMap<String, String>,
) -> Vec<String> {
    let key = normalize_capability_env_key(capability);
    let providers_key = format!("SONGBIRD_CAPABILITY_{key}_PROVIDERS");
    if let Some(raw) = env.get(&providers_key) {
        let providers: Vec<String> =
            raw.split(',').filter_map(trim_nonempty).map(str::to_lowercase).collect();
        if !providers.is_empty() {
            return providers;
        }
    }

    let mut out = Vec::new();
    for primal in get_configured_primal_names_in_env(env) {
        if primal_declares_capability_in(&primal, capability, env) && !out.contains(&primal) {
            out.push(primal);
        }
    }
    out
}

/// Filters by capability from the live process environment.
#[must_use]
pub fn find_primals_with_capability(capability: &str) -> Vec<String> {
    let map: HashMap<String, String> = songbird_process_env::vars().collect();
    find_primals_with_capability_in_env(capability, &map)
}

// ==================== ENDPOINT CONFIGURATION ====================

fn env_or_default_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env(key).unwrap_or_else(|_| default.into())
}

fn default_production_base_url_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> String {
    let host = get_bind_address_with(env);
    let port =
        super::env_port_with(env, "SONGBIRD_PRODUCTION_HTTPS_PORT", FALLBACK_PRODUCTION_HTTPS_PORT);
    format!("https://{host}:{port}")
}

fn default_staging_base_url_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> String {
    let host = get_bind_address_with(env);
    let port = super::env_port_with(env, "SONGBIRD_STAGING_HTTP_PORT", FALLBACK_STAGING_HTTP_PORT);
    format!("http://{host}:{port}")
}

/// Get canonical endpoint URL based on environment and service (injectable env reader).
#[must_use]
pub fn get_canonical_endpoint_with(
    service_name: &str,
    default_port: u16,
    env: impl Fn(&str) -> Result<String, std::env::VarError>,
) -> String {
    let base_url = match env_or_default_with(&env, "SONGBIRD_ENVIRONMENT", "development").as_str() {
        "production" | "prod" => {
            env_or_default_with(&env, "SONGBIRD_BASE_URL", default_production_base_url_with(&env))
        }
        "staging" => env_or_default_with(
            &env,
            "SONGBIRD_BASE_URL",
            env_or_default_with(
                &env,
                "SONGBIRD_STAGING_BASE_URL",
                default_staging_base_url_with(&env),
            ),
        ),
        _ => {
            let host = get_bind_address_with(&env);
            env_or_default_with(&env, "SONGBIRD_BASE_URL", format!("http://{host}:{default_port}"))
        }
    };

    env_or_default_with(
        &env,
        &format!("SONGBIRD_{}_ENDPOINT", service_name.to_uppercase()),
        base_url,
    )
}

/// Get canonical endpoint URL based on environment and service
#[must_use]
pub fn get_canonical_endpoint(service_name: &str, default_port: u16) -> String {
    get_canonical_endpoint_with(service_name, default_port, read_process_env)
}

/// Get canonical discovery endpoint
#[must_use]
pub fn get_canonical_discovery_endpoint() -> String {
    let default_port =
        SafeEnv::get_port("SONGBIRD_CANONICAL_DISCOVERY_PORT", FALLBACK_CANONICAL_DISCOVERY_PORT);
    get_canonical_endpoint("discovery", default_port)
}

/// Get canonical security endpoint
#[must_use]
pub fn get_canonical_security_endpoint() -> String {
    let default_port =
        SafeEnv::get_port("SONGBIRD_CANONICAL_SECURITY_PORT", FALLBACK_CANONICAL_SECURITY_PORT);
    get_canonical_endpoint("security", default_port)
}

/// Get canonical orchestrator endpoint
#[must_use]
pub fn get_canonical_orchestrator_endpoint() -> String {
    let default_port = SafeEnv::get_port(
        "SONGBIRD_CANONICAL_ORCHESTRATOR_PORT",
        FALLBACK_CANONICAL_ORCHESTRATOR_PORT,
    );
    get_canonical_endpoint("orchestrator", default_port)
}

/// Get canonical gaming endpoint
#[must_use]
pub fn get_canonical_gaming_endpoint() -> String {
    let default_port =
        SafeEnv::get_port("SONGBIRD_CANONICAL_GAMING_PORT", FALLBACK_CANONICAL_GAMING_PORT);
    get_canonical_endpoint("gaming", default_port)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use std::collections::HashMap;

    use super::super::get_port_range_start;
    use super::*;

    fn env_map<'a>(
        pairs: &'a [(&'a str, &'a str)],
    ) -> impl Fn(&str) -> Result<String, std::env::VarError> + 'a {
        move |k: &str| {
            pairs
                .iter()
                .find(|(a, _)| *a == k)
                .map(|(_, v)| (*v).to_string())
                .ok_or(std::env::VarError::NotPresent)
        }
    }

    #[test]
    fn test_get_primal_endpoint_with_primary_env_wins() {
        let ep = get_primal_endpoint_with(
            "ai_provider",
            &env_map(&[
                ("AI_PROVIDER_ENDPOINT", "http://explicit-ai-provider:1"),
                ("PRIMAL_AI_PROVIDER_ENDPOINT", "http://generic:2"),
            ]),
        );
        assert_eq!(ep, "http://explicit-ai-provider:1");
    }

    #[test]
    fn test_get_primal_endpoint_with_primal_prefix_fallback() {
        let ep = get_primal_endpoint_with(
            "nest",
            &env_map(&[("PRIMAL_NEST_ENDPOINT", "http://nest:3")]),
        );
        assert_eq!(ep, "http://nest:3");
    }

    #[test]
    fn test_calculate_primal_port_offset_stable_per_name() {
        let a = calculate_primal_port_offset("foo");
        let b = calculate_primal_port_offset("foo");
        let c = calculate_primal_port_offset("bar");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_get_configured_primal_names_in_env_collects_endpoints() {
        let mut m = HashMap::new();
        m.insert(String::from("SECURITY_PROVIDER_ENDPOINT"), String::from("http://x"));
        m.insert(String::from("PRIMAL_COMPUTE_PROVIDER_ENDPOINT"), String::from("http://y"));
        m.insert(String::from("SONGBIRD_IGNORED_ENDPOINT"), String::from("http://z"));
        let mut names = get_configured_primal_names_in_env(&m);
        names.sort();
        assert!(names.contains(&String::from("security_provider")));
        assert!(names.contains(&String::from("compute_provider")));
        assert!(!names.contains(&String::from("songbird_ignored")));
    }

    #[test]
    fn test_get_common_primal_ports_from_env_map_override() {
        let mut m = HashMap::new();
        m.insert(String::from("SONGBIRD_COMMON_PORTS"), String::from("1111,2222"));
        let ports = get_common_primal_ports_from_env_map(&m);
        assert_eq!(ports, vec![1111, 2222]);
    }

    #[test]
    fn test_get_common_primal_ports_computed_with_enable_flags() {
        let mut m = HashMap::new();
        m.insert(String::from("SONGBIRD_ENABLE_ALPHA"), String::from("true"));
        m.insert(String::from("SONGBIRD_ENABLE_BETA"), String::from("1"));
        m.insert(String::from("SONGBIRD_ENABLE_GAMMA"), String::from("false"));
        let ports = get_common_primal_ports_from_env_map(&m);
        assert!(ports.len() >= 2);
        assert_eq!(ports[0], get_port_range_start());
    }

    #[test]
    fn test_find_primals_capability_providers_override() {
        let mut m = HashMap::new();
        m.insert(
            String::from("SONGBIRD_CAPABILITY_STORAGE_PROVIDERS"),
            String::from("alpha, Beta "),
        );
        let out = find_primals_with_capability_in_env("storage", &m);
        assert_eq!(out, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_find_primals_from_declared_capabilities() {
        let mut m = HashMap::new();
        m.insert(String::from("FOO_CAPABILITIES"), String::from("embeddings, text-gen"));
        m.insert(String::from("FOO_ENDPOINT"), String::from("http://foo"));
        let out = find_primals_with_capability_in_env("embeddings", &m);
        assert_eq!(out, vec!["foo"]);
    }

    #[test]
    fn test_find_primals_alternate_capability_keys() {
        let mut m = HashMap::new();
        m.insert(String::from("PRIMAL_BAR_CAPABILITIES"), String::from("compute"));
        m.insert(String::from("BAR_ENDPOINT"), String::from("http://bar"));
        let out = find_primals_with_capability_in_env("compute", &m);
        assert_eq!(out, vec!["bar"]);
    }

    #[test]
    fn test_get_canonical_endpoint_development_default() {
        let ep = get_canonical_endpoint_with(
            "discovery",
            9000,
            env_map(&[
                ("SONGBIRD_ENVIRONMENT", "development"),
                ("SONGBIRD_BIND_ADDRESS", "127.0.0.1"),
            ]),
        );
        assert!(ep.starts_with("http://127.0.0.1:9000"));
    }

    #[test]
    fn test_get_canonical_endpoint_service_override() {
        let ep = get_canonical_endpoint_with(
            "discovery",
            9000,
            env_map(&[
                ("SONGBIRD_ENVIRONMENT", "development"),
                ("SONGBIRD_BIND_ADDRESS", "127.0.0.1"),
                ("SONGBIRD_DISCOVERY_ENDPOINT", "http://override:7777"),
            ]),
        );
        assert_eq!(ep, "http://override:7777");
    }

    #[test]
    fn test_get_canonical_endpoint_production_uses_base_url_chain() {
        let ep = get_canonical_endpoint_with(
            "security",
            443,
            env_map(&[
                ("SONGBIRD_ENVIRONMENT", "production"),
                ("SONGBIRD_BIND_ADDRESS", "10.0.0.2"),
                ("SONGBIRD_PRODUCTION_HTTPS_PORT", "9443"),
                ("SONGBIRD_BASE_URL", "https://edge.example:443"),
            ]),
        );
        assert_eq!(ep, "https://edge.example:443");
    }

    #[test]
    fn test_get_canonical_endpoint_staging_prefers_staging_base() {
        let ep = get_canonical_endpoint_with(
            "orchestrator",
            8080,
            env_map(&[
                ("SONGBIRD_ENVIRONMENT", "staging"),
                ("SONGBIRD_BIND_ADDRESS", "127.0.0.1"),
                ("SONGBIRD_STAGING_HTTP_PORT", "9080"),
                ("SONGBIRD_STAGING_BASE_URL", "http://staging.internal:9080"),
            ]),
        );
        assert_eq!(ep, "http://staging.internal:9080");
    }

    #[test]
    fn test_find_primals_empty_when_no_match() {
        let m = HashMap::new();
        let out = find_primals_with_capability_in_env("nonexistent-cap", &m);
        assert!(out.is_empty());
    }

    #[test]
    fn test_find_primals_capability_name_normalizes_dashes() {
        let mut m = HashMap::new();
        m.insert(String::from("SONGBIRD_CAPABILITY_TEXT_GEN_PROVIDERS"), String::from("p1"));
        let out = find_primals_with_capability_in_env("text-gen", &m);
        assert_eq!(out, vec!["p1"]);
    }

    #[test]
    fn test_get_common_primal_ports_empty_override_yields_empty_vec() {
        let mut m = HashMap::new();
        m.insert(String::from("SONGBIRD_COMMON_PORTS"), String::from("  ,  , "));
        let ports = get_common_primal_ports_from_env_map(&m);
        assert!(ports.is_empty());
    }
}
