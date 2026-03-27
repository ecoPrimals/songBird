// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🌐 API Management
//!
//! **MODERN API LAYER** ✅

pub mod ai_first_response;
pub mod ai_workload_classification;

use serde::{Deserialize, Serialize};
// use songbird_types::constants::canonical; // Not yet available
// use songbird_types::SongbirdResult;
// use songbird_config;

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub port: u16,
    pub host: String,
    pub enable_cors: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            enable_cors: true,
        }
    }
}

/// Core API handler
#[derive(Debug)]
pub struct CoreApi;

/// API handler
#[derive(Debug)]
pub struct ApiHandler;

impl Default for CoreApi {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreApi {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn api_config_default() {
        let c = ApiConfig::default();
        assert_eq!(c.port, 8080);
        assert_eq!(c.host, "127.0.0.1");
        assert!(c.enable_cors);
    }

    #[test]
    fn api_config_clone_eq() {
        let a = ApiConfig::default();
        let b = a.clone();
        assert_eq!(a.port, b.port);
        assert_eq!(a.host, b.host);
        assert_eq!(a.enable_cors, b.enable_cors);
    }

    #[test]
    fn core_api_new_and_default_equivalent() {
        let a = CoreApi::new();
        let b = CoreApi;
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
    }

    #[test]
    fn api_handler_debug_stable() {
        let h = ApiHandler;
        assert!(format!("{h:?}").contains("ApiHandler"));
    }

    #[test]
    fn api_config_serde_roundtrip() {
        let c = ApiConfig {
            port: 9443,
            host: "::1".to_string(),
            enable_cors: false,
        };
        let json = serde_json::to_string(&c).unwrap();
        let d: ApiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c.port, d.port);
        assert_eq!(c.host, d.host);
        assert_eq!(c.enable_cors, d.enable_cors);
    }

    #[test]
    fn api_config_serde_rejects_wrong_port_type() {
        let json = r#"{"port":"nope","host":"127.0.0.1"}"#;
        let err = serde_json::from_str::<ApiConfig>(json);
        assert!(err.is_err());
    }

    #[test]
    fn core_api_type_name_hint() {
        let t = std::any::type_name::<CoreApi>();
        assert!(t.contains("CoreApi"));
    }
}
