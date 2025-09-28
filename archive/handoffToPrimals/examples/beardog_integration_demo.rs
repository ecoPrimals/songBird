//! Universal Security Provider Integration Demo
//!
//! Demonstrates how any security provider integrates with Songbird through
//! the universal adapter system. This replaces hardcoded BearDog integration.
//!
//! ## Features Demonstrated
//! - Setting up universal security adapter
//! - Creating encrypted snapshots with any security provider
//! - Access control with universal authorization
//! - Audit logging with universal security events

use songbird_errors::{SongbirdError, SongbirdResult};
use songbird_universal::adapters::security::{
    UniversalSecurityAdapter, SecurityProvider, SecurityCapability, ProviderHealth,
    EncryptionContext, EncryptedData, AuthCredentials, AuthToken
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> SongbirdResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Universal Security Provider Integration Demo");

    // Create universal security adapter
    let security_adapter = UniversalSecurityAdapter::new();

    // Register a security provider (could be BearDog, custom implementation, etc.)
    let security_provider = SecurityProvider {
        id: "security-provider-1".to_string(),
        name: "Universal Security Provider".to_string(), // No hardcoded "BearDog"
        capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Decryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
            SecurityCapability::AuditLogging,
        ],
        endpoint: "https://security-provider:8443".to_string(),
        priority: 10,
        health_status: ProviderHealth::Healthy,
    };

    security_adapter.register_provider(security_provider).await?;

    // Demo 1: Universal Encryption
    demo_universal_encryption(&security_adapter).await?;

    // Demo 2: Universal Authentication  
    demo_universal_authentication(&security_adapter).await?;

    // Demo 3: Universal Authorization
    demo_universal_authorization(&security_adapter).await?;

    // Demo 4: Health Monitoring
    demo_health_monitoring(&security_adapter).await?;

    info!("✅ Universal Security Provider Integration Demo completed successfully");
    Ok(())
}

/// Demo universal encryption with any security provider
async fn demo_universal_encryption(adapter: &UniversalSecurityAdapter) -> SongbirdResult<()> {
    info!("🔒 Demo 1: Universal Encryption");

    let test_data = b"Sensitive data to encrypt";
    let context = EncryptionContext {
        algorithm: "AES-256-GCM".to_string(),
        key_id: Some("demo-key".to_string()),
    };

    // Encrypt using any available provider
    match adapter.encrypt_data(test_data, context).await {
        Ok(encrypted_data) => {
            info!("✅ Successfully encrypted {} bytes using provider: {}", 
                  test_data.len(), encrypted_data.provider_id);

            // Decrypt the data
            match adapter.decrypt_data(&encrypted_data).await {
                Ok(decrypted_data) => {
                    info!("✅ Successfully decrypted {} bytes", decrypted_data.len());
                    assert_eq!(test_data, decrypted_data.as_slice());
                }
                Err(e) => warn!("⚠️ Decryption failed: {}", e),
            }
        }
        Err(e) => {
            warn!("⚠️ No encryption providers available: {}", e);
            info!("💡 This is expected in demo mode - in production, register actual providers");
        }
    }

    Ok(())
}

/// Demo universal authentication with any security provider
async fn demo_universal_authentication(adapter: &UniversalSecurityAdapter) -> SongbirdResult<()> {
    info!("🔐 Demo 2: Universal Authentication");

    let credentials = AuthCredentials {
        username: "demo-user".to_string(),
        password: "demo-password".to_string(),
        provider: None, // Let adapter choose best provider
    };

    match adapter.authenticate(credentials).await {
        Ok(token) => {
            info!("✅ Successfully authenticated user, token from provider: {}", token.provider);
            info!("🎫 Token expires at: {}", token.expires_at);
        }
        Err(e) => {
            warn!("⚠️ Authentication failed: {}", e);
            info!("💡 This is expected in demo mode - in production, providers handle auth");
        }
    }

    Ok(())
}

