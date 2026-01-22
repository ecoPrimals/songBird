//! Fault injection tests for adaptive TLS implementation
//!
//! These tests simulate various failure modes and edge cases to ensure
//! robustness and proper error handling.

use songbird_http_client::tls::{AdaptiveExtensions, ExtensionStrategy, ExtensionType};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn fault_empty_hostname() {
    // Test: Empty hostname
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    adaptive.record_success("", vec![ExtensionType::Sni]);
    let profile = adaptive.get_profile("");
    assert!(profile.is_some());
    
    let extensions = adaptive.get_extensions("");
    assert!(!extensions.is_empty());
}

#[tokio::test]
async fn fault_empty_extension_list() {
    // Test: Empty extension list
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    adaptive.record_success("example.com", vec![]);
    let profile = adaptive.get_profile("example.com").unwrap();
    assert_eq!(profile.successful_extensions.len(), 0);
    
    // Should still return modern defaults for new requests
    let extensions = adaptive.get_extensions("example.com");
    assert_eq!(extensions.len(), 0); // Will use learned empty set
}

#[tokio::test]
async fn fault_profile_with_zero_successes() {
    // Test: Profile exists but has zero successes
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // Only record failure
    adaptive.record_failure("failing.server", vec![ExtensionType::Sni]);
    
    // Should return modern defaults (no successful profile)
    let extensions = adaptive.get_extensions("failing.server");
    assert_eq!(extensions.len(), 6); // Modern set
}

#[tokio::test]
async fn fault_duplicate_extensions_in_list() {
    // Test: Extension list with duplicates
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    let duplicates = vec![
        ExtensionType::Sni,
        ExtensionType::Sni,
        ExtensionType::Alpn,
        ExtensionType::Alpn,
    ];
    
    adaptive.record_success("duplicate.server", duplicates.clone());
    let profile = adaptive.get_profile("duplicate.server").unwrap();
    
    // Profile stores as-is (implementation doesn't dedupe)
    assert_eq!(profile.successful_extensions.len(), 4);
}

#[tokio::test]
async fn fault_unicode_hostname() {
    // Test: Unicode characters in hostname
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    let unicode_hostname = "例え.example.com";
    adaptive.record_success(unicode_hostname, vec![ExtensionType::Sni]);
    
    let profile = adaptive.get_profile(unicode_hostname);
    assert!(profile.is_some());
}

#[tokio::test]
async fn fault_profile_timestamp_in_past() {
    // Test: Profile timestamps should be reasonable
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    adaptive.record_success("example.com", vec![ExtensionType::Sni]);
    let profile = adaptive.get_profile("example.com").unwrap();
    
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(profile.last_updated).unwrap();
    
    // Timestamp should be very recent (within 1 second)
    assert!(duration < Duration::from_secs(1));
}

#[tokio::test]
async fn fault_strategy_change_after_learning() {
    // Test: Strategy changes don't affect learned profiles in Adaptive mode
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    adaptive.record_success("example.com", vec![ExtensionType::Sni]);
    
    // Create new adaptive with different strategy
    let adaptive_modern = AdaptiveExtensions::new(ExtensionStrategy::Modern);
    
    // Different strategies return different results
    let adaptive_ext = adaptive.get_extensions("example.com");
    let modern_ext = adaptive_modern.get_extensions("example.com");
    
    assert_ne!(adaptive_ext, modern_ext);
}

#[tokio::test]
async fn fault_concurrent_clear_and_access() {
    // Test: Clear while other tasks are accessing
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // Populate profiles
    for i in 0..100 {
        adaptive.record_success(
            &format!("server{}.com", i),
            vec![ExtensionType::Sni],
        );
    }
    
    let adaptive_clone = adaptive.clone();
    let accessor = tokio::spawn(async move {
        for i in 0..100 {
            let _ = adaptive_clone.get_extensions(&format!("server{}.com", i));
            sleep(Duration::from_micros(100)).await;
        }
    });
    
    // Clear in middle of accesses
    sleep(Duration::from_millis(1)).await;
    adaptive.clear_profiles();
    
    // Should not panic
    accessor.await.unwrap();
}

#[tokio::test]
async fn fault_profile_overflow_counters() {
    // Test: Very high success/failure counts
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    for _ in 0..10000 {
        adaptive.record_success("popular.server", vec![ExtensionType::Sni]);
    }
    
    let profile = adaptive.get_profile("popular.server").unwrap();
    assert_eq!(profile.success_count, 10000);
}

#[tokio::test]
async fn fault_nonexistent_profile_operations() {
    // Test: Operations on nonexistent profiles
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    // Get profile that doesn't exist
    let profile = adaptive.get_profile("nonexistent.server");
    assert!(profile.is_none());
    
    // Get extensions for unknown server (should return defaults)
    let extensions = adaptive.get_extensions("nonexistent.server");
    assert_eq!(extensions.len(), 6); // Modern defaults
}

