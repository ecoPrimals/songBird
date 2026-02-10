//! Tests for BearDog JWT delegation
//!
//! These tests verify the capability-based discovery and JWT provisioning.
//! ✅ ALL TESTS ARE FULLY CONCURRENT — no env var mutation!

#[cfg(test)]
mod tests {
    use crate::auth::{
        discover_beardog_socket_with, get_beardog_socket_for_jwt_with, provision_jwt_secret,
    };
    use std::collections::HashMap;

    /// Create a mock env reader (concurrent-safe, no global state)
    fn mock_env(vars: HashMap<&str, &str>) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        let owned: HashMap<String, String> =
            vars.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| owned.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn test_discover_beardog_socket_with_env_var() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", "/tmp/test-beardog.sock")]));
        let socket = discover_beardog_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-beardog.sock");
    }

    #[test]
    fn test_get_beardog_socket_for_jwt() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", "/tmp/jwt-test.sock")]));
        let socket = get_beardog_socket_for_jwt_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap(), "/tmp/jwt-test.sock");
    }

    #[tokio::test]
    async fn test_provision_jwt_secret_fallback() {
        // No BearDog available, should fall back to secure random
        let secret = provision_jwt_secret(None, "test_purpose").await.unwrap();

        // Should be base64-encoded (64 bytes → ~88 characters)
        assert!(secret.len() >= 85);
        assert!(!secret.is_empty());
    }

    #[tokio::test]
    async fn test_provision_jwt_secret_different_each_time() {
        // Secure fallback should generate different secrets each time
        let secret1 = provision_jwt_secret(None, "test1").await.unwrap();
        let secret2 = provision_jwt_secret(None, "test2").await.unwrap();

        assert_ne!(secret1, secret2);
    }

    #[tokio::test]
    #[ignore = "Requires BearDog running"]
    async fn test_provision_jwt_secret_from_beardog() {
        // This test requires BearDog to be running
        // Set BEARDOG_SOCKET to test with real BearDog
        if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
            let secret = provision_jwt_secret(Some(&socket), "songbird_test").await.unwrap();

            assert!(secret.len() >= 85);
            println!("✅ Got JWT secret from BearDog: {} chars", secret.len());
        } else {
            println!("⏭️  Skipping BearDog test (set BEARDOG_SOCKET to enable)");
        }
    }
}