/// Demo universal authorization with any security provider
async fn demo_universal_authorization(adapter: &UniversalSecurityAdapter) -> SongbirdResult<()> {
    info!("🔒 Demo 3: Universal Authorization");

    // Create a demo token (in real usage, this comes from authentication)
    let demo_token = AuthToken {
        token: "demo-token".to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        provider: "demo-provider".to_string(),
    };

    let resource = "sensitive-resource";
    let action = "read";

    match adapter.authorize(&demo_token, resource, action).await {
        Ok(authorized) => {
            if authorized {
                info!("✅ Access granted to resource: {} for action: {}", resource, action);
            } else {
                info!("❌ Access denied to resource: {} for action: {}", resource, action);
            }
        }
        Err(e) => {
            warn!("⚠️ Authorization check failed: {}", e);
            info!("💡 This is expected in demo mode - in production, providers handle authz");
        }
    }

    Ok(())
}

/// Demo health monitoring of all security providers
async fn demo_health_monitoring(adapter: &UniversalSecurityAdapter) -> SongbirdResult<()> {
    info!("📊 Demo 4: Health Monitoring");

    match adapter.health_check().await {
        Ok(health_report) => {
            info!("📈 Security Provider Health Report:");
            info!("   Total providers: {}", health_report.total_providers);
            info!("   Healthy providers: {}", health_report.healthy_providers);
            
            for (provider_id, health) in health_report.provider_status {
                info!("   Provider {}: {:?}", provider_id, health);
            }
        }
        Err(e) => {
            warn!("⚠️ Health check failed: {}", e);
        }
    }

    Ok(())
}

/// Example of registering multiple security providers
#[allow(dead_code)]
async fn demo_multiple_providers() -> SongbirdResult<()> {
    let adapter = UniversalSecurityAdapter::new();

    // Register primary security provider (could be BearDog)
    let primary_provider = SecurityProvider {
        id: "primary-security".to_string(),
        name: "Primary Security Provider".to_string(),
        capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
        ],
        endpoint: "https://primary-security:8443".to_string(),
        priority: 10,
        health_status: ProviderHealth::Healthy,
    };

    // Register backup security provider (could be custom implementation)
    let backup_provider = SecurityProvider {
        id: "backup-security".to_string(),
        name: "Backup Security Provider".to_string(),
        capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
        ],
        endpoint: "https://backup-security:8444".to_string(),
        priority: 5, // Lower priority
        health_status: ProviderHealth::Healthy,
    };

    adapter.register_provider(primary_provider).await?;
    adapter.register_provider(backup_provider).await?;

    info!("✅ Registered multiple security providers for redundancy");
    
    // The adapter will automatically use the highest priority available provider
    // and fall back to others if the primary fails
    
    Ok(())
}

/// Example of environment-driven provider configuration
#[allow(dead_code)]
async fn demo_environment_configuration() -> SongbirdResult<()> {
    let adapter = UniversalSecurityAdapter::new();

    // In production, read from environment variables:
    // SECURITY_PROVIDER_1_ENDPOINT=https://provider1:8443
    // SECURITY_PROVIDER_1_CAPABILITIES=encryption,authentication
    // SECURITY_PROVIDER_1_PRIORITY=10

    let endpoint = std::env::var("SECURITY_PROVIDER_1_ENDPOINT")
        .unwrap_or_else(|_| "https://default-security:8443".to_string());
    
    let capabilities_str = std::env::var("SECURITY_PROVIDER_1_CAPABILITIES")
        .unwrap_or_else(|_| "encryption,authentication".to_string());
    
    let priority: u8 = std::env::var("SECURITY_PROVIDER_1_PRIORITY")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);

    // Parse capabilities from environment
    let capabilities = capabilities_str
        .split(',')
        .filter_map(|cap| match cap.trim() {
            "encryption" => Some(SecurityCapability::Encryption),
            "decryption" => Some(SecurityCapability::Decryption),
            "authentication" => Some(SecurityCapability::Authentication),
            "authorization" => Some(SecurityCapability::Authorization),
            "audit" => Some(SecurityCapability::AuditLogging),
            "threat_detection" => Some(SecurityCapability::ThreatDetection),
            _ => None,
        })
        .collect();

    let provider = SecurityProvider {
        id: "env-provider".to_string(),
        name: "Environment-Configured Provider".to_string(),
        capabilities,
        endpoint,
        priority,
        health_status: ProviderHealth::Healthy,
    };

    adapter.register_provider(provider).await?;

    info!("✅ Configured security provider from environment variables");
    info!("💡 This enables deployment-time provider selection without code changes");

    Ok(())
} 