// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
        }
    }

    /// Parse from string
    #[expect(
        clippy::should_implement_trait,
        reason = "intentional pattern; clippy false positive for this API"
    )] // returns Option, not Result — intentionally different from FromStr
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TCP" => Some(Self::Tcp),
            "UDP" => Some(Self::Udp),
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
    #[must_use]
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
            description: format!("Songbird {} {external_port}", protocol.as_str()),
            lease_duration: crate::DEFAULT_MAPPING_TTL,
        }
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Set lease duration
    #[must_use]
    pub const fn with_lease_duration(mut self, seconds: u32) -> Self {
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
    #[must_use]
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
    #[must_use]
    pub const fn with_external_ip(mut self, ip: IpAddr) -> Self {
        self.external_ip = Some(ip);
        self
    }

    /// Time remaining before renewal needed (at half TTL)
    #[must_use]
    pub fn time_until_renewal(&self) -> Duration {
        let half_ttl = Duration::from_secs(u64::from(self.lease_duration) / 2);
        let elapsed = self.created_at.elapsed();

        if elapsed >= half_ttl {
            Duration::from_secs(0)
        } else {
            half_ttl.checked_sub(elapsed).unwrap_or_default()
        }
    }

    /// Time remaining before expiration
    #[must_use]
    pub fn time_until_expiration(&self) -> Duration {
        let ttl = Duration::from_secs(u64::from(self.lease_duration));
        let elapsed = self.created_at.elapsed();

        if elapsed >= ttl {
            Duration::from_secs(0)
        } else {
            ttl.checked_sub(elapsed).unwrap_or_default()
        }
    }

    /// Check if renewal is needed
    #[must_use]
    pub fn needs_renewal(&self) -> bool {
        self.time_until_renewal().as_secs() == 0
    }

    /// Check if expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.time_until_expiration().as_secs() == 0
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::Ipv4Addr;
    use std::time::{Duration, Instant};

    #[test]
    fn protocol_conversion_and_case_insensitivity() {
        assert_eq!(Protocol::Tcp.as_str(), "TCP");
        assert_eq!(Protocol::Udp.as_str(), "UDP");

        assert_eq!(Protocol::from_str("tcp"), Some(Protocol::Tcp), "TCP is case-insensitive");
        assert_eq!(Protocol::from_str("Udp"), Some(Protocol::Udp));
        assert_eq!(Protocol::from_str("invalid"), None);
        assert_eq!(Protocol::from_str(""), None);
    }

    #[test]
    fn port_mapping_request_builder_chains() {
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
    fn port_mapping_from_request_and_with_external_ip() {
        let req = PortMappingRequest::new(
            80,
            8080,
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            Protocol::Udp,
        );
        let m =
            PortMapping::from_request(&req).with_external_ip(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)));
        assert_eq!(m.external_port, 80);
        assert_eq!(m.internal_port, 8080);
        assert_eq!(
            m.external_ip,
            Some(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))),
            "with_external_ip should set WAN address"
        );
        assert!(m.active);
    }

    #[test]
    fn port_mapping_short_ttl_still_fresh() {
        let req = PortMappingRequest::new(
            3492,
            3492,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 144)),
            Protocol::Tcp,
        );

        let mut mapping = PortMapping::from_request(&req);
        mapping.lease_duration = 2;

        assert!(mapping.active);
        assert!(!mapping.is_expired());
    }

    fn mapping_with_created_at(lease_secs: u32, created_offset: Duration) -> PortMapping {
        PortMapping {
            external_port: 1,
            internal_port: 1,
            internal_client: IpAddr::V4(Ipv4Addr::LOCALHOST),
            external_ip: None,
            protocol: Protocol::Tcp,
            description: "t".to_string(),
            lease_duration: lease_secs,
            created_at: Instant::now()
                .checked_sub(created_offset)
                .expect("test offsets should stay within Instant representable range"),
            active: true,
        }
    }

    #[test]
    fn port_mapping_time_until_renewal_before_half_ttl() {
        let m = mapping_with_created_at(100, Duration::from_secs(10));
        let until = m.time_until_renewal();
        assert!(
            until.as_secs() > 0,
            "well before half TTL, renewal should not be due yet: {until:?}"
        );
        assert!(!m.needs_renewal(), "should not need renewal before half TTL");
    }

    #[test]
    fn port_mapping_needs_renewal_at_or_after_half_ttl() {
        let m = mapping_with_created_at(100, Duration::from_secs(50));
        assert!(m.needs_renewal(), "at half of 100s TTL, renewal should be due");
        assert_eq!(
            m.time_until_renewal(),
            Duration::from_secs(0),
            "no time left until renewal threshold"
        );
    }

    #[test]
    fn port_mapping_expiration_and_zero_lease_edge() {
        let m = mapping_with_created_at(60, Duration::from_secs(61));
        assert!(m.is_expired(), "past full TTL should be expired");
        assert_eq!(m.time_until_expiration(), Duration::from_secs(0), "no time left after expiry");

        let fresh = mapping_with_created_at(0, Duration::from_secs(0));
        assert!(fresh.needs_renewal(), "zero lease => half TTL is 0, renewal is immediately due");
    }

    #[test]
    fn port_mapping_inactive_still_reports_timing() {
        let mut m = mapping_with_created_at(3600, Duration::from_secs(0));
        m.active = false;
        assert!(!m.active);
        assert!(!m.is_expired(), "fresh mapping should not read as expired");
    }

    #[test]
    fn port_mapping_serde_roundtrip_request() {
        let req = PortMappingRequest::new(
            443,
            8443,
            IpAddr::V4(Ipv4Addr::new(192, 168, 2, 2)),
            Protocol::Tcp,
        )
        .with_lease_duration(120);
        let json = serde_json::to_string(&req).expect("serialize");
        let back: PortMappingRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(req.external_port, back.external_port);
        assert_eq!(req.internal_client, back.internal_client);
        assert_eq!(req.lease_duration, back.lease_duration);
    }
}
