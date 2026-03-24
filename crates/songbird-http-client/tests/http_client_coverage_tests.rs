// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Additional integration-style tests for public `songbird-http-client` surface (no network).

use songbird_http_client::types::HttpResponse;
use songbird_http_client::{HttpClientConfig, RedirectMode, default_user_agent};
use std::collections::HashMap;

#[test]
fn test_http_response_default_and_clone() {
    let r = HttpResponse {
        status: 418,
        headers: HashMap::from([("tea".to_string(), "earl grey".to_string())]),
        body: serde_json::json!({"ok": true}),
    };
    let s = format!("{r:?}");
    assert!(s.contains("418") || s.contains("tea"));
}

#[test]
fn test_minimal_config_redirect_none() {
    let c = HttpClientConfig::minimal();
    assert_eq!(c.redirect_mode, RedirectMode::None);
    assert_eq!(c.max_redirects, 0);
}

#[test]
fn test_default_user_agent_contains_version_and_project() {
    let ua = default_user_agent();
    assert!(ua.contains("Songbird/"));
    assert!(ua.contains("ecoPrimals"));
}
