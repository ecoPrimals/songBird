// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CORS Configuration
//!
//! Cross-Origin Resource Sharing (CORS) configuration for web services.
//!
//! Default allowed origins come from `SONGBIRD_CORS_ORIGINS` (comma-separated); if unset,
//! the fallback is `http://localhost:3000`. Reads use [`songbird_types::SafeEnv`] →
//! [`songbird_process_env::var`].

#![allow(missing_docs, reason = "CORS struct fields follow standard browser terminology")]

use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            origins: SafeEnv::get_or_default(
                "SONGBIRD_CORS_ORIGINS",
                songbird_types::defaults::network::DEFAULT_CORS_ORIGIN,
            )
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
            allowed_methods: vec![String::from("GET"), String::from("POST")],
            allowed_headers: vec![String::from("Content-Type")],
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::CorsConfig;
    use songbird_test_utils::canonical_test_framework::TestContext;

    #[test]
    fn default_cors_has_expected_methods() {
        let c = CorsConfig::default();
        assert!(!c.enabled);
        assert!(c.allowed_methods.contains(&String::from("GET")));
    }

    #[test]
    fn cors_json_roundtrip() {
        let ctx = TestContext::new("cors_json");
        let c = CorsConfig {
            enabled: true,
            origins: vec![String::from("https://a.example")],
            allowed_methods: vec![String::from("PUT")],
            allowed_headers: vec![String::from("X-Req")],
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: CorsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c.enabled, back.enabled);
        assert_eq!(c.origins, back.origins);
        assert!(!ctx.is_timeout());
    }
}
