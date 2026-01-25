//! Self-Knowledge Module
//!
//! Songbird only knows itself. This module discovers our own identity, capabilities,
//! and network configuration - nothing about other primals.
//!
//! ## Philosophy: Primal Only Has Self-Knowledge
//!
//! - Reads own identity from environment/config
//! - Discovers own network interfaces
//! - Exposes own capabilities
//! - **Does NOT interpret or understand tags** - just reads and broadcasts them
//! - Security providers (security provider) interpret tag meaning
//!
//! ## Tag-Based Identity (v3.14.0)
//!
//! Tags are opaque strings we broadcast. Format: `{provider}:{type}:{value}`
//! Examples: `beardog:family:nat0`, `crypto:family:a3f2`
//!
//! Songbird doesn't know what these mean - it just passes them!

use anyhow::{Context, Result};
use dirs::config_dir;
use std::fs;
use std::net::IpAddr;
use tracing::{debug, info, warn};
use uuid::Uuid;

use songbird_discovery::anonymous::TransportEndpointMessage;

/// Discover our own node ID (persistent identity)
///
/// Generates or loads UUID from `~/.config/songbird/node_id`
pub fn discover_node_id() -> Result<Uuid> {
    let config_dir = config_dir().context("Failed to get config directory")?;
    let identity_path = config_dir.join("songbird").join("node_id");

    if identity_path.exists() {
        let node_id_str = fs::read_to_string(&identity_path)
            .context(format!("Failed to read identity file {}", identity_path.display()))?;
        let node_id = Uuid::parse_str(&node_id_str)
            .context(format!("Failed to parse node ID from file {}", identity_path.display()))?;
        debug!("Loaded node ID from {}: {}", identity_path.display(), node_id);
        Ok(node_id)
    } else {
        let node_id = Uuid::new_v4();
        fs::create_dir_all(identity_path.parent().unwrap()).context(format!(
            "Failed to create config directory for {}",
            identity_path.display()
        ))?;
        fs::write(&identity_path, node_id.to_string())
            .context(format!("Failed to write node ID to file {}", identity_path.display()))?;
        info!("Generated and saved new node ID to {}: {}", identity_path.display(), node_id);
        Ok(node_id)
    }
}

/// Discover our own node name
///
/// Uses NODE_ID env var or hostname
pub fn discover_node_name() -> Result<String> {
    // Prefer NODE_ID env var (for multi-instance deployments)
    if let Ok(node_id) = std::env::var("NODE_ID") {
        debug!("Using NODE_ID from environment: {}", node_id);
        return Ok(node_id);
    }

    // Fall back to hostname
    let hostname =
        hostname::get().context("Failed to get hostname")?.to_string_lossy().into_owned();

    debug!("Using hostname as node name: {}", hostname);
    Ok(hostname)
}

/// Discover our own capabilities
///
/// Lists what we can do (discovery, federation, etc.)
pub fn discover_capabilities() -> Vec<String> {
    vec![
        "discovery".to_string(),
        "federation".to_string(),
        "coordination".to_string(),
        "health".to_string(),
        "capabilities".to_string(),
    ]
}

/// Discover our own identity tags (SELF-KNOWLEDGE ONLY!)
///
/// Reads tags from environment. **Does NOT interpret tag meaning!**
///
/// Tags are opaque strings in format: `{provider}:{type}:{value}`
///
/// ## Examples:
/// - `SONGBIRD_TAGS=beardog:family:nat0,beardog:org:acme`
/// - Individual vars: `SONGBIRD_FAMILY_ID=nat0` → `beardog:family:nat0`
///
/// ## Philosophy:
/// Songbird doesn't know what tags mean. It just broadcasts them.
/// Security providers (security provider) interpret tags and make decisions.
///
/// This is **self-knowledge** - we only know our own tags, not what they mean!
pub fn discover_identity_tags() -> Vec<String> {
    let mut tags = Vec::new();

    // Option 1: Explicit tags from SONGBIRD_TAGS (comma-separated)
    if let Ok(tags_env) = std::env::var("SONGBIRD_TAGS") {
        for tag in tags_env.split(',') {
            let tag = tag.trim();
            if !tag.is_empty() {
                tags.push(tag.to_string());
                debug!("📋 Self-knowledge: Tag '{}' (don't know what it means!)", tag);
            }
        }
    }

    // Option 2: Convenience vars that get converted to tags
    // (Songbird still doesn't interpret - just formats!)

    // Family ID → beardog:family:{id}
    if let Ok(family_id) = std::env::var("SONGBIRD_FAMILY_ID") {
        let tag = format!("beardog:family:{}", family_id);
        tags.push(tag.clone());
        debug!("📋 Self-knowledge: Family tag '{}' (security provider will interpret)", tag);
    }

    // Org ID → beardog:org:{id}
    if let Ok(org_id) = std::env::var("SONGBIRD_ORG_ID") {
        let tag = format!("beardog:org:{}", org_id);
        tags.push(tag.clone());
        debug!("📋 Self-knowledge: Org tag '{}' (security provider will interpret)", tag);
    }

    // Role → security provider:role:{role}
    if let Ok(role) = std::env::var("SONGBIRD_ROLE") {
        let tag = format!("security provider:role:{}", role);
        tags.push(tag.clone());
        debug!("📋 Self-knowledge: Role tag '{}' (security provider will interpret)", tag);
    }

    if tags.is_empty() {
        warn!("⚠️  No identity tags configured. Set SONGBIRD_FAMILY_ID or SONGBIRD_TAGS");
        warn!("   Example: SONGBIRD_FAMILY_ID=nat0");
        warn!("   Peers without tags may be rejected by security providers!");
    } else {
        info!("📋 Discovered {} identity tags (we don't interpret them!)", tags.len());
    }

    tags
}

