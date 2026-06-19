// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP request parameters from JSON-RPC
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpRequestParams {
    /// Target URL (http:// or https://)
    pub url: String,

    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    #[serde(default = "default_method")]
    pub method: String,

    /// HTTP headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Request body (optional)
    #[serde(default)]
    pub body: Option<String>,

    /// Timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_method() -> String {
    String::from("GET")
}

const fn default_timeout_ms() -> u64 {
    30_000 // 30 seconds
}

/// HTTP response for JSON-RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponseResult {
    /// HTTP status code
    pub status_code: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body as string
    pub body: String,

    /// Request elapsed time in milliseconds
    pub elapsed_ms: u128,
}

/// HTTP response from client
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
}
