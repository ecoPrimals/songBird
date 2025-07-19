//! Network monitoring, statistics, and diagnostics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Total bytes received
    pub bytes_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total requests processed
    pub requests_processed: u64,
    /// Active connections
    pub active_connections: u64,
    /// Total connections
    pub total_connections: u64,
}

/// Network health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthStatus {
    /// Overall health status
    pub status: HealthStatus,
    /// Health check timestamp
    pub last_check: SystemTime,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded
    Degraded,
    /// Service is unhealthy
    Unhealthy,
}

/// Network diagnostics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnostics {
    /// Network interface statistics
    pub interfaces: HashMap<String, InterfaceStats>,
    /// Connection statistics
    pub connections: ConnectionStats,
    /// System uptime
    pub uptime: Duration,
    /// Load averages
    pub load_average: (f64, f64, f64),
}

/// Interface statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStats {
    /// Interface name
    pub name: String,
    /// Bytes received
    pub rx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Interface is up
    pub is_up: bool,
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// Active TCP connections
    pub tcp_active: u64,
    /// Passive TCP connections
    pub tcp_passive: u64,
    /// Failed TCP connections
    pub tcp_failed: u64,
    /// TCP connections reset
    pub tcp_resets: u64,
    /// Current established connections
    pub tcp_established: u64,
    /// UDP packets received
    pub udp_in_packets: u64,
    /// UDP packets sent
    pub udp_out_packets: u64,
}

impl NetworkStats {
    /// Create new empty network statistics
    pub fn new() -> Self {
        Self {
            bytes_received: 0,
            bytes_sent: 0,
            requests_processed: 0,
            active_connections: 0,
            total_connections: 0,
        }
    }

    /// Update bytes received
    pub fn add_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }

    /// Update bytes sent
    pub fn add_bytes_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
    }

    /// Increment request count
    pub fn increment_requests(&mut self) {
        self.requests_processed += 1;
    }

    /// Set active connections
    pub fn set_active_connections(&mut self, count: u64) {
        self.active_connections = count;
    }

    /// Increment total connections
    pub fn increment_total_connections(&mut self) {
        self.total_connections += 1;
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkHealthStatus {
    /// Create new health status
    pub fn new(status: HealthStatus) -> Self {
        Self {
            status,
            last_check: SystemTime::now(),
        }
    }

    /// Update health status
    pub fn update_status(&mut self, status: HealthStatus) {
        self.status = status;
        self.last_check = SystemTime::now();
    }

    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }

    /// Check if degraded
    pub fn is_degraded(&self) -> bool {
        self.status == HealthStatus::Degraded
    }

    /// Check if unhealthy
    pub fn is_unhealthy(&self) -> bool {
        self.status == HealthStatus::Unhealthy
    }
}

impl NetworkDiagnostics {
    /// Create new network diagnostics
    pub fn new() -> Self {
        Self {
            interfaces: HashMap::new(),
            connections: ConnectionStats::default(),
            uptime: Duration::from_secs(0),
            load_average: (0.0, 0.0, 0.0),
        }
    }

    /// Add interface statistics
    pub fn add_interface(&mut self, name: String, stats: InterfaceStats) {
        self.interfaces.insert(name, stats);
    }

    /// Update system uptime
    pub fn set_uptime(&mut self, uptime: Duration) {
        self.uptime = uptime;
    }

    /// Update load average
    pub fn set_load_average(&mut self, load: (f64, f64, f64)) {
        self.load_average = load;
    }

    /// Get total received bytes across all interfaces
    pub fn total_rx_bytes(&self) -> u64 {
        self.interfaces.values().map(|i| i.rx_bytes).sum()
    }

    /// Get total transmitted bytes across all interfaces
    pub fn total_tx_bytes(&self) -> u64 {
        self.interfaces.values().map(|i| i.tx_bytes).sum()
    }

    /// Get number of active interfaces
    pub fn active_interfaces(&self) -> usize {
        self.interfaces.values().filter(|i| i.is_up).count()
    }
}

impl Default for NetworkDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl InterfaceStats {
    /// Create new interface statistics
    pub fn new(name: String) -> Self {
        Self {
            name,
            rx_bytes: 0,
            rx_packets: 0,
            rx_errors: 0,
            tx_bytes: 0,
            tx_packets: 0,
            tx_errors: 0,
            is_up: false,
        }
    }

    /// Calculate receive error rate
    pub fn rx_error_rate(&self) -> f64 {
        if self.rx_packets == 0 {
            0.0
        } else {
            self.rx_errors as f64 / self.rx_packets as f64
        }
    }

    /// Calculate transmit error rate
    pub fn tx_error_rate(&self) -> f64 {
        if self.tx_packets == 0 {
            0.0
        } else {
            self.tx_errors as f64 / self.tx_packets as f64
        }
    }

    /// Check if interface has errors
    pub fn has_errors(&self) -> bool {
        self.rx_errors > 0 || self.tx_errors > 0
    }
}

impl ConnectionStats {
    /// Create new connection statistics
    pub fn new() -> Self {
        Self {
            tcp_active: 0,
            tcp_passive: 0,
            tcp_failed: 0,
            tcp_resets: 0,
            tcp_established: 0,
            udp_in_packets: 0,
            udp_out_packets: 0,
        }
    }

    /// Calculate TCP failure rate
    pub fn tcp_failure_rate(&self) -> f64 {
        let total_attempts = self.tcp_active + self.tcp_passive;
        if total_attempts == 0 {
            0.0
        } else {
            self.tcp_failed as f64 / total_attempts as f64
        }
    }

    /// Get total TCP connections attempted
    pub fn total_tcp_attempts(&self) -> u64 {
        self.tcp_active + self.tcp_passive
    }
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self::new()
    }
}