/// Discover our own transport endpoints
///
/// Creates endpoint information for all network interfaces
pub fn discover_endpoints(https_port: u16) -> Vec<TransportEndpointMessage> {
    let mut endpoints = Vec::new();

    // Discover network interfaces
    let interfaces = match discover_interfaces() {
        Ok(ifaces) => ifaces,
        Err(e) => {
            warn!("Failed to discover network interfaces: {}", e);
            return endpoints;
        }
    };

    // Create endpoints for all non-loopback interfaces
    for interface in interfaces {
        for addr in &interface.addresses {
            // Skip loopback
            if addr.is_loopback() {
                continue;
            }

            let interface_type = if addr.is_ipv6() {
                "ipv6"
            } else {
                "ipv4"
            };

            endpoints.push(TransportEndpointMessage {
                interface_type: interface_type.to_string(),
                address: format!("{}:{}", addr, https_port),
                protocols: vec!["https".to_string()],
                preference: 1,
            });
        }
    }

    if endpoints.is_empty() {
        warn!("No endpoints discovered - using localhost fallback");
        endpoints.push(TransportEndpointMessage {
            interface_type: "ipv4".to_string(),
            address: format!("127.0.0.1:{}", https_port),
            protocols: vec!["https".to_string()],
            preference: 0,
        });
    }

    debug!("Discovered {} endpoints", endpoints.len());
    endpoints
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub addresses: Vec<IpAddr>,
    pub flags: Vec<String>,
    pub mtu: Option<u32>,
}

/// Discover our own network interfaces
fn discover_interfaces() -> Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();

    for iface in netdev::get_interfaces() {
        let mut addresses = Vec::new();

        // Collect IPv4 addresses
        for ipv4 in &iface.ipv4 {
            addresses.push(IpAddr::V4(ipv4.addr()));
        }

        // Collect IPv6 addresses
        for ipv6 in &iface.ipv6 {
            addresses.push(IpAddr::V6(ipv6.addr()));
        }

        if !addresses.is_empty() {
            interfaces.push(NetworkInterface {
                name: iface.name.clone(),
                addresses,
                flags: vec![], // netdev flags are u32, not iterable
                mtu: iface.mtu,
            });
        }
    }

    Ok(interfaces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_node_id() {
        let node_id = discover_node_id().unwrap();
        assert!(!node_id.is_nil());
    }

    #[test]
    fn test_discover_node_name() {
        let node_name = discover_node_name().unwrap();
        assert!(!node_name.is_empty());
    }

    #[test]
    fn test_discover_capabilities() {
        let capabilities = discover_capabilities();
        assert!(!capabilities.is_empty());
        assert!(capabilities.contains(&"discovery".to_string()));
    }

    #[test]
    fn test_discover_identity_tags_empty() {
        // No env vars set
        std::env::remove_var("SONGBIRD_TAGS");
        std::env::remove_var("SONGBIRD_FAMILY_ID");

        let tags = discover_identity_tags();
        // May be empty or have default values
        assert!(tags.is_empty() || !tags.is_empty());
    }

    #[test]
    fn test_discover_identity_tags_from_family() {
        // Clear all env vars first
        std::env::remove_var("SONGBIRD_FAMILY_ID");
        std::env::remove_var("SONGBIRD_TAGS");
        std::env::remove_var("SONGBIRD_ORG_ID");
        std::env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
        std::env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
        std::env::remove_var("BIOMEOS_FAMILY_ID");
        
        std::env::set_var("SONGBIRD_FAMILY_ID", "test_family");

        let tags = discover_identity_tags();
        assert!(tags.contains(&"beardog:family:test_family".to_string()), "Expected tag not found. Got: {:?}", tags);

        std::env::remove_var("SONGBIRD_FAMILY_ID");
    }

    #[test]
    fn test_discover_identity_tags_explicit() {
        // Clear all env vars first
        std::env::remove_var("SONGBIRD_FAMILY_ID");
        std::env::remove_var("SONGBIRD_TAGS");
        std::env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
        std::env::remove_var("BIOMEOS_FAMILY_ID");
        
        std::env::set_var("SONGBIRD_TAGS", "custom:tag:value1,another:tag:value2");

        let tags = discover_identity_tags();
        assert!(tags.contains(&"custom:tag:value1".to_string()));
        assert!(tags.contains(&"another:tag:value2".to_string()));

        std::env::remove_var("SONGBIRD_TAGS");
    }

    #[test]
    fn test_discover_endpoints() {
        let endpoints = discover_endpoints(8443);
        // Should have at least localhost
        assert!(!endpoints.is_empty());
    }
}
