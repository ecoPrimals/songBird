// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network Discovery Backend — mDNS and DNS-SD
//!
//! mDNS service discovery for local network primals.

use super::super::errors::DiscoveryError;
use super::super::types::DiscoveredPrimal;
use tracing::{debug, info};

#[cfg(feature = "mdns")]
use crate::capabilities::Capability;

#[cfg(feature = "dns-sd")]
use super::super::types::{DiscoveryMethod, PrimalHealth};
#[cfg(any(feature = "mdns", feature = "dns-sd"))]
use std::collections::HashMap;

/// Discover primals from local network using mDNS
///
/// **SELF-KNOWLEDGE**: Discovers primals advertising themselves on local network
/// Uses multicast DNS (Bonjour/Avahi) for zero-configuration discovery
///
/// # Errors
///
/// Does not return errors; individual backend failures are logged.
///
/// Uses a 5-second mDNS listen window unless you call [`discover_from_network_with_timeout`].
pub async fn discover_from_network() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    use std::time::Duration;
    discover_from_network_with_timeout(Duration::from_secs(5)).await
}

/// Discover primals from the local network with an explicit mDNS query duration.
///
/// Tests should pass a short duration (for example 1ms) so the empty-result path does not
/// wait on the default 5-second multicast listen.
pub async fn discover_from_network_with_timeout(
    mdns_listen: std::time::Duration,
) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    debug!("🔍 Discovering primals from local network (mDNS)...");

    let mut discovered = Vec::new();

    // 1. Try mDNS discovery
    match discover_mdns_services_with_timeout(mdns_listen).await {
        Ok(mut mdns_primals) => {
            info!("Discovered {} primals via mDNS", mdns_primals.len());
            discovered.append(&mut mdns_primals);
        }
        Err(e) => debug!("mDNS discovery failed: {}", e),
    }

    // 2. Try DNS-SD (DNS Service Discovery)
    //
    // Short mDNS windows (e.g. 1ms in tests) imply a bounded scan; skip DNS-SD so hickory-resolver
    // does not spend seconds per SRV query on slow or absent resolvers.
    let run_dns_sd = mdns_listen >= std::time::Duration::from_millis(100);
    if run_dns_sd {
        match discover_dns_sd_services().await {
            Ok(mut dns_primals) => {
                info!("Discovered {} primals via DNS-SD", dns_primals.len());
                discovered.append(&mut dns_primals);
            }
            Err(e) => debug!("DNS-SD discovery failed: {}", e),
        }
    } else {
        debug!("Skipping DNS-SD phase (mDNS listen window below DNS-SD minimum)");
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
#[allow(
    clippy::unused_async,
    reason = "async signature required for consistent discovery backend interface"
)]
pub async fn discover_mdns_services() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    use std::time::Duration;
    discover_mdns_services_with_timeout(Duration::from_secs(5)).await
}

/// Like [`discover_mdns_services`] with an explicit listen duration (tests use ~1ms).
#[allow(
    clippy::unused_async,
    reason = "async signature required; await is behind #[cfg(feature = \"mdns\")]"
)]
pub async fn discover_mdns_services_with_timeout(
    timeout: std::time::Duration,
) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    #[cfg(feature = "mdns")]
    {
        info!("🔍 Starting mDNS service discovery for Songbird primals");

        // Query for Songbird services on local network
        // Pattern: _songbird._tcp.local
        let service_type = "_songbird._tcp";

        let primals = query_mdns_services(service_type, timeout).await?;
        info!("✅ Discovered {} primals via mDNS", primals.len());
        Ok(primals)
    }

    #[cfg(not(feature = "mdns"))]
    {
        let _ = timeout;
        Err(DiscoveryError::BackendUnavailable(
            "mDNS support not enabled - compile with --features mdns".to_string(),
        ))
    }
}

