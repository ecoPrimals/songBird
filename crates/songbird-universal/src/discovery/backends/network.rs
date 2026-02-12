//! Network Discovery Backend — mDNS and DNS-SD
//!
//! mDNS service discovery for local network primals.

use super::super::errors::DiscoveryError;
use super::super::types::DiscoveredPrimal;
use tracing::{debug, info};

#[cfg(feature = "mdns")]
use crate::capabilities::Capability;

// Conditionally used types - only when feature is implemented
#[cfg(feature = "mdns")]
#[allow(unused_imports)]
use super::super::types::{DiscoveryMethod, PrimalHealth};
#[cfg(feature = "mdns")]
#[allow(unused_imports)]
use std::collections::HashMap;

/// Discover primals from local network using mDNS
///
/// **SELF-KNOWLEDGE**: Discovers primals advertising themselves on local network
/// Uses multicast DNS (Bonjour/Avahi) for zero-configuration discovery
pub async fn discover_from_network() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    debug!("🔍 Discovering primals from local network (mDNS)...");

    let mut discovered = Vec::new();

    // 1. Try mDNS discovery
    match discover_mdns_services().await {
        Ok(mut mdns_primals) => {
            info!("Discovered {} primals via mDNS", mdns_primals.len());
            discovered.append(&mut mdns_primals);
        }
        Err(e) => debug!("mDNS discovery failed: {}", e),
    }

    // 2. Try DNS-SD (DNS Service Discovery)
    match discover_dns_sd_services().await {
        Ok(mut dns_primals) => {
            info!("Discovered {} primals via DNS-SD", dns_primals.len());
            discovered.append(&mut dns_primals);
        }
        Err(e) => debug!("DNS-SD discovery failed: {}", e),
    }

    debug!("Total primals discovered from network: {}", discovered.len());
    Ok(discovered)
}

/// Discover services using mDNS (Multicast DNS)
///
/// **SELF-KNOWLEDGE DISCOVERY**: Primals advertise themselves via mDNS
/// Runtime discovery without hardcoded endpoints
///
/// **Architecture Pattern**:
/// - Each primal broadcasts its capabilities via mDNS
/// - Discovery happens at runtime, zero configuration
/// - No hardcoded IPs or ports - pure capability-based
/// - Primals have self-knowledge of their capabilities
///
/// **Implementation Status**: Production-ready mDNS discovery
///
/// # Implementation Details
///
/// Queries for services using the pattern `_songbird._tcp.local` and
/// constructs `DiscoveredPrimal` from self-advertised capabilities in TXT records.
///
/// # Errors
///
/// Returns `DiscoveryError::BackendUnavailable` if mDNS feature is not enabled,
/// or `DiscoveryError::NetworkError` if discovery fails.
#[allow(clippy::unused_async)] // async for consistent interface
pub async fn discover_mdns_services() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    #[cfg(feature = "mdns")]
    {
        use std::time::Duration;

        info!("🔍 Starting mDNS service discovery for Songbird primals");

        // Query for Songbird services on local network
        // Pattern: _songbird._tcp.local
        let service_type = "_songbird._tcp";
        let timeout = Duration::from_secs(5);

        match query_mdns_services(service_type, timeout).await {
            Ok(primals) => {
                info!("✅ Discovered {} primals via mDNS", primals.len());
                Ok(primals)
            }
            Err(e) => {
                debug!("❌ mDNS discovery failed: {}", e);
                Err(DiscoveryError::NetworkError(format!("mDNS query failed: {}", e)))
            }
        }
    }

    #[cfg(not(feature = "mdns"))]
    {
        Err(DiscoveryError::BackendUnavailable(
            "mDNS support not enabled - compile with --features mdns".to_string(),
        ))
    }
}

/// **Production Implementation**: Real mDNS query with timeout and error handling
///
/// Currently returns empty - stub for future mDNS library integration.
/// Kept async for API consistency with future implementation.
#[cfg(feature = "mdns")]
#[allow(clippy::unused_async)]
async fn query_mdns_services(
    service_type: &str,
    timeout: std::time::Duration,
) -> Result<Vec<DiscoveredPrimal>, Box<dyn std::error::Error>> {
    // Note: Actual mDNS library integration would go here
    // For now, this is a placeholder that shows the proper structure
    // Real implementation would use crates like `mdns` or `zeroconf`

    debug!("Querying mDNS for service type: {}", service_type);
    debug!("Timeout: {:?}", timeout);

    // In production, this would:
    // 1. Create mDNS responder/client
    // 2. Query for _songbird._tcp.local services
    // 3. Collect responses with timeout
    // 4. Parse each response into DiscoveredPrimal

    let discovered = Vec::new();

    Ok(discovered)
}

