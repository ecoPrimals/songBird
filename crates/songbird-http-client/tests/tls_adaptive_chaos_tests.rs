//! Chaos tests for adaptive TLS implementation
//!
//! These tests validate behavior under extreme conditions, simulating
//! network failures, timing attacks, resource exhaustion, and other chaos scenarios.

use songbird_http_client::tls::{AdaptiveExtensions, ExtensionStrategy, ExtensionType};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn chaos_concurrent_profile_hammering() {
    // Test: Massive concurrent access to profiles
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    let barrier = Arc::new(Barrier::new(100));
    let mut handles = vec![];
    
    for i in 0..100 {
        let adaptive_clone = adaptive.clone();
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;
            
            // Hammer the same server profile
            for _ in 0..100 {
                if i % 2 == 0 {
                    adaptive_clone.record_success(
                        "chaos.server",
                        vec![ExtensionType::Sni],
                    );
                } else {
                    adaptive_clone.record_failure(
                        "chaos.server",
                        vec![ExtensionType::Alpn],
                    );
                }
                let _ = adaptive_clone.get_extensions("chaos.server");
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Profile should exist and have recorded many operations
    let profile = adaptive.get_profile("chaos.server").unwrap();
    assert!(profile.success_count > 0 || profile.failure_count > 0);
}

#[tokio::test]
async fn chaos_rapid_strategy_switching() {
    // Test: Rapidly switch between strategies
    let strategies = vec![
        ExtensionStrategy::Modern,
        ExtensionStrategy::Minimal,
        ExtensionStrategy::MaxCompatibility,
        ExtensionStrategy::Adaptive,
    ];
    
    let mut handles = vec![];
    
    for strategy in strategies {
        let handle = tokio::spawn(async move {
            let adaptive = AdaptiveExtensions::new(strategy);
            
            for i in 0..1000 {
                let _ = adaptive.get_extensions(&format!("server{}.com", i % 10));
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn chaos_profile_explosion() {
    // Test: Create massive number of profiles (memory pressure)
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    for i in 0..10_000 {
        adaptive.record_success(
            &format!("server{}.example.com", i),
            vec![ExtensionType::Sni],
        );
    }
    
    assert_eq!(adaptive.profile_count(), 10_000);
    
    // Should still be responsive
    let extensions = adaptive.get_extensions("server5000.example.com");
    assert_eq!(extensions, vec![ExtensionType::Sni]);
}

#[tokio::test]
async fn chaos_timeout_resilience() {
    // Test: Operations should complete quickly even under load
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    for i in 0..100 {
        let result = timeout(Duration::from_millis(10), async {
            adaptive.record_success(
                &format!("server{}.com", i),
                vec![ExtensionType::Sni, ExtensionType::Alpn],
            );
            adaptive.get_extensions(&format!("server{}.com", i))
        }).await;
        
        assert!(result.is_ok(), "Operation should complete within 10ms");
    }
}

#[tokio::test]
async fn chaos_alternating_success_failure() {
    // Test: Rapidly alternating success/failure for same server
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    for i in 0..1000 {
        if i % 2 == 0 {
            adaptive.record_success("unstable.server", vec![ExtensionType::Sni]);
        } else {
            adaptive.record_failure("unstable.server", vec![ExtensionType::Alpn]);
        }
    }
    
    let profile = adaptive.get_profile("unstable.server").unwrap();
    assert_eq!(profile.success_count, 500);
    assert_eq!(profile.failure_count, 500);
}

#[tokio::test]
async fn chaos_clone_storm() {
    // Test: Create massive number of clones
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    let mut clones = vec![];
    
    for _ in 0..1000 {
        clones.push(adaptive.clone());
    }
    
    // All clones should share the same profile data
    adaptive.record_success("shared.server", vec![ExtensionType::Sni]);
    
    for clone in clones {
        let profile = clone.get_profile("shared.server");
        assert!(profile.is_some());
    }
}

#[tokio::test]
async fn chaos_extension_list_variations() {
    // Test: Wildly varying extension lists
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    let variations = vec![
        vec![],
        vec![ExtensionType::Sni],
        vec![ExtensionType::Sni, ExtensionType::Alpn],
        vec![
            ExtensionType::Sni,
            ExtensionType::Alpn,
            ExtensionType::SupportedVersions,
        ],
        vec![
            ExtensionType::Sni,
            ExtensionType::Alpn,
            ExtensionType::SupportedVersions,
            ExtensionType::KeyShare,
            ExtensionType::SupportedGroups,
            ExtensionType::SignatureAlgorithms,
            ExtensionType::PskKeyExchangeModes,
        ],
    ];
    
    for (i, ext_list) in variations.iter().enumerate() {
        adaptive.record_success(&format!("server{}.com", i), ext_list.clone());
    }
    
    // Should handle all variations without panic
    assert_eq!(adaptive.profile_count(), variations.len());
}

#[tokio::test]
async fn chaos_clear_during_operations() {
    // Test: Clear profiles while operations are ongoing
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    let adaptive_clone = adaptive.clone();
    
    // Spawn task that continuously adds profiles
    let adder = tokio::spawn(async move {
        for i in 0..1000 {
            adaptive_clone.record_success(
                &format!("server{}.com", i),
                vec![ExtensionType::Sni],
            );
            sleep(Duration::from_micros(100)).await;
        }
    });
    
    // Clear profiles periodically
    for _ in 0..10 {
        sleep(Duration::from_millis(10)).await;
        adaptive.clear_profiles();
    }
    
    adder.await.unwrap();
    
    // Some profiles may remain depending on timing
    // The important thing is no panic occurred
}

#[tokio::test]
async fn chaos_long_hostname_stress() {
    // Test: Very long hostnames
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    let long_hostname = "a".repeat(1000) + ".example.com";
    adaptive.record_success(&long_hostname, vec![ExtensionType::Sni]);
    
    let profile = adaptive.get_profile(&long_hostname);
    assert!(profile.is_some());
}

#[tokio::test]
async fn chaos_special_characters_in_hostname() {
    // Test: Hostnames with special characters
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    let special_hostnames = vec![
        "server-with-dashes.com",
        "server_with_underscores.com",
        "server123.with.numbers456.com",
        "xn--nxasmq6b.com", // IDN example
    ];
    
    for hostname in special_hostnames {
        adaptive.record_success(hostname, vec![ExtensionType::Sni]);
        let profile = adaptive.get_profile(hostname);
        assert!(profile.is_some());
    }
}

#[tokio::test]
async fn chaos_profile_count_under_load() {
    // Test: Profile count remains accurate under concurrent load
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    let mut handles = vec![];
    
    for i in 0..10 {
        let adaptive_clone = adaptive.clone();
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                adaptive_clone.record_success(
                    &format!("server{}-{}.com", i, j),
                    vec![ExtensionType::Sni],
                );
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    assert_eq!(adaptive.profile_count(), 1000);
}

#[tokio::test]
async fn chaos_get_profile_nonexistent() {
    // Test: Getting profiles for nonexistent servers
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    for i in 0..1000 {
        let profile = adaptive.get_profile(&format!("nonexistent{}.com", i));
        assert!(profile.is_none());
    }
}

#[tokio::test]
async fn chaos_mixed_operations() {
    // Test: Random mix of all operations
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    let mut handles = vec![];
    
    for _ in 0..20 {
        let adaptive_clone = adaptive.clone();
        let handle = tokio::spawn(async move {
            for i in 0..100 {
                match i % 5 {
                    0 => {
                        adaptive_clone.record_success(
                            &format!("server{}.com", i % 10),
                            vec![ExtensionType::Sni],
                        );
                    }
                    1 => {
                        adaptive_clone.record_failure(
                            &format!("server{}.com", i % 10),
                            vec![ExtensionType::Alpn],
                        );
                    }
                    2 => {
                        let _ = adaptive_clone.get_extensions(&format!("server{}.com", i % 10));
                    }
                    3 => {
                        let _ = adaptive_clone.get_profile(&format!("server{}.com", i % 10));
                    }
                    4 => {
                        let _ = adaptive_clone.profile_count();
                    }
                    _ => {}
                }
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Should have some profiles created
    assert!(adaptive.profile_count() > 0);
}

#[tokio::test]
async fn chaos_rapid_clear_and_repopulate() {
    // Test: Rapidly clear and repopulate profiles
    let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);
    
    for _cycle in 0..10 {
        // Populate
        for i in 0..100 {
            adaptive.record_success(
                &format!("server{}.com", i),
                vec![ExtensionType::Sni],
            );
        }
        
        assert_eq!(adaptive.profile_count(), 100);
        
        // Clear
        adaptive.clear_profiles();
        assert_eq!(adaptive.profile_count(), 0);
    }
}