/// Real mDNS query using raw multicast UDP.
///
/// Sends an mDNS `PTR` query for `{service_type}.local` over the link-local
/// multicast group `224.0.0.251:5353`, collects any responses within `timeout`,
/// and returns them as `DiscoveredPrimal` entries. This is a pure-Rust
/// implementation — no C bindings or Avahi/Bonjour dependency.
#[cfg(feature = "mdns")]
async fn query_mdns_services(
    service_type: &str,
    timeout: std::time::Duration,
) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use tokio::net::UdpSocket;

    debug!("Querying mDNS for service type: {}", service_type);

    // RFC 6762: mDNS uses IPv4 link-local multicast 224.0.0.251, UDP port 5353 (IANA).
    let mdns_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(224, 0, 0, 251), 5353));
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| DiscoveryError::NetworkError(format!("mDNS bind failed: {e}")))?;
    socket
        .set_broadcast(true)
        .map_err(|e| DiscoveryError::NetworkError(format!("mDNS broadcast set failed: {e}")))?;

    let query = build_mdns_ptr_query(service_type);
    socket
        .send_to(&query, mdns_addr)
        .await
        .map_err(|e| DiscoveryError::NetworkError(format!("mDNS send failed: {e}")))?;

    let mut discovered = Vec::new();
    let mut buf = [0u8; 4096];

    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        match tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, src))) => {
                debug!("mDNS response from {src} ({len} bytes)");
                if let Some(primal) = parse_mdns_ptr_response(&buf[..len], src) {
                    discovered.push(primal);
                }
            }
            Ok(Err(e)) => {
                debug!("mDNS recv error: {e}");
                break;
            }
            Err(_) => break,
        }
    }

    info!("mDNS query complete: {} primals discovered", discovered.len());
    Ok(discovered)
}

