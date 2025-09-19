//! Canonical Network Example
//!
//! Demonstrates the canonical patterns for network operations

use songbird_errors::{SongbirdError, SongbirdResult, success, unified::success_result};
use std::net::SocketAddr;
use std::time::Instant;

/// Network health information
#[derive(Debug, Clone)]
pub struct NetworkHealth {
    pub interface: String,
    pub status: NetworkStatus,
    pub latency_ms: Option<u64>,
    pub packet_loss_percent: f64,
}

/// Network status enumeration
#[derive(Debug, Clone)]
pub enum NetworkStatus {
    Healthy,
    Degraded,
    Unreachable,
}

/// Canonical network health checker
pub struct CanonicalNetworkChecker;

impl CanonicalNetworkChecker {
    /// Check network health with canonical response pattern
    ///
    /// This demonstrates the canonical pattern:
    /// - AI-first metadata
    /// - Performance tracking
    /// - Automation hints
    /// - Confidence scoring
    /// - Suggested actions
    pub async fn check_network_health(&self) -> SongbirdResult<SongbirdResponse<NetworkHealth>> {
        let _start_time = Instant::now();

        // Simulate network health check
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Simulate different health scenarios
        let health = match interface {
            "eth0" => NetworkHealth {
                interface: interface.to_string(),
                status: NetworkStatus::Healthy,
                latency_ms: Some(5),
                packet_loss_percent: 0.1,
            },
            "wlan0" => NetworkHealth {
                interface: interface.to_string(),
                status: NetworkStatus::Degraded,
                latency_ms: Some(45),
                packet_loss_percent: 2.3,
            },
            "broken" => {
                // Demonstrate canonical error handling
                return Err(SongbirdError::internal_error(network_error(format!(
                    "Network interface '{interface}' is unreachable"
                )));
            }
            _ => NetworkHealth {
                interface: interface.to_string(),
                status: NetworkStatus::Healthy,
                latency_ms: Some(15),
                packet_loss_percent: 0.5,
            },
        };

        // Create canonical response based on health status
        let response = match health.status {
            NetworkStatus::Healthy => success_result(health),
            NetworkStatus::Degraded => success_result(health),
            NetworkStatus::Unreachable => success_result(health),
        };

        Ok(songbird_errors::evolved_success(response))
    }

    /// Scan for available network interfaces
    ///
    /// Demonstrates canonical pattern for collection operations
    pub async fn scan_interfaces(&self) -> SongbirdResult<SongbirdResponse<Vec<String>>> {
        let _start_time = Instant::now();

        // Simulate interface discovery
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

        let interfaces = vec!["eth0".to_string(), "wlan0".to_string(), "lo".to_string()];

        Ok(songbird_errors::evolved_success(success_result(interfaces)))
    }

    /// Test network connectivity to a specific endpoint
    ///
    /// Demonstrates canonical error handling with detailed context
    pub async fn test_connectivity(&self) -> SongbirdResult<SongbirdResponse<bool>> {
        let _start_time = Instant::now();

        // Simulate connectivity test
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

        // Simulate different connectivity scenarios
        let is_reachable = match endpoint.port() {
            80 | 443 => true, // Web ports usually work
            22 => true,       // SSH usually works
            8080 => false,    // Development port might be blocked
            _ => true,        // Assume others work
        };

        if is_reachable {
            Ok(songbird_errors::evolved_success(success_result(true)))
        } else {
            Ok(songbird_errors::evolved_success(success_result(false)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_canonical_network_health_check() {
        let checker = CanonicalNetworkChecker;

        // Test healthy interface
        let result = checker.check_network_health("eth0").await;
        assert!(result.is_ok());

        let response = result
            .map_err(|e| SongbirdError::network(format!("Test operation failed: {e}")))?;
        assert!(response.confidence > 0.9);
        assert!(matches!(response.data.status, NetworkStatus::Healthy));

        // Test error case
        let error_result = checker.check_network_health("broken").await;
        assert!(error_result.is_err());
    }

    #[tokio::test]
    async fn test_canonical_interface_scan() {
        let checker = CanonicalNetworkChecker;

        let result = checker.scan_interfaces().await;
        assert!(result.is_ok());

        let response = result
            .map_err(|e| SongbirdError::network(format!("Test operation failed: {e}")))?;
        assert!(!response.data.is_empty());
        assert!(response.confidence > 0.8);
    }
}
