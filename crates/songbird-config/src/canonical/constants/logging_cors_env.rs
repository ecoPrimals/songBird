// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Log level, environment classification, and CORS policy helpers.

use songbird_types::error_helpers::SafeEnv;

use super::{env_or_default_with, read_process_env};

/// Get log level from environment or default
#[must_use]
pub fn get_log_level() -> String {
    get_log_level_with(&read_process_env)
}

/// Same as [`get_log_level`] with an injectable env reader.
#[must_use]
pub fn get_log_level_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env("SONGBIRD_LOG_LEVEL")
        .or_else(|_| env("LOG_LEVEL"))
        .or_else(|_| env("RUST_LOG"))
        .unwrap_or_else(|_| {
            match env("SONGBIRD_ENV").as_deref() {
                Ok("production") => "warn".to_string(),
                Ok("staging") => "info".to_string(),
                _ => "debug".to_string(), // Testing and development default
            }
        })
}

/// Check if running in development environment
#[must_use]
pub fn is_development_environment() -> bool {
    let env = SafeEnv::get_or_default("SONGBIRD_ENVIRONMENT", "development");
    env == "development" || env == "dev"
}

/// Check if running in production environment
#[must_use]
pub fn is_production_environment() -> bool {
    let env = SafeEnv::get_or_default("SONGBIRD_ENVIRONMENT", "development");
    env == "production" || env == "prod"
}

fn is_production_environment_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> bool {
    let e = env_or_default_with(env, "SONGBIRD_ENVIRONMENT", "development");
    e == "production" || e == "prod"
}

/// Get canonical CORS origins based on environment
///
/// SOVEREIGNTY: No hardcoded origins. All must be explicitly configured.
#[must_use]
pub fn get_canonical_cors_origins() -> Vec<String> {
    get_canonical_cors_origins_with(&read_process_env)
}

/// Same as [`get_canonical_cors_origins`] with an injectable env reader.
#[must_use]
pub fn get_canonical_cors_origins_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> Vec<String> {
    env("SONGBIRD_CORS_ORIGINS").map_or_else(
        |_| {
            if is_production_environment_with(env) {
                // Production: No defaults - fail secure
                tracing::warn!(
                    "SONGBIRD_CORS_ORIGINS not set in production. CORS will deny all origins. \
                     Set SONGBIRD_CORS_ORIGINS to comma-separated list of allowed origins."
                );
                Vec::new() // Empty = deny all (secure default)
            } else {
                // Development: Calculate from bind address
                let bind_addr = super::get_bind_address_with(env);
                let default_ports = [3000, 8080, 8081];

                default_ports
                    .iter()
                    .flat_map(|port| {
                        vec![
                            format!("http://{}:{}", bind_addr, port),
                            format!("http://localhost:{}", port),
                        ]
                    })
                    .collect()
            }
        },
        |origins| origins.split(',').map(|s| s.trim().to_string()).collect(),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_env<'a>(
        map: &'a HashMap<&str, &str>,
    ) -> impl Fn(&str) -> Result<String, std::env::VarError> + 'a {
        move |key| map.get(key).map(ToString::to_string).ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn get_log_level_with_uses_songbird_log_level() {
        let map = HashMap::from([("SONGBIRD_LOG_LEVEL", "trace")]);
        let env = mock_env(&map);
        assert_eq!(get_log_level_with(&env), "trace");
    }

    #[test]
    fn get_log_level_with_falls_back_to_log_level() {
        let map = HashMap::from([("LOG_LEVEL", "info")]);
        let env = mock_env(&map);
        assert_eq!(get_log_level_with(&env), "info");
    }

    #[test]
    fn get_log_level_with_falls_back_to_rust_log() {
        let map = HashMap::from([("RUST_LOG", "debug")]);
        let env = mock_env(&map);
        assert_eq!(get_log_level_with(&env), "debug");
    }

    #[test]
    fn get_log_level_with_prefers_songbird_over_log_level_and_rust_log() {
        let map = HashMap::from([
            ("SONGBIRD_LOG_LEVEL", "error"),
            ("LOG_LEVEL", "info"),
            ("RUST_LOG", "debug"),
        ]);
        let env = mock_env(&map);
        assert_eq!(get_log_level_with(&env), "error");
    }

    #[test]
    fn get_log_level_with_production_env_defaults_to_warn() {
        let map = HashMap::from([("SONGBIRD_ENV", "production")]);
        let env = mock_env(&map);
        assert_eq!(get_log_level_with(&env), "warn");
    }

    #[test]
    fn get_log_level_with_staging_env_defaults_to_info() {
        let map = HashMap::from([("SONGBIRD_ENV", "staging")]);
        let env = mock_env(&map);
        assert_eq!(get_log_level_with(&env), "info");
    }

    #[test]
    fn get_log_level_with_no_env_defaults_to_debug() {
        let map = HashMap::new();
        let env = mock_env(&map);
        assert_eq!(get_log_level_with(&env), "debug");
    }

    #[test]
    fn get_canonical_cors_origins_with_splits_comma_separated_list() {
        let map = HashMap::from([("SONGBIRD_CORS_ORIGINS", "http://example.com,http://other.com")]);
        let env = mock_env(&map);
        assert_eq!(
            get_canonical_cors_origins_with(&env),
            vec!["http://example.com".to_string(), "http://other.com".to_string(),]
        );
    }

    #[test]
    fn get_canonical_cors_origins_with_trims_whitespace() {
        let map = HashMap::from([("SONGBIRD_CORS_ORIGINS", " http://a.com , http://b.com ")]);
        let env = mock_env(&map);
        assert_eq!(
            get_canonical_cors_origins_with(&env),
            vec!["http://a.com".to_string(), "http://b.com".to_string()]
        );
    }

    #[test]
    fn get_canonical_cors_origins_with_production_denies_all_when_unset() {
        let map = HashMap::from([("SONGBIRD_ENVIRONMENT", "production")]);
        let env = mock_env(&map);
        assert!(get_canonical_cors_origins_with(&env).is_empty());
    }

    #[test]
    fn get_canonical_cors_origins_with_development_generates_default_origins() {
        let map = HashMap::from([("SONGBIRD_ENVIRONMENT", "development")]);
        let env = mock_env(&map);
        let bind_addr = songbird_types::constants::DEVELOPMENT_BIND_ADDRESS;
        let expected: Vec<String> = [3000, 8080, 8081]
            .iter()
            .flat_map(|port| {
                [format!("http://{bind_addr}:{port}"), format!("http://localhost:{port}")]
            })
            .collect();
        assert_eq!(get_canonical_cors_origins_with(&env), expected);
    }
}
