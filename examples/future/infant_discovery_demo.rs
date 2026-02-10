//! # 🍼 Infant Discovery Demo
//!
//! **Demonstrates zero-knowledge startup with pure capability discovery**
//!
//! This example shows how a Songbird service can start with ZERO hardcoded
//! knowledge of other primals and discover everything at runtime.
//!
//! ## Zero Knowledge Philosophy
//!
//! ```text
//! ❌ OLD: Service "knows" about beardog, toadstool, squirrel
//! ✅ NEW: Service only knows itself, discovers capabilities
//! ```
//!
//! ## Running This Example
//!
//! ```bash
//! # Set environment for discovery
//! export SECURITY_ENDPOINT=http://localhost:9443
//! export COMPUTE_ENDPOINT=http://localhost:8001
//! export STORAGE_ENDPOINT=http://localhost:8002
//!
//! # Run the demo
//! cargo run --example infant_discovery_demo
//! ```
//!
//! ## Expected Output
//!
//! ```text
//! 🍼 Infant Discovery Demo - Zero Knowledge Startup
//! ===================================================
//!
//! 1️⃣ Starting with ZERO knowledge of other primals...
//! ✅ Service identity: songbird-infant-demo
//! ✅ Knows only itself!
//!
//! 2️⃣ Discovering security capability...
//! 🔍 Looking for: security
//! ✅ Found provider at: http://localhost:9443
//!
//! 3️⃣ Discovering compute capability...
//! 🔍 Looking for: compute
//! ✅ Found provider at: http://localhost:8001
//!
//! 🎊 Success! Discovered 2 capabilities with zero hardcoding!
//! ```

use std::collections::HashMap;

/// Demonstrates infant discovery - starting with zero knowledge
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍼 Infant Discovery Demo - Zero Knowledge Startup");
    println!("===================================================\n");

    // Step 1: Define self-identity (only thing we know)
    println!("1️⃣ Starting with ZERO knowledge of other primals...");
    let self_identity = ServiceIdentity {
        service_id: "songbird-infant-demo".to_string(),
        provides_capabilities: vec!["discovery".to_string()],
    };
    println!("✅ Service identity: {}", self_identity.service_id);
    println!("✅ Knows only itself!\n");

    // Step 2: Discover security capability (could be ANY provider)
    println!("2️⃣ Discovering security capability...");
    match discover_capability("security").await {
        Ok(provider) => {
            println!("🔍 Looking for: security");
            println!("✅ Found provider at: {}\n", provider.endpoint);
        }
        Err(e) => {
            println!("⚠️  No security provider found: {}", e);
            println!("   (Set SECURITY_ENDPOINT to enable)\n");
        }
    }

    // Step 3: Discover compute capability (could be ANY provider)
    println!("3️⃣ Discovering compute capability...");
    match discover_capability("compute").await {
        Ok(provider) => {
            println!("🔍 Looking for: compute");
            println!("✅ Found provider at: {}\n", provider.endpoint);
        }
        Err(e) => {
            println!("⚠️  No compute provider found: {}", e);
            println!("   (Set COMPUTE_ENDPOINT to enable)\n");
        }
    }

    // Step 4: Discover storage capability
    println!("4️⃣ Discovering storage capability...");
    match discover_capability("storage").await {
        Ok(provider) => {
            println!("🔍 Looking for: storage");
            println!("✅ Found provider at: {}\n", provider.endpoint);
        }
        Err(e) => {
            println!("⚠️  No storage provider found: {}", e);
            println!("   (Set STORAGE_ENDPOINT to enable)\n");
        }
    }

    // Step 5: Discover AI capability
    println!("5️⃣ Discovering AI capability...");
    match discover_capability("ai").await {
        Ok(provider) => {
            println!("🔍 Looking for: ai");
            println!("✅ Found provider at: {}\n", provider.endpoint);
        }
        Err(e) => {
            println!("⚠️  No AI provider found: {}", e);
            println!("   (Set AI_ENDPOINT to enable)\n");
        }
    }

    println!("🎊 Infant Discovery Complete!");
    println!("   No hardcoded primal names");
    println!("   No hardcoded endpoints");
    println!("   Pure capability-based discovery\n");

    Ok(())
}

