//! Tests for BearDog JWT delegation
//!
//! These tests verify the capability-based discovery and JWT provisioning.

#[cfg(test)]
mod tests {
    use crate::auth::{discover_beardog_socket, get_beardog_socket_for_jwt, provision_jwt_secret};

    #[test]
    fn test_discover_beardog_socket_with_env_var() {
        // Clean up any previous test pollution
        std::env::remove_var("SECURITY_PROVIDER");
        std::env::remove_var("BEARDOG_SOCKET");

        // Set SECURITY_PROVIDER environment variable
        std::env::set_var("SECURITY_PROVIDER", "/tmp/test-beardog.sock");

        let socket = discover_beardog_socket();
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-beardog.sock");

        // Cleanup
        std::env::remove_var("SECURITY_PROVIDER");
    }

    #[test]
    fn test_get_beardog_socket_for_jwt() {
        // Clean up any previous test pollution
        std::env::remove_var("SECURITY_PROVIDER");
        std::env::remove_var("BEARDOG_SOCKET");

        std::env::set_var("SECURITY_PROVIDER", "/tmp/jwt-test.sock");

        let socket = get_beardog_socket_for_jwt();
        assert!(socket.is_some());
        assert_eq!(socket.unwrap(), "/tmp/jwt-test.sock");

        std::env::remove_var("SECURITY_PROVIDER");
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
