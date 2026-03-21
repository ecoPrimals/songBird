// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! CORS Configuration
//!
//! Cross-Origin Resource Sharing (CORS) configuration for web services.

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
            origins: SafeEnv::get_or_default("SONGBIRD_CORS_ORIGINS", "http://localhost:3000")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
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
        assert!(c.allowed_methods.contains(&"GET".to_string()));
    }

    #[test]
    fn cors_json_roundtrip() {
        let ctx = TestContext::new("cors_json");
        let c = CorsConfig {
            enabled: true,
            origins: vec!["https://a.example".to_string()],
            allowed_methods: vec!["PUT".to_string()],
            allowed_headers: vec!["X-Req".to_string()],
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: CorsConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c.enabled, back.enabled);
        assert_eq!(c.origins, back.origins);
        assert!(!ctx.is_timeout());
    }
}
