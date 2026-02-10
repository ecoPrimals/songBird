//! NAT-PMP (NAT Port Mapping Protocol) implementation — RFC 6886
//!
//! Simple binary UDP protocol for routers that support it (Apple AirPort, etc.).
//! Sends packets to gateway:5351. Much simpler than UPnP.

use crate::error::{IgdError, Result};
use crate::mapping::Protocol;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{debug, trace};

/// NAT-PMP client
pub struct NatPmpClient {
    /// Gateway address (port 5351)
    gateway: SocketAddr,
    /// Response timeout
    timeout: Duration,
}

/// NAT-PMP mapping response
#[derive(Debug, Clone)]
pub struct NatPmpMappingResponse {
    /// Internal port that was mapped
    pub internal_port: u16,
    /// External port assigned by router
    pub external_port: u16,
    /// Lifetime granted in seconds
    pub lifetime: u32,
}

/// NAT-PMP external IP response
#[derive(Debug, Clone)]
pub struct NatPmpExternalIp {
    /// External IPv4 address
    pub ip: Ipv4Addr,
    /// Seconds since router boot
    pub epoch: u32,
}

impl NatPmpClient {
    /// Create NAT-PMP client targeting the given gateway IP
    pub fn new(gateway_ip: IpAddr) -> Self {
        Self {
            gateway: SocketAddr::new(gateway_ip, crate::NAT_PMP_PORT),
            timeout: Duration::from_secs(3),
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(gateway_ip: IpAddr, timeout: Duration) -> Self {
        Self {
            gateway: SocketAddr::new(gateway_ip, crate::NAT_PMP_PORT),
            timeout,
        }
    }

    /// Get external IP address from gateway
    pub async fn get_external_ip(&self) -> Result<NatPmpExternalIp> {
        debug!("NAT-PMP: Requesting external IP from {}", self.gateway);

        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        // Request: version=0, opcode=0 (2 bytes)
        let request = [0x00u8, 0x00];
        socket.send_to(&request, self.gateway).await?;

        let mut buf = [0u8; 12];
        let (len, _) = tokio::time::timeout(self.timeout, socket.recv_from(&mut buf))
            .await
            .map_err(|_| IgdError::Timeout)??;

        if len < 12 {
            return Err(IgdError::InvalidResponse(format!(
                "NAT-PMP response too short: {} bytes (expected 12)",
                len
            )));
        }

        // Validate response
        // byte 0: version (0)
        // byte 1: opcode (128 = 0x80, response flag set)
        // bytes 2-3: result code (0 = success)
        // bytes 4-7: seconds since epoch
        // bytes 8-11: external IP address
        let result_code = u16::from_be_bytes([buf[2], buf[3]]);

        if buf[1] != 128 {
            return Err(IgdError::InvalidResponse(format!(
                "NAT-PMP unexpected opcode: {} (expected 128)",
                buf[1]
            )));
        }

        if result_code != 0 {
            return Err(IgdError::NatPmpError(format!(
                "NAT-PMP error code: {} ({})",
                result_code,
                nat_pmp_error_description(result_code)
            )));
        }

        let epoch = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let ip = Ipv4Addr::new(buf[8], buf[9], buf[10], buf[11]);

        debug!("NAT-PMP: External IP = {}, epoch = {}", ip, epoch);

        Ok(NatPmpExternalIp {
            ip,
            epoch,
        })
    }

    /// Request a port mapping
    pub async fn map_port(
        &self,
        internal_port: u16,
        external_port: u16,
        protocol: Protocol,
        lifetime: u32,
    ) -> Result<NatPmpMappingResponse> {
        debug!(
            "NAT-PMP: Mapping {} {}:{} -> :{}  (TTL {}s)",
            protocol.as_str(),
            internal_port,
            external_port,
            external_port,
            lifetime
        );

        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        let opcode: u8 = match protocol {
            Protocol::Udp => 0x01,
            Protocol::Tcp => 0x02,
        };

        // Build request (12 bytes)
        let mut request = [0u8; 12];
        request[0] = 0x00; // version
        request[1] = opcode;
        // bytes 2-3: reserved (0x00 0x00)
        request[4..6].copy_from_slice(&internal_port.to_be_bytes());
        request[6..8].copy_from_slice(&external_port.to_be_bytes());
        request[8..12].copy_from_slice(&lifetime.to_be_bytes());

        socket.send_to(&request, self.gateway).await?;

        let mut buf = [0u8; 16];
        let (len, _) = tokio::time::timeout(self.timeout, socket.recv_from(&mut buf))
            .await
            .map_err(|_| IgdError::Timeout)??;

        if len < 16 {
            return Err(IgdError::InvalidResponse(format!(
                "NAT-PMP mapping response too short: {} bytes (expected 16)",
                len
            )));
        }

        // Validate response
        // byte 0: version
        // byte 1: opcode + 128 (response flag)
        // bytes 2-3: result code
        // bytes 4-7: seconds since epoch
        // bytes 8-9: internal port
        // bytes 10-11: mapped external port
        // bytes 12-15: mapping lifetime
        let result_code = u16::from_be_bytes([buf[2], buf[3]]);

        if result_code != 0 {
            return Err(IgdError::NatPmpError(format!(
                "NAT-PMP mapping failed with code {}: {}",
                result_code,
                nat_pmp_error_description(result_code)
            )));
        }

        let resp = NatPmpMappingResponse {
            internal_port: u16::from_be_bytes([buf[8], buf[9]]),
            external_port: u16::from_be_bytes([buf[10], buf[11]]),
            lifetime: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
        };

        debug!(
            "NAT-PMP: Mapped {}:{} -> :{} (TTL {}s)",
            resp.internal_port, resp.external_port, resp.external_port, resp.lifetime
        );

        Ok(resp)
    }

    /// Delete a port mapping (set lifetime to 0)
    pub async fn delete_mapping(&self, internal_port: u16, protocol: Protocol) -> Result<()> {
        debug!("NAT-PMP: Deleting mapping for {} port {}", protocol.as_str(), internal_port);

        // Delete = map with lifetime 0 and external port 0
        let _ = self.map_port(internal_port, 0, protocol, 0).await?;

        debug!("NAT-PMP: Mapping deleted");
        Ok(())
    }

    /// Probe whether NAT-PMP is available on the gateway
    pub async fn probe(&self) -> bool {
        match self.get_external_ip().await {
            Ok(_) => {
                debug!("NAT-PMP: Gateway at {} responds", self.gateway);
                true
            }
            Err(e) => {
                trace!("NAT-PMP: Gateway at {} not available: {}", self.gateway, e);
                false
            }
        }
    }
}

/// Human-readable NAT-PMP error descriptions
fn nat_pmp_error_description(code: u16) -> &'static str {
    match code {
        0 => "Success",
        1 => "Unsupported version",
        2 => "Not authorized / refused",
        3 => "Network failure",
        4 => "Out of resources",
        5 => "Unsupported opcode",
        _ => "Unknown error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nat_pmp_error_descriptions() {
        assert_eq!(nat_pmp_error_description(0), "Success");
        assert_eq!(nat_pmp_error_description(2), "Not authorized / refused");
        assert_eq!(nat_pmp_error_description(99), "Unknown error");
    }

    #[test]
    fn test_nat_pmp_client_creation() {
        let client = NatPmpClient::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)));
        assert_eq!(client.gateway.port(), 5351);
        assert_eq!(client.gateway.ip(), IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)));
    }

    #[test]
    fn test_request_encoding() {
        // Verify port encoding is correct (big-endian)
        let port: u16 = 3492;
        let bytes = port.to_be_bytes();
        assert_eq!(bytes, [0x0D, 0xA4]);

        // Verify lifetime encoding
        let lifetime: u32 = 86400;
        let bytes = lifetime.to_be_bytes();
        assert_eq!(bytes, [0x00, 0x01, 0x51, 0x80]);
    }
}