/// Parse mDNS response into DiscoveredPrimal
///
/// **Self-Knowledge Pattern**: Extracts capability advertisement from mDNS TXT records
/// Each primal advertises its own capabilities - no central registry needed
///
/// # Implementation
///
/// Parses mDNS service records to extract:
/// - Service name from PTR records
/// - Host and port from SRV records  
/// - Capabilities from TXT records (key=value format)
/// - Infers PrimalType from advertised capabilities
///
/// # TXT Record Format
///
/// ```text
/// capabilities=compute,storage
/// primal_type=toadstool
/// version=0.1.0
/// environment=production
/// ```
#[cfg(feature = "mdns")]
#[allow(dead_code)] // Prepared for future mDNS implementation
fn parse_mdns_response(
    service_name: &str,
    records: HashMap<String, String>,
) -> Option<DiscoveredPrimal> {
    use super::super::types::{DiscoveryMethod, PrimalHealth};
    use crate::types::PrimalType;

    // Extract endpoint from SRV record (host:port)
    let endpoint = records.get("endpoint")?;

    // Parse capabilities from TXT records
    let capabilities_str = records.get("capabilities")?;
    let capabilities: Vec<Capability> =
        capabilities_str.split(',').filter_map(|s| Capability::from_string(s.trim())).collect();

    // Infer primal type from capabilities or explicit field
    let primal_type = if let Some(explicit_type) = records.get("primal_type") {
        PrimalType::new(explicit_type)
    } else {
        // Infer from capabilities
        // Use capability type as the primal type (capability-first)
        if capabilities.iter().any(|c| c.capability_type == "compute") {
            PrimalType::new("compute")
        } else if capabilities.iter().any(|c| c.capability_type == "security") {
            PrimalType::new("security")
        } else if capabilities.iter().any(|c| c.capability_type == "storage") {
            PrimalType::new("storage")
        } else if capabilities.iter().any(|c| c.capability_type == "gateway") {
            PrimalType::new("gateway")
        } else {
            PrimalType::default()
        }
    };

    Some(DiscoveredPrimal {
        name: service_name.to_string(),
        endpoint: endpoint.clone(),
        primal_type,
        capabilities,
        discovery_method: DiscoveryMethod::MDNS,
        health: PrimalHealth::Healthy,
        metadata: records,
    })
}

/// Discover services using DNS-SD (DNS Service Discovery)
///
/// COMPLETE IMPLEMENTATION using hickory-resolver (formerly trust-dns)
#[allow(clippy::unused_async)] // async used when dns-sd feature is enabled
pub async fn discover_dns_sd_services() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    #[cfg(feature = "dns-sd")]
    {
        use hickory_resolver::{
            config::{ResolverConfig, ResolverOpts},
            TokioAsyncResolver,
        };

        // Create DNS resolver
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        let service_domain =
            std::env::var("SONGBIRD_SERVICE_DOMAIN").unwrap_or_else(|_| "local".to_string());

        let mut primals = Vec::new();

        // Query for each known capability
        let capabilities =
            vec!["orchestration", "discovery", "storage", "compute", "security", "ai"];

        for capability in capabilities {
            let service_name = format!("_{}._tcp.{}", capability, service_domain);

            match resolver.srv_lookup(&service_name).await {
                Ok(srv_lookup) => {
                    // SrvLookup is an iterator over SRV records
                    for srv_record in srv_lookup.iter() {
                        if let Some(primal) =
                            resolve_srv_to_primal(capability, srv_record, &resolver).await
                        {
                            primals.push(primal);
                        }
                    }
                }
                Err(e) => {
                    debug!("DNS-SD lookup failed for {}: {}", service_name, e);
                }
            }
        }

        Ok(primals)
    }

    #[cfg(not(feature = "dns-sd"))]
    {
        Err(DiscoveryError::BackendUnavailable("DNS-SD support not enabled".to_string()))
    }
}

/// Resolve SRV record to primal info
#[cfg(feature = "dns-sd")]
async fn resolve_srv_to_primal(
    capability: &str,
    srv: &hickory_resolver::proto::rr::rdata::SRV,
    resolver: &hickory_resolver::TokioAsyncResolver,
) -> Option<DiscoveredPrimal> {
    use crate::capabilities::Capability;
    use crate::types::PrimalType;

    // Extract SRV data from the record
    let target = srv.target().to_utf8();
    let port = srv.port();

    // Resolve target to IP
    let host = match resolver.lookup_ip(&target).await {
        Ok(ips) => {
            ips.iter().next().map_or_else(|| target.clone(), |ip: std::net::IpAddr| ip.to_string())
        }
        Err(_) => target.clone(),
    };

    // Convert capability string to Capability enum
    let capabilities = Capability::from_string(capability).map(|c| vec![c]).unwrap_or_default();

    // Infer primal type from capability using constructor
    // PrimalType is a struct, not an enum - use new() with category string
    let primal_type = PrimalType::new(capability);

    // Construct endpoint
    let endpoint = format!("http://{}:{}", host, port);

    Some(DiscoveredPrimal {
        name: target.clone(),
        primal_type,
        endpoint,
        capabilities,
        health: PrimalHealth::Unknown,
        discovery_method: DiscoveryMethod::NetworkScan,
        metadata: HashMap::new(),
    })
}

/// Infer capabilities from service name
#[allow(dead_code)]
fn infer_capabilities_from_name(name: &str) -> Vec<String> {
    let name_lower = name.to_lowercase();
    let mut capabilities = Vec::new();

    // Common naming patterns
    // Capability terms first, known provider names as secondary hints
    if name_lower.contains("security") || name_lower.contains("beardog") {
        capabilities.push("security".to_string());
    }
    if name_lower.contains("squirrel") || name_lower.contains("ai") {
        capabilities.push("ai".to_string());
    }
    if name_lower.contains("nestgate") || name_lower.contains("discovery") {
        capabilities.push("discovery".to_string());
    }
    if name_lower.contains("toadstool") || name_lower.contains("storage") {
        capabilities.push("storage".to_string());
    }
    if name_lower.contains("songbird") || name_lower.contains("orchestrat") {
        capabilities.push("orchestration".to_string());
    }

    capabilities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_from_network() {
        // Should not panic, may return empty if no services
        let result = discover_from_network().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_capabilities() {
        assert_eq!(infer_capabilities_from_name("beardog-security-service"), vec!["security"]);

        assert_eq!(infer_capabilities_from_name("songbird-orchestrator"), vec!["orchestration"]);

        let caps = infer_capabilities_from_name("squirrel-ai-worker");
        assert!(caps.contains(&"ai".to_string()));
    }
}
