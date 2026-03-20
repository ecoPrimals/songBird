// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🌐 API Management
//!
//! **MODERN API LAYER** ✅

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
