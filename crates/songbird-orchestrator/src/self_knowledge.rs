// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
//! - The security provider interprets tag meaning
//!
//! ## Tag-Based Identity (v3.14.0)
//!
//! Tags are opaque strings we broadcast. Format: `{provider}:{type}:{value}`
//! Examples: `crypto:family:my-family`, `crypto:family:a3f2`
//!
//! Songbird doesn't know what these mean - it just passes them!

use anyhow::{Context, Result};
use dirs::config_dir;
use std::fs;
use std::net::IpAddr;
use tracing::{debug, info, warn};
use uuid::Uuid;

use songbird_discovery::anonymous::TransportEndpointMessage;
use songbird_types::primal_names;

/// Discover our own node ID (persistent identity)
///
/// Generates or loads UUID from `~/.config/songbird/node_id`
/// # Errors
///
/// Returns an error if the operation fails.
pub fn discover_node_id() -> Result<Uuid> {
    let config_dir = config_dir().context("Failed to get config directory")?;
    let identity_path = config_dir.join(primal_names::APP_DIR).join("node_id");

    if identity_path.exists() {
        let node_id_str = fs::read_to_string(&identity_path)
            .context(format!("Failed to read identity file {}", identity_path.display()))?;
        let node_id = Uuid::parse_str(&node_id_str)
            .context(format!("Failed to parse node ID from file {}", identity_path.display()))?;
        debug!("Loaded node ID from {}: {}", identity_path.display(), node_id);
        Ok(node_id)
    } else {
        let node_id = Uuid::new_v4();
        let parent = identity_path
            .parent()
            .context(format!("Identity path has no parent: {}", identity_path.display()))?;
        fs::create_dir_all(parent).context(format!(
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
/// Uses `NODE_ID` env var or hostname
/// # Errors
///
/// Returns an error if the operation fails.
pub fn discover_node_name() -> Result<String> {
    // Prefer NODE_ID env var (for multi-instance deployments)
    if let Ok(node_id) = songbird_process_env::var("NODE_ID") {
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
#[must_use]
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
/// - `SONGBIRD_TAGS=crypto:family:nat0,crypto:org:acme` (deployments may still use legacy `beardog:*` prefixes)
/// - Individual vars: `SONGBIRD_FAMILY_ID=nat0` → family tag (see implementation; legacy wire uses `beardog:family:*`)
///
/// ## Philosophy:
/// Songbird doesn't know what tags mean. It just broadcasts them.
/// The security provider interprets tags and makes decisions.
///
/// This is **self-knowledge** - we only know our own tags, not what they mean!
#[must_use]
pub fn discover_identity_tags() -> Vec<String> {
    discover_identity_tags_with(|key| songbird_process_env::var(key).ok())
}

/// Discover identity tags with injectable environment reader (concurrent-safe)
pub fn discover_identity_tags_with<F>(env_reader: F) -> Vec<String>
where
    F: Fn(&str) -> Option<String>,
{
    let mut tags = Vec::new();

    // Option 1: Explicit tags from SONGBIRD_TAGS (comma-separated)
    if let Some(tags_env) = env_reader("SONGBIRD_TAGS") {
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

    // Family ID → legacy family tag `beardog:family:{id}` (wire compatibility)
    if let Some(family_id) = env_reader("SONGBIRD_FAMILY_ID") {
        let tag = format!("beardog:family:{family_id}");
        tags.push(tag.clone());
        debug!("📋 Self-knowledge: Family tag '{}' (security provider will interpret)", tag);
    }

    // Org ID → legacy org tag `beardog:org:{id}` (wire compatibility)
    if let Some(org_id) = env_reader("SONGBIRD_ORG_ID") {
        let tag = format!("beardog:org:{org_id}");
        tags.push(tag.clone());
        debug!("📋 Self-knowledge: Org tag '{}' (security provider will interpret)", tag);
    }

    // Role → security provider:role:{role}
    if let Some(role) = env_reader("SONGBIRD_ROLE") {
        let tag = format!("security provider:role:{role}");
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
                address: format!("{addr}:{https_port}"),
                protocols: vec!["https".to_string()],
                preference: 1,
            });
        }
    }

    if endpoints.is_empty() {
        warn!("No endpoints discovered - using localhost fallback");
        endpoints.push(TransportEndpointMessage {
            interface_type: "ipv4".to_string(),
            address: format!("127.0.0.1:{https_port}"),
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
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

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

    /// Create a mock env reader from a `HashMap` (concurrent-safe, no global state)
    fn mock_env(
        vars: std::collections::HashMap<String, String>,
    ) -> impl Fn(&str) -> Option<String> {
        move |key| vars.get(key).cloned()
    }

    #[test]
    fn test_discover_identity_tags_empty() {
        // Empty environment — no tags configured
        let tags = discover_identity_tags_with(mock_env(std::collections::HashMap::new()));
        assert!(tags.is_empty());
    }

    #[test]
    fn test_discover_identity_tags_from_family() {
        let mut env = std::collections::HashMap::new();
        env.insert("SONGBIRD_FAMILY_ID".to_string(), "test_family".to_string());

        let tags = discover_identity_tags_with(mock_env(env));
        assert!(
            tags.contains(&"beardog:family:test_family".to_string()),
            "Expected tag not found. Got: {tags:?}"
        );
    }

    #[test]
    fn test_discover_identity_tags_explicit() {
        let mut env = std::collections::HashMap::new();
        env.insert("SONGBIRD_TAGS".to_string(), "custom:tag:value1,another:tag:value2".to_string());

        let tags = discover_identity_tags_with(mock_env(env));

        assert!(
            tags.contains(&"custom:tag:value1".to_string()),
            "Expected custom:tag:value1 in tags. Got: {tags:?}"
        );
        assert!(
            tags.contains(&"another:tag:value2".to_string()),
            "Expected another:tag:value2 in tags. Got: {tags:?}"
        );
    }

    #[test]
    fn test_discover_identity_tags_org_and_role() {
        let mut env = std::collections::HashMap::new();
        env.insert("SONGBIRD_ORG_ID".to_string(), "acme".to_string());
        env.insert("SONGBIRD_ROLE".to_string(), "relay".to_string());

        let tags = discover_identity_tags_with(mock_env(env));

        assert!(tags.contains(&"beardog:org:acme".to_string()));
        assert!(tags.contains(&"security provider:role:relay".to_string()));
    }

    #[test]
    fn test_discover_identity_tags_all_sources() {
        let mut env = std::collections::HashMap::new();
        env.insert("SONGBIRD_TAGS".to_string(), "explicit:tag:1".to_string());
        env.insert("SONGBIRD_FAMILY_ID".to_string(), "nat0".to_string());
        env.insert("SONGBIRD_ORG_ID".to_string(), "org1".to_string());
        env.insert("SONGBIRD_ROLE".to_string(), "edge".to_string());

        let tags = discover_identity_tags_with(mock_env(env));

        assert_eq!(tags.len(), 4);
        assert!(tags.contains(&"explicit:tag:1".to_string()));
        assert!(tags.contains(&"beardog:family:nat0".to_string()));
        assert!(tags.contains(&"beardog:org:org1".to_string()));
        assert!(tags.contains(&"security provider:role:edge".to_string()));
    }

    #[test]
    fn test_discover_endpoints() {
        let endpoints = discover_endpoints(8443);
        // Should have at least localhost
        assert!(!endpoints.is_empty());
    }

    #[test]
    fn discover_identity_tags_trims_whitespace() {
        let mut env = std::collections::HashMap::new();
        env.insert("SONGBIRD_TAGS".to_string(), "  a:b:c  ,  d:e:f  ".to_string());
        let tags = discover_identity_tags_with(|k| env.get(k).cloned());
        assert!(tags.contains(&"a:b:c".to_string()));
        assert!(tags.contains(&"d:e:f".to_string()));
    }

    #[test]
    fn discover_capabilities_stable_set() {
        let c = discover_capabilities();
        assert!(c.iter().any(|x| x == "coordination"));
        assert_eq!(c.len(), 5);
    }

    #[test]
    fn network_interface_struct_fields() {
        let ni = NetworkInterface {
            name: "eth0".to_string(),
            addresses: vec!["127.0.0.1".parse().unwrap()],
            flags: vec![],
            mtu: Some(1500),
        };
        assert_eq!(ni.name, "eth0");
        assert_eq!(ni.mtu, Some(1500));
    }
}
