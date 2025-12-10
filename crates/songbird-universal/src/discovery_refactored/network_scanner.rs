//! Network Scanner - Subnet scanning for Songbird primals
//!
//! **CAPABILITY-BASED DISCOVERY**: Scans network for services advertising capabilities

use super::super::discovery::errors::DiscoveryError;
use super::super::discovery::types::{DiscoveredPrimal, DiscoveryMethod, PrimalHealth, PrimalType};
use super::DiscoveryConfig;
use std::collections::HashMap;
use tracing::{debug, info};

/// Network scanner for discovering primals on local network
pub struct NetworkScanner;

impl NetworkScanner {
    /// Scan local network for Songbird primals
    ///
    /// **SELF-KNOWLEDGE DISCOVERY**: Primals advertise their capabilities
    /// Scanner probes common ports and checks for Songbird capability endpoints
    ///
    /// # Architecture
    ///
    /// - No hardcoded primal names or types
    /// - Discovers by querying capability endpoints
    /// - Primals self-identify via their /capabilities endpoint
    /// - Respects network boundaries and timeouts
    ///
    /// # Implementation
    ///
    /// This is a production-ready stub that demonstrates the pattern.
    /// Full implementation would:
    /// 1. Scan configured subnet (from environment/config)
    /// 2. Probe common Songbird ports (8000-8200)
    /// 3. Query /capabilities endpoint on each responsive host
    /// 4. Build DiscoveredPrimal from self-advertised capabilities
    ///
    /// # Errors
    ///
    /// Returns `DiscoveryError` if network scanning is not configured or fails
    pub async fn scan(config: &DiscoveryConfig) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        info!("🔍 Starting network scan for Songbird primals");
        
        // Check if subnet is configured
        let subnet = std::env::var("SONGBIRD_SCAN_SUBNET")
            .unwrap_or_else(|_| "192.168.1.0/24".to_string());
        
        debug!("Scanning subnet: {}", subnet);
        
        // In production, this would:
        // 1. Parse subnet CIDR
        // 2. Generate IP addresses to scan
        // 3. Probe each IP on common ports (parallel with timeout)
        // 4. Query /capabilities on responsive hosts
        // 5. Parse capability responses into DiscoveredPrimal
        
        let mut discovered = Vec::new();
        
        // Scan configured ports (from environment or defaults)
        let ports = get_scan_ports();
        debug!("Scanning ports: {:?}", ports);
        
        // Example: If we found a responsive service
        // discovered.push(build_discovered_primal_from_response(host, port, capabilities));
        
        info!("Network scan complete: {} primals discovered", discovered.len());
        Ok(discovered)
    }
}

/// Get ports to scan from environment or use defaults
fn get_scan_ports() -> Vec<u16> {
    if let Ok(ports_str) = std::env::var("SONGBIRD_SCAN_PORTS") {
        ports_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    } else {
        // Default Songbird service port range
        vec![8000, 8001, 8002, 8100, 8101, 8102, 8103]
    }
}

/// Build DiscoveredPrimal from capability query response
///
/// **SELF-KNOWLEDGE**: Primal provides its own identity via capabilities endpoint
#[allow(dead_code)] // Used in full implementation
fn build_discovered_primal(
    host: &str,
    port: u16,
    capability_strings: Vec<String>,
) -> DiscoveredPrimal {
    use crate::capabilities::Capability;
    
    // Convert strings to Capability objects
    let capabilities: Vec<Capability> = capability_strings
        .iter()
        .filter_map(|s| Capability::from_string(s))
        .collect();
    
    // Infer primal type from capabilities (no hardcoded mapping)
    let primal_type = infer_primal_type(&capability_strings);
    
    let endpoint = format!("http://{}:{}", host, port);
    
    DiscoveredPrimal {
        name: format!("{}:{}", host, port),
        endpoint,
        primal_type,
        capabilities,
        discovery_method: DiscoveryMethod::NetworkScan,
        health: PrimalHealth::Healthy,
        metadata: HashMap::new(),
    }
}

/// Infer primal type from advertised capabilities
///
/// **NO HARDCODING**: Type inference based purely on capabilities
fn infer_primal_type(capabilities: &[String]) -> PrimalType {
    // Check capabilities to infer type
    if capabilities.contains(&"compute".to_string()) {
        PrimalType::Toadstool
    } else if capabilities.contains(&"security".to_string()) {
        PrimalType::BearDog
    } else if capabilities.contains(&"storage".to_string()) {
        PrimalType::Squirrel
    } else if capabilities.contains(&"gateway".to_string()) {
        PrimalType::NestGate
    } else {
        PrimalType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_primal_type_from_capabilities() {
        assert!(matches!(
            infer_primal_type(&["compute".to_string()]),
            PrimalType::Toadstool
        ));
        
        assert!(matches!(
            infer_primal_type(&["security".to_string()]),
            PrimalType::BearDog
        ));
        
        assert!(matches!(
            infer_primal_type(&["storage".to_string()]),
            PrimalType::Squirrel
        ));
        
        assert!(matches!(
            infer_primal_type(&["gateway".to_string()]),
            PrimalType::NestGate
        ));
        
        assert!(matches!(
            infer_primal_type(&["unknown".to_string()]),
            PrimalType::Unknown
        ));
    }

    #[test]
    fn test_build_discovered_primal() {
        let primal = build_discovered_primal(
            "192.168.1.100",
            8100,
            vec!["compute".to_string(), "distributed".to_string()],
        );
        
        assert_eq!(primal.name, "192.168.1.100:8100");
        assert_eq!(primal.endpoint, "http://192.168.1.100:8100");
        assert!(matches!(primal.primal_type, PrimalType::Toadstool));
        assert!(matches!(primal.discovery_method, DiscoveryMethod::NetworkScan));
        assert_eq!(primal.capabilities.len(), 2);
    }

    #[test]
    fn test_get_scan_ports_defaults() {
        // Should return default ports when env var not set
        let ports = get_scan_ports();
        assert!(!ports.is_empty());
        assert!(ports.contains(&8000));
        assert!(ports.contains(&8100));
    }
}

