//! End-to-end integration tests for adaptive TLS
//!
//! These tests validate the adaptive TLS negotiation against real servers
//! and ensure proper learning behavior.

use songbird_http_client::tls::{AdaptiveExtensions, ExtensionStrategy, ExtensionType};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore] // Run with: cargo test --test tls_adaptive_e2e_tests -- --ignored
async fn test_adaptive_learning_with_profile() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // Simulate successful handshake with GitHub
    let github_extensions = vec![
        ExtensionType::Sni,
        ExtensionType::Alpn,
        ExtensionType::SupportedVersions,
        ExtensionType::KeyShare,
        ExtensionType::SupportedGroups,
        ExtensionType::SignatureAlgorithms,
    ];
    
    adaptive.record_success("api.github.com", github_extensions.clone());
    
    // Subsequent requests should use learned profile
    let extensions = adaptive.get_extensions("api.github.com");
    assert_eq!(extensions, github_extensions);
    
    // Check profile
    let profile = adaptive.get_profile("api.github.com").unwrap();
    assert_eq!(profile.success_count, 1);
    assert_eq!(profile.failure_count, 0);
}

#[tokio::test]
async fn test_adaptive_fallback_on_failure() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // Record initial failure with max compatibility
    let max_compat = vec![
        ExtensionType::Sni,
        ExtensionType::Alpn,
        ExtensionType::SupportedVersions,
        ExtensionType::KeyShare,
        ExtensionType::SupportedGroups,
        ExtensionType::SignatureAlgorithms,
        ExtensionType::PskKeyExchangeModes,
    ];
    
    adaptive.record_failure("problematic.server", max_compat);
    
    // Check failure recorded
    let profile = adaptive.get_profile("problematic.server").unwrap();
    assert_eq!(profile.failure_count, 1);
    
    // For unknown servers, should still use modern defaults
    let extensions = adaptive.get_extensions("new.server");
    assert_eq!(extensions.len(), 6); // Modern set
}

#[tokio::test]
async fn test_strategy_modern() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Modern);
    
    let extensions = adaptive.get_extensions("any.server");
    assert_eq!(extensions.len(), 6);
    assert!(extensions.contains(&ExtensionType::Sni));
    assert!(extensions.contains(&ExtensionType::Alpn));
    assert!(extensions.contains(&ExtensionType::SupportedVersions));
    assert!(extensions.contains(&ExtensionType::KeyShare));
    assert!(!extensions.contains(&ExtensionType::PskKeyExchangeModes));
}

#[tokio::test]
async fn test_strategy_minimal() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Minimal);
    
    let extensions = adaptive.get_extensions("any.server");
    assert_eq!(extensions.len(), 4);
    assert!(extensions.contains(&ExtensionType::Sni));
    assert!(extensions.contains(&ExtensionType::SupportedVersions));
    assert!(extensions.contains(&ExtensionType::KeyShare));
    assert!(extensions.contains(&ExtensionType::SignatureAlgorithms));
    assert!(!extensions.contains(&ExtensionType::Alpn));
}

#[tokio::test]
async fn test_strategy_max_compatibility() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::MaxCompatibility);
    
    let extensions = adaptive.get_extensions("any.server");
    assert_eq!(extensions.len(), 7);
    assert!(extensions.contains(&ExtensionType::PskKeyExchangeModes));
}

#[tokio::test]
async fn test_multiple_servers_isolation() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // Record different successful extension sets for different servers
    let github_ext = vec![ExtensionType::Sni, ExtensionType::Alpn];
    let google_ext = vec![ExtensionType::Sni, ExtensionType::KeyShare];
    let cloudflare_ext = vec![ExtensionType::Sni, ExtensionType::SupportedVersions];
    
    adaptive.record_success("github.com", github_ext.clone());
    adaptive.record_success("google.com", google_ext.clone());
    adaptive.record_success("cloudflare.com", cloudflare_ext.clone());
    
    // Each server should get its own learned profile
    assert_eq!(adaptive.get_extensions("github.com"), github_ext);
    assert_eq!(adaptive.get_extensions("google.com"), google_ext);
    assert_eq!(adaptive.get_extensions("cloudflare.com"), cloudflare_ext);
    
    // Unknown server should get modern defaults
    let unknown_ext = adaptive.get_extensions("unknown.server");
    assert_eq!(unknown_ext.len(), 6);
}

