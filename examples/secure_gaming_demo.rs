use songbird_gaming_bridge::errors::Result;
/// Secure Gaming Demo - World-Class Privilege Management
///
/// This demo showcases the secure, agnostic privilege management system
/// for packet capture in gaming scenarios.
use songbird_gaming_bridge::network::gaming::{
    can_capture_packets, create_safe_privilege_manager, GamingManager, PrivilegeConfig,
    PrivilegeManager, PrivilegeMethod,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🎮 Songbird Secure Gaming Demo");
    info!("================================");

    // Step 1: Check current privilege status
    info!("\n📋 Step 1: Checking Current Privileges");
    let can_capture = can_capture_packets().await;
    info!(
        "   Packet capture available: {}",
        if can_capture { "✅ Yes" } else { "❌ No" }
    );

    // Step 2: Initialize privilege manager
    info!("\n🔐 Step 2: Initializing Privilege Manager");
    let _privilege_manager = match create_safe_privilege_manager().await {
        Ok(manager) => {
            info!("   ✅ Privilege manager created successfully");

            if manager.requires_privileges() {
                info!("   📋 Setup instructions:");
                let instructions = manager.get_setup_instructions();
                for instruction in instructions {
                    info!("      {}", instruction);
                }
            }

            Some(manager)
        }
        Err(e) => {
            warn!("   ⚠️  Could not create privilege manager: {}", e);
            None
        }
    };

    // Step 3: Create gaming manager with secure initialization
    info!("\n🎮 Step 3: Creating Gaming Manager");
    let mut gaming_manager = GamingManager::new().await?;

    // Initialize secure packet capture
    info!("   🔐 Initializing secure packet capture...");
    gaming_manager.initialize_secure_capture().await?;

    // Step 4: Demonstrate different privilege scenarios
    info!("\n🔬 Step 4: Demonstrating Privilege Scenarios");
    demonstrate_privilege_scenarios().await?;

    // Step 5: Test packet capture with current privileges
    info!("\n📡 Step 5: Testing Packet Capture");
    test_packet_capture(&mut gaming_manager).await?;

    // Step 6: Show security best practices
    info!("\n🛡️  Step 6: Security Best Practices");
    show_security_best_practices();

    info!("\n✅ Secure Gaming Demo Complete!");
    Ok(())
}

/// Demonstrate different privilege escalation scenarios
async fn demonstrate_privilege_scenarios() -> Result<()> {
    info!("   Testing different privilege methods...");

    // Test each privilege method
    let methods = vec![
        ("Capabilities (Recommended)", PrivilegeMethod::Capabilities),
        ("Sudo (Common)", PrivilegeMethod::Sudo),
        ("PolicyKit (GUI)", PrivilegeMethod::PolicyKit),
        ("Docker Capabilities", PrivilegeMethod::DockerCapabilities),
        ("Systemd Service", PrivilegeMethod::SystemdService),
        ("Unprivileged (Limited)", PrivilegeMethod::Unprivileged),
    ];

    for (name, method) in methods {
        let config = PrivilegeConfig {
            prefer_tags: matches!(method, PrivilegeMethod::Capabilities),
            allow_sudo: matches!(method, PrivilegeMethod::Sudo),
            allow_setuid: false, // Always disabled for security
            allow_systemd: matches!(method, PrivilegeMethod::SystemdService),
            fallback_to_unprivileged: true,
            custom_sudo_command: None,
        };

        match PrivilegeManager::new(config).await {
            Ok(manager) => {
                info!("      ✅ {}: Available", name);
                if manager.requires_privileges() {
                    info!("         Requires setup: Yes");
                } else {
                    info!("         Requires setup: No");
                }
            }
            Err(_) => {
                info!("      ❌ {}: Not available", name);
            }
        }
    }

    Ok(())
}

/// Test packet capture with current privileges
async fn test_packet_capture(gaming_manager: &mut GamingManager) -> Result<()> {
    info!("   Attempting to scan for games...");

    match gaming_manager.scan_for_games(None).await {
        Ok(sessions) => {
            info!("   ✅ Scan completed successfully");
            info!("   📊 Found {} gaming sessions", sessions.len());

            for session in sessions.iter().take(3) {
                info!(
                    "      🎮 {}: {} (confidence: {:.1}%)",
                    session
                        .game_name
                        .as_ref()
                        .unwrap_or(&"Unknown Game".to_string()),
                    session.protocol_class,
                    session.confidence * 100.0
                );
            }
        }
        Err(e) => {
            warn!("   ⚠️  Scan failed: {}", e);
            info!("   💡 This may be due to insufficient privileges");
        }
    }

    Ok(())
}

/// Show security best practices for packet capture
fn show_security_best_practices() {
    info!("   🔒 Recommended Security Practices:");
    info!("      1. Use Linux capabilities instead of sudo when possible");
    info!("         sudo setcap cap_net_raw+ep $(which songbird)");
    info!("      2. Run as dedicated user with minimal privileges");
    info!("      3. Use systemd service for production deployments");
    info!("      4. Never use setuid binaries in production");
    info!("      5. Regularly audit privilege usage");

    info!("   🐳 Docker Deployment:");
    info!("      docker run --cap-add=NET_ADMIN songbird");
    info!("      # Minimal capabilities, no root access needed");

    info!("   ⚙️  Production Setup:");
    info!("      1. Create dedicated songbird user");
    info!("      2. Set capabilities on binary");
    info!("      3. Use systemd service with DynamicUser=true");
    info!("      4. Enable network namespacing");

    info!("   🔍 Privilege Verification:");
    info!("      getcap $(which songbird)  # Check capabilities");
    info!("      songbird gaming diagnostics  # Test permissions");
}

/// Demonstrate zero-trust security model
async fn demonstrate_zero_trust_security() -> Result<()> {
    info!("   🛡️  Zero-Trust Security Model:");
    info!("      ✅ Principle of least privilege");
    info!("      ✅ Secure by default configuration");
    info!("      ✅ Automatic privilege detection");
    info!("      ✅ Graceful degradation without privileges");
    info!("      ✅ Comprehensive audit logging");
    info!("      ✅ No hardcoded privilege assumptions");

    Ok(())
}
