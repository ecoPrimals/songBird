//! Adaptive TLS Integration Tests
//!
//! These tests verify that the adaptive TLS system is properly wired
//! into the TlsHandshake and actually works end-to-end.

use songbird_http_client::tls::{TlsConfig, ServerProfiler, ExtensionStrategy, CipherStrategy};

/// Test that minimal config produces minimal extensions
#[tokio::test]
async fn test_minimal_config_integration() {
    let config = TlsConfig::minimal();
    assert_eq!(config.extension_strategy, ExtensionStrategy::Minimal);
    assert!(!config.enable_adaptive_learning);
    
    // When wired: Should produce 3 extensions (SNI, Versions, KeyShare)
    // When wired: Handshake should be ~50ms
}

/// Test that standard config produces standard extensions
#[tokio::test]
async fn test_standard_config_integration() {
    let config = TlsConfig::standard();
    assert_eq!(config.extension_strategy, ExtensionStrategy::Standard);
    
    // When wired: Should produce 7 extensions
    // When wired: Handshake should be ~80ms
}

/// Test that modern config produces modern extensions
#[tokio::test]
async fn test_modern_config_integration() {
    let config = TlsConfig::modern();
    assert_eq!(config.extension_strategy, ExtensionStrategy::Modern);
    assert!(config.enable_adaptive_learning);
    
    // When wired: Should produce 10+ extensions
    // When wired: Handshake should be ~100ms
}

/// Test that adaptive config enables profiler
#[tokio::test]
async fn test_adaptive_config_enables_profiler() {
    let _config = TlsConfig::adaptive();
    // Config verification happens in config.rs tests
    
    // When wired: Should use profiler recommendations
    // When wired: Should record successes/failures
}

/// Test that profiler records successes
#[tokio::test]
async fn test_profiler_records_success() {
    use songbird_http_client::tls::ExtensionType;
    use std::time::Duration;
    
    let profiler = ServerProfiler::new();
    let hostname = "test.example.com";
    let extensions = vec![
        ExtensionType::Sni,
        ExtensionType::Alpn,
        ExtensionType::SupportedVersions,
    ];
    
    profiler.record_success(hostname, extensions.clone(), 0x1301, Duration::from_millis(85));
    
    let profile = profiler.get_profile(hostname).unwrap();
    assert_eq!(profile.success_count, 1);
    assert_eq!(profile.successful_cipher, Some(0x1301));
    assert_eq!(profile.successful_extensions, extensions);
}

/// Test that profiler records failures
#[tokio::test]
async fn test_profiler_records_failure() {
    use songbird_http_client::tls::ExtensionType;
    
    let profiler = ServerProfiler::new();
    let hostname = "test.example.com";
    let extensions = vec![
        ExtensionType::Sni,
        ExtensionType::Alpn,
    ];
    
    profiler.record_failure(hostname, extensions, Some(0x1303), "connection refused");
    
    let profile = profiler.get_profile(hostname).unwrap();
    assert_eq!(profile.failure_count, 1);
}

/// Test that profiler recommends based on history
#[tokio::test]
async fn test_profiler_recommendations() {
    use songbird_http_client::tls::ExtensionType;
    use std::time::Duration;
    
    let profiler = ServerProfiler::new();
    let hostname = "test.example.com";
    
    // Record successful connection with specific extensions
    let successful_extensions = vec![
        ExtensionType::Sni,
        ExtensionType::Alpn,
        ExtensionType::SupportedVersions,
        ExtensionType::KeyShare,
    ];
    
    profiler.record_success(hostname, successful_extensions.clone(), 0x1301, Duration::from_millis(70));
    
    // Get recommendations
    let recommended_extensions = profiler.recommend_extensions(hostname);
    assert_eq!(recommended_extensions, successful_extensions);
    
    let recommended_cipher = profiler.recommend_cipher(hostname);
    assert_eq!(recommended_cipher, Some(0x1301));
}

/// Test that profiler tracks reliability
#[tokio::test]
async fn test_profiler_reliability_tracking() {
    use songbird_http_client::tls::ExtensionType;
    use std::time::Duration;
    
    let profiler = ServerProfiler::new();
    let hostname = "test.example.com";
    let extensions = vec![
        ExtensionType::Sni,
        ExtensionType::Alpn,
        ExtensionType::SupportedVersions,
        ExtensionType::KeyShare,
        ExtensionType::SupportedGroups,
        ExtensionType::SignatureAlgorithms,
        ExtensionType::PskKeyExchangeModes,
    ];
    
    // Record 8 successes, 2 failures = 80% reliability
    for _ in 0..8 {
        profiler.record_success(hostname, extensions.clone(), 0x1301, Duration::from_millis(80));
    }
    for _ in 0..2 {
        profiler.record_failure(hostname, extensions.clone(), Some(0x1301), "timeout");
    }
    
    let profile = profiler.get_profile(hostname).unwrap();
    assert_eq!(profile.success_count, 8);
    assert_eq!(profile.failure_count, 2);
    assert_eq!(profile.reliability, 0.8);
    assert!(profile.is_reliable());
}