/// Service identity - the ONLY thing a service knows about itself
#[derive(Debug)]
struct ServiceIdentity {
    service_id: String,
    provides_capabilities: Vec<String>,
}

/// Discovered capability provider
#[derive(Debug)]
struct CapabilityProvider {
    capability: String,
    endpoint: String,
    metadata: HashMap<String, String>,
}

/// Discover a capability provider with zero hardcoded knowledge
///
/// Discovery order:
/// 1. `{CAPABILITY}_ENDPOINT` environment variable
/// 2. mDNS discovery (if available)
/// 3. File-based discovery (if available)
/// 4. Network scanning (if enabled)
///
/// This function has ZERO hardcoded primal names or endpoints!
async fn discover_capability(capability: &str) -> Result<CapabilityProvider, String> {
    // Method 1: Environment variable (most common in development/production)
    let env_key = format!("{}_ENDPOINT", capability.to_uppercase());
    if let Ok(endpoint) = std::env::var(&env_key) {
        return Ok(CapabilityProvider {
            capability: capability.to_string(),
            endpoint,
            metadata: HashMap::new(),
        });
    }

    // Method 2: mDNS discovery (local network)
    // NOTE: mDNS is now available! Use songbird_config::discovery::MdnsDiscovery
    // See MDNS_INTEGRATION_COMPLETE_FEB_01_2026.md for integration guide
    // if let Ok(provider) = discover_mdns(capability).await {
    //     return Ok(provider);
    // }

    // Method 3: File-based discovery
    let config_path = format!("/etc/songbird/capabilities/{}.json", capability);
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(endpoint) = parse_endpoint_from_config(&content) {
            return Ok(CapabilityProvider {
                capability: capability.to_string(),
                endpoint,
                metadata: HashMap::new(),
            });
        }
    }

    // Method 4: Network scanning (if enabled)
    // Only scan if explicitly enabled for security
    if std::env::var("SONGBIRD_ENABLE_NETWORK_SCAN").is_ok() {
        // TODO: Implement network scanning
    }

    Err(format!(
        "No provider found for capability '{}'. Set {}_ENDPOINT environment variable.",
        capability,
        capability.to_uppercase()
    ))
}

/// Parse endpoint from configuration file
fn parse_endpoint_from_config(content: &str) -> Result<String, String> {
    // Simple JSON parsing (in production, use serde_json)
    if let Some(start) = content.find("\"endpoint\"") {
        if let Some(colon) = content[start..].find(':') {
            let after_colon = &content[start + colon + 1..];
            if let Some(quote_start) = after_colon.find('"') {
                if let Some(quote_end) = after_colon[quote_start + 1..].find('"') {
                    let endpoint = &after_colon[quote_start + 1..quote_start + 1 + quote_end];
                    return Ok(endpoint.to_string());
                }
            }
        }
    }
    Err("Failed to parse endpoint from config".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_from_env() {
        std::env::set_var("TEST_CAPABILITY_ENDPOINT", "http://test:9000");

        let result = discover_capability("test_capability").await;
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.capability, "test_capability");
        assert_eq!(provider.endpoint, "http://test:9000");

        std::env::remove_var("TEST_CAPABILITY_ENDPOINT");
    }

    #[tokio::test]
    async fn test_discovery_failure() {
        std::env::remove_var("NONEXISTENT_ENDPOINT");

        let result = discover_capability("nonexistent").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_service_identity() {
        let identity = ServiceIdentity {
            service_id: "test-service".to_string(),
            provides_capabilities: vec!["test".to_string()],
        };

        assert_eq!(identity.service_id, "test-service");
        assert_eq!(identity.provides_capabilities.len(), 1);
    }
}