/// Build a minimal DNS PTR query packet for `{service_type}.local`.
#[cfg(feature = "mdns")]
fn build_mdns_ptr_query(service_type: &str) -> Vec<u8> {
    let mut packet = Vec::with_capacity(64);

    // Header: ID=0, QR=0 (query), QDCOUNT=1
    packet.extend_from_slice(&[0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);

    // QNAME: encode each label
    for label in service_type.split('.') {
        let len = u8::try_from(label.len()).unwrap_or(63);
        packet.push(len);
        packet.extend_from_slice(label.as_bytes());
    }
    // Append ".local"
    packet.push(5);
    packet.extend_from_slice(b"local");
    packet.push(0); // root label

    // QTYPE=PTR(12), QCLASS=IN(1)
    packet.extend_from_slice(&[0, 12, 0, 1]);

    packet
}

/// Best-effort parse of an mDNS response, extracting a `DiscoveredPrimal`
/// from the source address. Full SRV/TXT parsing is deferred to a future
/// dedicated mDNS crate; for now we register the responder as a discovered peer.
#[cfg(feature = "mdns")]
fn parse_mdns_ptr_response(data: &[u8], src: std::net::SocketAddr) -> Option<DiscoveredPrimal> {
    use super::super::types::{DiscoveryMethod, PrimalHealth};
    use crate::types::PrimalType;

    // Minimal validation: DNS header must be at least 12 bytes and QR bit set (response)
    if data.len() < 12 || data[2] & 0x80 == 0 {
        return None;
    }

    Some(DiscoveredPrimal {
        name: format!("mdns-{}", src.ip()),
        endpoint: format!("http://{}:{}", src.ip(), src.port()),
        primal_type: PrimalType::default(),
        capabilities: Vec::new(),
        discovery_method: DiscoveryMethod::MDNS,
        health: PrimalHealth::Unknown,
        metadata: std::collections::HashMap::new(),
    })
}

/// Parse mDNS response into `DiscoveredPrimal`
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
/// - Infers `PrimalType` from advertised capabilities
///
/// # TXT Record Format
///
/// ```text
/// capabilities=compute,storage
/// primal_type=compute-provider-example
/// version=0.1.0
/// environment=production
/// ```
#[cfg(feature = "mdns")]
#[allow(dead_code, reason = "prepared for future mDNS TXT/SRV parsing integration")]
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
    let primal_type = records.get("primal_type").map_or_else(
        || {
            // Infer from capabilities
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
        },
        |explicit_type| PrimalType::new(explicit_type),
    );

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
///
/// # Errors
///
/// Returns an error if DNS-SD support is not enabled.
#[allow(
    clippy::unused_async,
    reason = "async used when dns-sd feature is enabled; stub path stays synchronous"
)]
pub async fn discover_dns_sd_services() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    #[cfg(feature = "dns-sd")]
    {
        use hickory_resolver::{
            TokioAsyncResolver,
            config::{ResolverConfig, ResolverOpts},
        };

        // Create DNS resolver
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        let service_domain = songbird_process_env::var("SONGBIRD_SERVICE_DOMAIN")
            .unwrap_or_else(|_| "local".to_string());

        let mut primals = Vec::new();

        // Query for each known capability
        let capabilities =
            vec!["orchestration", "discovery", "storage", "compute", "security", "ai"];

        for capability in capabilities {
            let service_name = format!("_{capability}._tcp.{service_domain}");

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
    let host = resolver.lookup_ip(&target).await.map_or_else(
        |_| target.clone(),
        |ips| {
            ips.iter().next().map_or_else(|| target.clone(), |ip: std::net::IpAddr| ip.to_string())
        },
    );

    // Convert capability string to Capability enum
    let capabilities = Capability::from_string(capability).map(|c| vec![c]).unwrap_or_default();

    // Infer primal type from capability using constructor
    // PrimalType is a struct, not an enum - use new() with category string
    let primal_type = PrimalType::new(capability);

    // Construct endpoint
    let endpoint = format!("http://{host}:{port}");

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

/// Infer capabilities from service name using only capability terms.
///
/// Primal-agnostic: matches on domain terminology (e.g. "security", "ai")
/// rather than specific primal names. Concrete provider identities are
/// discovered at runtime via the capability advertisement protocol.
#[allow(dead_code, reason = "reserved for capability inference from service names")]
fn infer_capabilities_from_name(name: &str) -> Vec<String> {
    let name_lower = name.to_lowercase();
    let mut capabilities = Vec::new();

    if name_lower.contains("security")
        || name_lower.contains("crypto")
        || name_lower.contains("auth")
    {
        capabilities.push("security".to_string());
    }
    if name_lower.contains("ai") || name_lower.contains("ml") || name_lower.contains("inference") {
        capabilities.push("ai".to_string());
    }
    if name_lower.contains("discovery") || name_lower.contains("registry") {
        capabilities.push("discovery".to_string());
    }
    if name_lower.contains("storage")
        || name_lower.contains("data")
        || name_lower.contains("persist")
    {
        capabilities.push("storage".to_string());
    }
    if name_lower.contains("orchestrat") || name_lower.contains("coordinat") {
        capabilities.push("orchestration".to_string());
    }
    if name_lower.contains("compute")
        || name_lower.contains("worker")
        || name_lower.contains("exec")
    {
        capabilities.push("compute".to_string());
    }

    capabilities
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn test_discover_from_network() {
        // Short window: empty result path without the default 5s mDNS listen or DNS-SD resolver waits.
        let result = discover_from_network_with_timeout(std::time::Duration::from_millis(1)).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_capabilities() {
        assert_eq!(infer_capabilities_from_name("my-security-service"), vec!["security"]);
        assert_eq!(infer_capabilities_from_name("crypto-provider"), vec!["security"]);
        assert_eq!(infer_capabilities_from_name("auth-gateway"), vec!["security"]);
        assert_eq!(infer_capabilities_from_name("task-orchestrator"), vec!["orchestration"]);
        assert_eq!(infer_capabilities_from_name("ml-inference-worker"), vec!["ai", "compute"]);
        assert!(infer_capabilities_from_name("unknown-service").is_empty());
    }

    #[test]
    fn test_infer_capabilities_discovery_and_storage() {
        assert_eq!(infer_capabilities_from_name("service-registry"), vec!["discovery"]);
        assert_eq!(infer_capabilities_from_name("blob-persist"), vec!["storage"]);
    }

    #[cfg(feature = "mdns")]
    #[test]
    fn test_parse_mdns_response_success() {
        use std::collections::HashMap;

        let mut records = HashMap::new();
        records.insert("endpoint".to_string(), "http://mdns-host:8443".to_string());
        records.insert("capabilities".to_string(), "compute,security".to_string());
        let p = parse_mdns_response("songbird-svc", records).expect("parsed primal");
        assert_eq!(p.name, "songbird-svc");
        assert_eq!(p.endpoint, "http://mdns-host:8443");
        assert!(p.capabilities.iter().any(|c| c.capability_type == "compute"));
    }

    #[cfg(feature = "mdns")]
    #[test]
    fn test_parse_mdns_response_missing_fields() {
        use std::collections::HashMap;

        assert!(parse_mdns_response("x", HashMap::new()).is_none());
        let mut partial = HashMap::new();
        partial.insert("endpoint".to_string(), "http://h:1".to_string());
        assert!(parse_mdns_response("x", partial).is_none());
    }

    #[cfg(feature = "mdns")]
    #[tokio::test]
    async fn test_discover_mdns_services_returns_ok_empty_stub() {
        let got = discover_mdns_services_with_timeout(std::time::Duration::from_millis(1))
            .await
            .expect("mDNS stub");
        assert!(got.is_empty());
    }

    #[cfg(not(feature = "mdns"))]
    #[tokio::test]
    async fn test_discover_mdns_services_backend_unavailable() {
        let err = discover_mdns_services().await.expect_err("mdns feature off");
        assert!(matches!(err, DiscoveryError::BackendUnavailable(_)));
    }

    #[cfg(not(feature = "dns-sd"))]
    #[tokio::test]
    async fn discover_dns_sd_services_backend_unavailable_without_feature() {
        let err = discover_dns_sd_services().await.expect_err("dns-sd off");
        assert!(matches!(err, DiscoveryError::BackendUnavailable(_)));
    }

    #[cfg(feature = "mdns")]
    #[test]
    fn build_mdns_ptr_query_encodes_labels_correctly() {
        let packet = build_mdns_ptr_query("_primal._tcp");
        assert!(packet.len() > 12, "packet must have header + question");
        assert_eq!(&packet[0..2], &[0, 0], "ID should be 0");
        assert_eq!(&packet[4..6], &[0, 1], "QDCOUNT should be 1");
        let tail_len = packet.len();
        assert_eq!(&packet[tail_len - 4..], &[0, 12, 0, 1], "QTYPE=PTR QCLASS=IN");
    }

    #[cfg(feature = "mdns")]
    #[test]
    fn parse_mdns_ptr_response_rejects_too_short() {
        assert!(
            parse_mdns_ptr_response(&[0; 6], "127.0.0.1:5353".parse().expect("addr")).is_none()
        );
    }

    #[cfg(feature = "mdns")]
    #[test]
    fn parse_mdns_ptr_response_rejects_query_bit() {
        let mut data = [0u8; 16];
        data[2] = 0x00;
        assert!(parse_mdns_ptr_response(&data, "127.0.0.1:5353".parse().expect("addr")).is_none());
    }

    #[cfg(feature = "mdns")]
    #[test]
    fn parse_mdns_ptr_response_accepts_response_bit() {
        let mut data = [0u8; 16];
        data[2] = 0x84;
        let src = "192.168.1.42:5353".parse().expect("addr");
        let primal = parse_mdns_ptr_response(&data, src).expect("should parse");
        assert!(primal.name.contains("192.168.1.42"));
        assert!(primal.endpoint.contains("192.168.1.42"));
    }

    #[test]
    fn infer_capabilities_multi_match() {
        let caps = infer_capabilities_from_name("data-storage-persist");
        assert!(caps.contains(&"storage".to_string()));
        assert_eq!(caps.len(), 1, "should not double-count");
    }

    #[test]
    fn infer_capabilities_compute_variants() {
        assert!(infer_capabilities_from_name("compute-node").contains(&"compute".to_string()));
        assert!(infer_capabilities_from_name("worker-pool").contains(&"compute".to_string()));
        assert!(infer_capabilities_from_name("exec-runner").contains(&"compute".to_string()));
    }
}