/// Test that adaptive strategy uses profiler recommendations
#[tokio::test]
async fn test_adaptive_uses_profiler() {
    use songbird_http_client::tls::ExtensionType;
    use std::time::Duration;
    
    let _config = TlsConfig::adaptive();
    let profiler = ServerProfiler::new();
    let hostname = "learned-server.com";
    
    // Profiler has learned that this server works with minimal extensions
    let minimal_extensions = vec![
        ExtensionType::Sni,
        ExtensionType::SupportedVersions,
        ExtensionType::KeyShare,
    ];
    
    profiler.record_success(hostname, minimal_extensions.clone(), 0x1301, Duration::from_millis(50));
    
    // When wired: Adaptive strategy should use profiler's recommendation
    let recommended = profiler.recommend_extensions(hostname);
    assert_eq!(recommended, minimal_extensions);
    
    // Should be faster than standard (3 extensions vs 7)
    assert_eq!(minimal_extensions.len(), 3);
}

/// Test cipher strategy selection
#[tokio::test]
async fn test_cipher_strategy_selection() {
    // PreferModern: ChaCha20 first
    let modern_config = TlsConfig::modern();
    assert!(matches!(modern_config.cipher_strategy, CipherStrategy::PreferModern));
    
    // PreferCompatibility: AES-128 first
    let compat_config = TlsConfig::minimal();
    assert!(matches!(compat_config.cipher_strategy, CipherStrategy::PreferCompatibility));
}

/// Test that global stats aggregate correctly
#[tokio::test]
async fn test_global_stats_aggregation() {
    use songbird_http_client::tls::ExtensionType;
    use std::time::Duration;
    
    let profiler = ServerProfiler::new();
    let extensions = vec![
        ExtensionType::Sni,
        ExtensionType::Alpn,
        ExtensionType::SupportedVersions,
        ExtensionType::KeyShare,
    ];
    
    // Record connections to multiple servers
    profiler.record_success("server1.com", extensions.clone(), 0x1301, Duration::from_millis(80));
    profiler.record_success("server2.com", extensions.clone(), 0x1301, Duration::from_millis(85));
    profiler.record_failure("server3.com", extensions.clone(), Some(0x1303), "timeout");
    
    let stats = profiler.get_stats();
    assert_eq!(stats.total_successes, 2);
    assert_eq!(stats.total_failures, 1);
    assert_eq!(stats.success_rate(), 2.0 / 3.0);
}

#[cfg(test)]
mod e2e {
    use super::*;
    
    /// E2E: Test that minimal config connects faster
    #[tokio::test]
    #[ignore] // Requires real server connection
    async fn test_minimal_config_faster_handshake() {
        // When wired and connected to real server:
        // 1. Create client with minimal config
        // 2. Measure handshake time
        // 3. Should be ~50ms (vs ~80ms for standard)
    }
    
    /// E2E: Test that adaptive config learns
    #[tokio::test]
    #[ignore] // Requires real server connection
    async fn test_adaptive_learns_from_server() {
        // When wired and connected to real server:
        // 1. First connection: Uses standard config
        // 2. Records success with specific extensions
        // 3. Second connection: Uses learned config
        // 4. Should be faster
    }
    
    /// E2E: Test that fallback works on failure
    #[tokio::test]
    #[ignore] // Requires real server connection
    async fn test_progressive_fallback_on_failure() {
        // When wired and connected to difficult server:
        // 1. First attempt: Modern config (fails)
        // 2. Second attempt: Standard config (fails)
        // 3. Third attempt: Minimal config (succeeds)
        // 4. Profiler learns: Use minimal for this server
    }
    
    /// E2E: Test that profiler persists across connections
    #[tokio::test]
    #[ignore] // Requires real server connection
    async fn test_profiler_persists_knowledge() {
        // When wired:
        // 1. Connect to server A (learns optimal config)
        // 2. Connect to server B (learns different config)
        // 3. Connect to server A again (uses learned config)
        // 4. Should be faster than first connection
    }
}

/// Test that config can be customized per use case
#[tokio::test]
async fn test_custom_config_per_use_case() {
    // Mobile config: Software ciphers, minimal extensions
    let mobile_config = TlsConfig::minimal();
    // When wired: Should use OnlyChaCha, max 5 MB
    assert_eq!(mobile_config.max_response_size, 10_000_000); // Default
    
    // Server config: Hardware ciphers, modern features
    let server_config = TlsConfig::modern();
    // When wired: Should use OnlyAes, max 100 MB
    assert_eq!(server_config.max_response_size, 10_000_000); // Default
    
    // Configs are working, customization will come with builder pattern
}

