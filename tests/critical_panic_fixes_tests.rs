//! Critical Panic Fixes Tests
//! 
//! Tests to verify that critical panic risks have been eliminated
//! and error handling works correctly in edge cases

use std::time::Duration;
use songbird_orchestrator::{
    communication::HttpCommunication,
    security::SecurityConfig,
};

#[cfg(test)]
mod panic_fixes_tests {
    use super::*;

    #[test]
    fn test_http_communication_creation_never_panics() {
        // Test that HttpCommunication creation never panics, even with invalid configurations
        let base_urls = vec![
            "http://localhost:8080".to_string(),
            "https://example.com".to_string(),
            "invalid-url".to_string(),
            "".to_string(),
        ];

        for base_url in base_urls {
            // This should never panic
            let comm = HttpCommunication::new(base_url.clone());
            assert!(!comm.base_url().is_empty() || base_url.is_empty());
        }
    }

    #[test]
    fn test_http_communication_with_timeout_never_panics() {
        // Test various timeout values
        let timeouts = vec![
            Duration::from_millis(1),
            Duration::from_secs(1),
            Duration::from_secs(60),
            Duration::from_secs(3600),
        ];

        for timeout in timeouts {
            // Create fresh communication instance for each test
            let comm = HttpCommunication::new("http://localhost:8080".to_string());
            // This should never panic
            let comm_with_timeout = comm.with_timeout(timeout);
            assert_eq!(comm_with_timeout.timeout(), timeout);
        }
    }

    #[test]
    fn test_security_config_default_never_panics() {
        // Test that SecurityConfig::default() never panics
        for _ in 0..10 {
            let config = SecurityConfig::default();
            
            // Verify config is valid
            assert!(!config.jwt_secret.is_empty());
            assert!(config.jwt_expiration.as_secs() > 0);
            assert!(config.encryption_key.len() == 32);
            assert!(config.enable_audit);
        }
    }

    #[test]
    fn test_security_config_encryption_key_is_valid() {
        let config = SecurityConfig::default();
        
        // Verify the encryption key is not all zeros (which would indicate failure)
        let all_zeros = [0u8; 32];
        let all_same = config.encryption_key.iter().all(|&x| x == config.encryption_key[0]);
        
        // Key should either be random (not all same) or our fallback pattern
        assert!(
            !all_same || config.encryption_key == [42u8; 32],
            "Encryption key should be random or fallback pattern"
        );
        
        // Should not be all zeros
        assert_ne!(config.encryption_key, all_zeros, "Encryption key should not be all zeros");
    }

    #[test]
    fn test_multiple_http_clients_creation() {
        // Test creating multiple HTTP clients doesn't cause issues
        let mut clients = Vec::new();
        
        for i in 0..10 {
            let base_url = format!("http://localhost:808{}", i);
            let comm = HttpCommunication::new(base_url);
            clients.push(comm);
        }
        
        // All clients should be created successfully
        assert_eq!(clients.len(), 10);
        
        // Test with timeout modifications
        for (i, client) in clients.into_iter().enumerate() {
            let timeout = Duration::from_secs(i as u64 + 1);
            let client_with_timeout = client.with_timeout(timeout);
            assert_eq!(client_with_timeout.timeout(), timeout);
        }
    }

    #[test]
    fn test_error_handling_graceful_degradation() {
        // Test that error handling provides graceful degradation
        let extreme_timeouts = vec![
            Duration::from_nanos(1),
            Duration::from_millis(0),
            Duration::MAX,
        ];

        for timeout in extreme_timeouts {
            // Should handle gracefully without panicking
            let result = std::panic::catch_unwind(|| {
                let comm = HttpCommunication::new("http://localhost:8080".to_string());
                comm.with_timeout(timeout)
            });
            
            assert!(result.is_ok(), "HTTP client creation should not panic with extreme timeout");
        }
    }

    #[test]
    fn test_security_config_consistency() {
        // Test that multiple SecurityConfig instances are consistent
        let configs: Vec<SecurityConfig> = (0..5).map(|_| SecurityConfig::default()).collect();
        
        // JWT secret should be consistent
        let first_jwt_secret = &configs[0].jwt_secret;
        for config in &configs {
            assert_eq!(&config.jwt_secret, first_jwt_secret);
        }
        
        // Expiration should be consistent
        let first_expiration = configs[0].jwt_expiration;
        for config in &configs {
            assert_eq!(config.jwt_expiration, first_expiration);
        }
        
        // Encryption keys should be valid (random or fallback)
        for config in &configs {
            assert!(config.encryption_key.len() == 32);
            // Each key should be valid (not all zeros)
            assert_ne!(config.encryption_key, [0u8; 32]);
        }
    }
} 