//! Port mapping types and management

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// Network protocol for port mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    /// TCP protocol
    #[serde(rename = "TCP")]
    Tcp,
    /// UDP protocol
    #[serde(rename = "UDP")]
    Udp,
}

impl Protocol {
    /// Convert to string for SOAP/NAT-PMP
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TCP" => Some(Protocol::Tcp),
            "UDP" => Some(Protocol::Udp),
            _ => None,
        }
    }
}

/// Request to create a port mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMappingRequest {
    /// External (WAN) port
    pub external_port: u16,
    
    /// Internal (LAN) port
    pub internal_port: u16,
    
    /// Internal client IP address
    pub internal_client: IpAddr,
    
    /// Protocol (TCP or UDP)
    pub protocol: Protocol,
    
    /// Human-readable description
    pub description: String,
    
    /// Lease duration in seconds (0 = permanent)
    pub lease_duration: u32,
}

impl PortMappingRequest {
    /// Create a new port mapping request
    pub fn new(
        external_port: u16,
        internal_port: u16,
        internal_client: IpAddr,
        protocol: Protocol,
    ) -> Self {
        Self {
            external_port,
            internal_port,
            internal_client,
            protocol,
            description: format!("Songbird {} {}", protocol.as_str(), external_port),
            lease_duration: crate::DEFAULT_MAPPING_TTL,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Set lease duration
    pub fn with_lease_duration(mut self, seconds: u32) -> Self {
        self.lease_duration = seconds;
        self
    }
}

/// Active port mapping
#[derive(Debug, Clone)]
pub struct PortMapping {
    /// External (WAN) port
    pub external_port: u16,
    
    /// Internal (LAN) port
    pub internal_port: u16,
    
    /// Internal client IP
    pub internal_client: IpAddr,
    
    /// External (WAN) IP
    pub external_ip: Option<IpAddr>,
    
    /// Protocol
    pub protocol: Protocol,
    
    /// Description
    pub description: String,
    
    /// Lease duration in seconds
    pub lease_duration: u32,
    
    /// When this mapping was created
    pub created_at: Instant,
    
    /// Whether this mapping is currently active
    pub active: bool,
}

impl PortMapping {
    /// Create from request
    pub fn from_request(req: &PortMappingRequest) -> Self {
        Self {
            external_port: req.external_port,
            internal_port: req.internal_port,
            internal_client: req.internal_client,
            external_ip: None,
            protocol: req.protocol,
            description: req.description.clone(),
            lease_duration: req.lease_duration,
            created_at: Instant::now(),
            active: true,
        }
    }

    /// Set external IP
    pub fn with_external_ip(mut self, ip: IpAddr) -> Self {
        self.external_ip = Some(ip);
        self
    }

    /// Time remaining before renewal needed (at half TTL)
    pub fn time_until_renewal(&self) -> Duration {
        let half_ttl = Duration::from_secs(self.lease_duration as u64 / 2);
        let elapsed = self.created_at.elapsed();
        
        if elapsed >= half_ttl {
            Duration::from_secs(0)
        } else {
            half_ttl - elapsed
        }
    }

    /// Time remaining before expiration
    pub fn time_until_expiration(&self) -> Duration {
        let ttl = Duration::from_secs(self.lease_duration as u64);
        let elapsed = self.created_at.elapsed();
        
        if elapsed >= ttl {
            Duration::from_secs(0)
        } else {
            ttl - elapsed
        }
    }

    /// Check if renewal is needed
    pub fn needs_renewal(&self) -> bool {
        self.time_until_renewal().as_secs() == 0
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        self.time_until_expiration().as_secs() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_protocol_conversion() {
        assert_eq!(Protocol::Tcp.as_str(), "TCP");
        assert_eq!(Protocol::Udp.as_str(), "UDP");
        
        assert_eq!(Protocol::from_str("tcp"), Some(Protocol::Tcp));
        assert_eq!(Protocol::from_str("UDP"), Some(Protocol::Udp));
        assert_eq!(Protocol::from_str("invalid"), None);
    }

    #[test]
    fn test_port_mapping_request() {
        let req = PortMappingRequest::new(
            3492,
            3492,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 144)),
            Protocol::Tcp,
        )
        .with_description("Test mapping".to_string())
        .with_lease_duration(3600);

        assert_eq!(req.external_port, 3492);
        assert_eq!(req.internal_port, 3492);
        assert_eq!(req.lease_duration, 3600);
        assert_eq!(req.description, "Test mapping");
    }

    #[test]
    fn test_port_mapping_lifecycle() {
        let req = PortMappingRequest::new(
            3492,
            3492,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 144)),
            Protocol::Tcp,
        );
        
        let mut mapping = PortMapping::from_request(&req);
        mapping.lease_duration = 2; // 2 second TTL for testing
        
        assert!(mapping.active);
        assert!(!mapping.is_expired());
    }
}
