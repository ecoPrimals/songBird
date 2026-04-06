// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for security provider JWT delegation
//!
//! These tests verify the capability-based discovery and JWT provisioning.
//! ✅ ALL TESTS ARE FULLY CONCURRENT — no env var mutation!

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

#[cfg(test)]
mod tests {
    use crate::auth::{
        discover_security_socket_with, get_security_socket_for_jwt_with, provision_jwt_secret,
    };
    use std::collections::HashMap;

    fn mock_env(vars: HashMap<&str, &str>) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        let owned: HashMap<String, String> =
            vars.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| owned.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn test_discover_security_socket_with_env_var() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", "/tmp/test-security.sock")]));
        let socket = discover_security_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-security.sock");
    }

    #[test]
    fn test_get_security_socket_for_jwt() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", "/tmp/jwt-test.sock")]));
        let socket = get_security_socket_for_jwt_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap(), "/tmp/jwt-test.sock");
    }

    #[tokio::test]
    async fn test_provision_jwt_secret_fallback() {
        let secret = provision_jwt_secret(None, "test_purpose").await.unwrap();
        assert!(secret.len() >= 85);
        assert!(!secret.is_empty());
    }

    #[tokio::test]
    async fn test_provision_jwt_secret_different_each_time() {
        let secret1 = provision_jwt_secret(None, "test1").await.unwrap();
        let secret2 = provision_jwt_secret(None, "test2").await.unwrap();
        assert_ne!(secret1, secret2);
    }

    #[tokio::test]
    #[ignore = "Requires security provider running"]
    async fn test_provision_jwt_secret_from_security_provider() {
        if let Ok(socket) =
            std::env::var("SECURITY_PROVIDER_SOCKET").or_else(|_| std::env::var("BEARDOG_SOCKET"))
        {
            let secret = provision_jwt_secret(Some(&socket), "songbird_test").await.unwrap();
            assert!(secret.len() >= 85);
            println!("✅ Got JWT secret from security provider: {} chars", secret.len());
        } else {
            println!("⏭️  Skipping (set SECURITY_PROVIDER_SOCKET to enable)");
        }
    }
}
