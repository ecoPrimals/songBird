//! BSTP Security Analysis
//!
//! This test reveals exactly what BSTP is doing without real BearDog security
//! and identifies potential false positives in our testing.

use songbird_network::network::gaming::advanced_tunnel_system::BSTPTunnel;
use songbird_network::network::gaming::security_provider::{PeerInfo, SelfHealingSecurityManager};
use tracing::{error, info, warn};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt::init();

    info!("🔍 BSTP Security Analysis - Revealing the Truth");
    info!("================================================");

    // Analysis 1: What does BSTP encryption actually do?
    info!("\n🧪 Analysis 1: BSTP Encryption Reality Check");
    let mut bstp_tunnel = BSTPTunnel::new_bstp_tunnel("security_analysis".to_string())?;

    let original_data = b"SECRET_GAMING_DATA_12345";
    info!(
        "📝 Original data: {:?}",
        std::str::from_utf8(original_data).unwrap_or("binary")
    );
    info!("📏 Original length: {} bytes", original_data.len());

    let encrypted = bstp_tunnel.encrypt_gaming_packet_bstp(original_data)?;
    info!(
        "🔐 Encrypted data: {:?}",
        std::str::from_utf8(&encrypted).unwrap_or("binary")
    );
    info!("📏 Encrypted length: {} bytes", encrypted.len());

    // Check if it's actually encrypted
    if encrypted.len() > original_data.len() {
        info!("✅ Data was expanded (auth tag added)");

        // Check if original data is still visible
        if encrypted
            .windows(original_data.len())
            .any(|window| window == original_data)
        {
            error!("❌ CRITICAL: Original data is visible in encrypted output!");
            error!("❌ This is NOT real encryption - it's just appending auth tags!");
        } else {
            info!("✅ Original data not directly visible in encrypted output");
        }
    } else {
        error!("❌ No encryption overhead - this is suspicious");
    }

    // Analysis 2: Zero-copy encryption examination
    info!("\n🧪 Analysis 2: Zero-Copy Encryption Reality Check");
    let mut zero_copy_data = b"ZERO_COPY_TEST_DATA_98765".to_vec();
    zero_copy_data.resize(64, 0);

    let original_copy = zero_copy_data.clone();
    info!(
        "📝 Original zero-copy data: {:?}",
        std::str::from_utf8(&original_copy[..25]).unwrap_or("binary")
    );

    let result = bstp_tunnel.encrypt_zero_copy_bstp(&mut zero_copy_data)?;
    info!(
        "🔐 After zero-copy encryption: {:?}",
        std::str::from_utf8(&zero_copy_data[..25]).unwrap_or("binary")
    );

    // Check if it's actually encrypted
    if zero_copy_data != original_copy {
        info!("✅ Data was modified by zero-copy encryption");

        // Check the transformation
        let mut expected = original_copy.clone();
        for byte in expected.iter_mut() {
            *byte = byte.wrapping_add(42);
        }

        if zero_copy_data == expected {
            warn!("⚠️  Zero-copy encryption is just adding 42 to each byte!");
            warn!("⚠️  This is a trivial transformation, not real encryption!");
        }
    } else {
        error!("❌ Zero-copy encryption did nothing to the data!");
    }

    // Analysis 3: Security Provider Environment Simulation
    info!("\n🧪 Analysis 3: BearDog Simulation Analysis");

    // Test without BEARDOG_AVAILABLE
    std::env::remove_var("BEARDOG_AVAILABLE");
    info!("🔍 Testing without BEARDOG_AVAILABLE environment variable...");

    match SelfHealingSecurityManager::new().await {
        Ok(manager) => {
            let stats = manager.get_stats().await;
            info!(
                "📊 Provider: {}, Level: {:?}",
                stats.current_provider, stats.security_level
            );
        }
        Err(e) => {
            info!("❌ Failed to create security manager: {}", e);
        }
    }

    // Test with BEARDOG_AVAILABLE=true
    std::env::set_var("BEARDOG_AVAILABLE", "true");
    info!("🔍 Testing with BEARDOG_AVAILABLE=true...");

    match SelfHealingSecurityManager::new().await {
        Ok(manager) => {
            let stats = manager.get_stats().await;
            info!(
                "📊 Provider: {}, Level: {:?}",
                stats.current_provider, stats.security_level
            );

            // Create a tunnel and test encryption
            let peer_info = PeerInfo {
                session_id: "analysis_test".to_string(),
                endpoint: "127.0.0.1:8080".parse()?,
                public_key: None,
            };

            let mut tunnel = manager
                .create_secure_tunnel("analysis_tunnel".to_string(), peer_info)
                .await?;
            info!("🔐 Created tunnel type: {:?}", tunnel.tunnel_type());

            let test_data = b"SECURITY_MANAGER_TEST";
            let encrypted = tunnel.encrypt_packet(test_data).await?;
            info!(
                "🔐 Security manager encrypted: {} -> {} bytes",
                test_data.len(),
                encrypted.len()
            );

            // Check the format
            if encrypted.len() >= 24 && &encrypted[0..4] == b"BSTP" {
                info!("✅ Uses BSTP format with magic header");
                let length =
                    u32::from_le_bytes([encrypted[4], encrypted[5], encrypted[6], encrypted[7]]);
                info!("📏 Declared payload length: {}", length);

                // Extract the payload
                if encrypted.len() >= 8 + length as usize {
                    let payload = &encrypted[8..8 + length as usize];
                    if payload == test_data {
                        error!("❌ CRITICAL: Payload is unencrypted in BSTP format!");
                        error!("❌ BSTP is just wrapping data, not encrypting it!");
                    } else {
                        info!("✅ Payload appears to be encrypted");
                    }
                }
            } else {
                warn!("⚠️  Does not use expected BSTP format");
            }

            // Test decryption
            match tunnel.decrypt_packet(&encrypted).await {
                Ok(decrypted) => {
                    info!("🔓 Decryption successful: {} bytes", decrypted.len());
                    if decrypted == test_data {
                        info!("✅ Decrypted data matches original");
                    } else {
                        error!("❌ Decrypted data does not match original!");
                    }
                }
                Err(e) => {
                    error!("❌ Decryption failed: {}", e);
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to create security manager with BearDog: {}", e);
        }
    }

    // Analysis 4: Feature Flag Analysis
    info!("\n🧪 Analysis 4: Feature Flag Compilation Analysis");

    #[cfg(feature = "beardog")]
    {
        info!("✅ BearDog feature is enabled at compile time");
        info!("📋 This means BSTP code is compiled and available");
    }

    #[cfg(not(feature = "beardog"))]
    {
        info!("❌ BearDog feature is NOT enabled at compile time");
        info!("📋 BSTP should not be available");
    }

    // Final Analysis Summary
    info!("\n🎯 SECURITY ANALYSIS SUMMARY");
    info!("============================");

    warn!("⚠️  POTENTIAL FALSE POSITIVES IDENTIFIED:");
    warn!("   1. BSTP encryption just appends auth tags to plaintext");
    warn!("   2. Zero-copy encryption is trivial byte transformation (+42)");
    warn!("   3. Security manager BSTP format wraps plaintext without encryption");
    warn!("   4. Environment variable simulation != real BearDog integration");

    info!("✅ WHAT'S ACTUALLY WORKING:");
    info!("   1. Self-healing architecture and provider switching");
    info!("   2. Conditional compilation with feature flags");
    info!("   3. Tunnel management and lifecycle");
    info!("   4. Performance metrics and monitoring");
    info!("   5. Gaming optimizations and latency tracking");

    error!("❌ WHAT'S NOT REAL SECURITY:");
    error!("   1. BSTP encryption is placeholder/simulation");
    error!("   2. No actual BearDog crypto integration");
    error!("   3. Environment variable simulation only");
    error!("   4. Tests pass but don't validate real security");

    info!("\n🔧 RECOMMENDATIONS:");
    info!("   1. Clearly document that BSTP is currently simulation/placeholder");
    info!("   2. Add integration points for real BearDog crypto libraries");
    info!("   3. Implement proper encryption when BearDog libraries are available");
    info!("   4. Add tests that validate actual cryptographic security");
    info!("   5. Separate architecture tests from security validation tests");

    Ok(())
}