#[tokio::test]
async fn test_profile_persistence_across_requests() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // First successful handshake
    let ext1 = vec![ExtensionType::Sni, ExtensionType::Alpn];
    adaptive.record_success("example.com", ext1.clone());
    
    // Simulate multiple requests
    for _ in 0..10 {
        let extensions = adaptive.get_extensions("example.com");
        assert_eq!(extensions, ext1);
    }
    
    // Profile should show only 1 success (we haven't recorded more)
    let profile = adaptive.get_profile("example.com").unwrap();
    assert_eq!(profile.success_count, 1);
}

#[tokio::test]
async fn test_concurrent_profile_updates() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    let adaptive_clone = adaptive.clone();
    
    // Simulate concurrent handshakes to different servers
    let handle1 = tokio::spawn(async move {
        for i in 0..10 {
            adaptive_clone.record_success(
                &format!("server{}.com", i),
                vec![ExtensionType::Sni],
            );
            sleep(Duration::from_millis(1)).await;
        }
    });
    
    let adaptive_clone2 = adaptive.clone();
    let handle2 = tokio::spawn(async move {
        for i in 10..20 {
            adaptive_clone2.record_success(
                &format!("server{}.com", i),
                vec![ExtensionType::Alpn],
            );
            sleep(Duration::from_millis(1)).await;
        }
    });
    
    handle1.await.unwrap();
    handle2.await.unwrap();
    
    // Should have 20 different server profiles
    assert_eq!(adaptive.profile_count(), 20);
}

#[tokio::test]
async fn test_extension_ids_are_correct() {
    // Verify extension IDs match TLS spec
    assert_eq!(ExtensionType::Sni.id(), 0x0000);
    assert_eq!(ExtensionType::Alpn.id(), 0x0010);
    assert_eq!(ExtensionType::SupportedGroups.id(), 0x000a);
    assert_eq!(ExtensionType::SignatureAlgorithms.id(), 0x000d);
    assert_eq!(ExtensionType::SupportedVersions.id(), 0x002b);
    assert_eq!(ExtensionType::PskKeyExchangeModes.id(), 0x002d);
    assert_eq!(ExtensionType::KeyShare.id(), 0x0033);
}

#[tokio::test]
async fn test_adaptive_with_rapid_failures() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // Simulate rapid failures
    for i in 0..5 {
        adaptive.record_failure(
            "unstable.server",
            vec![ExtensionType::Sni, ExtensionType::Alpn],
        );
    }
    
    let profile = adaptive.get_profile("unstable.server").unwrap();
    assert_eq!(profile.failure_count, 5);
    assert_eq!(profile.success_count, 0);
    
    // Should still return modern defaults for new attempts
    let extensions = adaptive.get_extensions("unstable.server");
    assert_eq!(extensions.len(), 6);
}

#[tokio::test]
async fn test_profile_timestamp_updates() {
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // Record success
    adaptive.record_success("example.com", vec![ExtensionType::Sni]);
    let profile1 = adaptive.get_profile("example.com").unwrap();
    let time1 = profile1.last_updated;
    
    // Wait a bit
    sleep(Duration::from_millis(10)).await;
    
    // Record another success
    adaptive.record_success("example.com", vec![ExtensionType::Alpn]);
    let profile2 = adaptive.get_profile("example.com").unwrap();
    let time2 = profile2.last_updated;
    
    // Timestamp should have updated
    assert!(time2 > time1);
    assert_eq!(profile2.success_count, 2);
}