#[tokio::test]
async fn fault_profile_with_all_extension_types() {
    // Test: Profile with every extension type
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    let all_extensions = vec![
        ExtensionType::Sni,
        ExtensionType::Alpn,
        ExtensionType::SupportedVersions,
        ExtensionType::KeyShare,
        ExtensionType::SupportedGroups,
        ExtensionType::SignatureAlgorithms,
        ExtensionType::PskKeyExchangeModes,
    ];
    
    adaptive.record_success("complete.server", all_extensions.clone());
    let extensions = adaptive.get_extensions("complete.server");
    
    assert_eq!(extensions, all_extensions);
}

#[tokio::test]
async fn fault_rapid_profile_updates() {
    // Test: Rapidly updating same profile
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    for i in 0..1000 {
        let extensions = if i % 2 == 0 {
            vec![ExtensionType::Sni]
        } else {
            vec![ExtensionType::Alpn]
        };
        adaptive.record_success("rapid.server", extensions);
    }
    
    let profile = adaptive.get_profile("rapid.server").unwrap();
    assert_eq!(profile.success_count, 1000);
    
    // Should have last recorded extension set
    assert!(
        profile.successful_extensions == vec![ExtensionType::Alpn]
    );
}

#[tokio::test]
async fn fault_profile_after_clear() {
    // Test: Accessing profile after clear
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    adaptive.record_success("example.com", vec![ExtensionType::Sni]);
    assert!(adaptive.get_profile("example.com").is_some());
    
    adaptive.clear_profiles();
    assert!(adaptive.get_profile("example.com").is_none());
}

#[tokio::test]
async fn fault_mixed_success_failure_same_server() {
    // Test: Both success and failure for same server
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    adaptive.record_success("mixed.server", vec![ExtensionType::Sni]);
    adaptive.record_failure("mixed.server", vec![ExtensionType::Alpn]);
    adaptive.record_success("mixed.server", vec![ExtensionType::KeyShare]);
    
    let profile = adaptive.get_profile("mixed.server").unwrap();
    assert_eq!(profile.success_count, 2);
    assert_eq!(profile.failure_count, 1);
    
    // Should use last successful extension set
    assert_eq!(profile.successful_extensions, vec![ExtensionType::KeyShare]);
}

#[tokio::test]
async fn fault_clone_independence() {
    // Test: Clones share profile data
    let adaptive1 = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    let adaptive2 = adaptive1.clone();
    
    adaptive1.record_success("shared.server", vec![ExtensionType::Sni]);
    
    // Clone should see the same profile
    let profile1 = adaptive1.get_profile("shared.server");
    let profile2 = adaptive2.get_profile("shared.server");
    
    assert!(profile1.is_some());
    assert!(profile2.is_some());
    assert_eq!(profile1.unwrap().success_count, profile2.unwrap().success_count);
}

#[tokio::test]
async fn fault_extension_type_equality() {
    // Test: Extension type equality works correctly
    assert_eq!(ExtensionType::Sni, ExtensionType::Sni);
    assert_ne!(ExtensionType::Sni, ExtensionType::Alpn);
    
    let ext1 = vec![ExtensionType::Sni, ExtensionType::Alpn];
    let ext2 = vec![ExtensionType::Sni, ExtensionType::Alpn];
    let ext3 = vec![ExtensionType::Alpn, ExtensionType::Sni];
    
    assert_eq!(ext1, ext2);
    assert_ne!(ext1, ext3); // Order matters
}

#[tokio::test]
async fn fault_profile_count_accuracy() {
    // Test: Profile count is always accurate
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    assert_eq!(adaptive.profile_count(), 0);
    
    adaptive.record_success("server1.com", vec![ExtensionType::Sni]);
    assert_eq!(adaptive.profile_count(), 1);
    
    adaptive.record_success("server2.com", vec![ExtensionType::Alpn]);
    assert_eq!(adaptive.profile_count(), 2);
    
    // Recording to existing profile shouldn't increase count
    adaptive.record_success("server1.com", vec![ExtensionType::KeyShare]);
    assert_eq!(adaptive.profile_count(), 2);
    
    adaptive.clear_profiles();
    assert_eq!(adaptive.profile_count(), 0);
}

#[tokio::test]
async fn fault_whitespace_in_hostname() {
    // Test: Hostnames with whitespace (unusual but shouldn't panic)
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    let hostname = "server with spaces.com";
    adaptive.record_success(hostname, vec![ExtensionType::Sni]);
    
    let profile = adaptive.get_profile(hostname);
    assert!(profile.is_some());
}

#[tokio::test]
async fn fault_very_long_extension_list() {
    // Test: Extremely long extension list (stress test)
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    let mut long_list = vec![];
    for _ in 0..1000 {
        long_list.push(ExtensionType::Sni);
    }
    
    adaptive.record_success("long.server", long_list.clone());
    let profile = adaptive.get_profile("long.server").unwrap();
    
    assert_eq!(profile.successful_extensions.len(), 1000);
}

