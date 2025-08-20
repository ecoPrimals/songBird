/// Socket Pool Module
///
/// Manages socket allocation, port management, and network resource pooling for gaming bridges
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::net::UdpSocket as TokioUdpSocket;
use tracing::{debug, warn};

use songbird_errors::{SongbirdError, SongbirdResult, success};

/// Socket pool for managing allocated ports and network resources
pub struct SocketPool {
    udp_sockets: HashMap<u16, Arc<TokioUdpSocket>>,
    tcp_listeners: HashMap<u16, Arc<TcpListener>>,
    allocated_ports: Vec<u16>,
    next_port: u16,
}

impl Default for SocketPool {
    fn default() -> Self {
        Self::new()
    }
}

impl SocketPool {
    /// Create a new socket pool
    pub fn new() -> Self {
        Self {
            udp_sockets: HashMap::new(),
            tcp_listeners: HashMap::new(),
            allocated_ports: Vec::new(),
            next_port: 50000, // Start from a high port number
        }
    }

    /// Allocate a UDP socket from the pool
    ///
    /// # Errors
    /// Returns error if socket allocation fails
    pub async fn allocate_udp_socket(&self) -> SongbirdResult<(u16, Arc<TokioUdpSocket>)> {
        // Try to find an available port
        for attempt in 0..100 {
            let port = self.next_port + attempt;
            let addr = format!("0.0.0.0:{port}");

            match TokioUdpSocket::bind(&addr).await {
                Ok(songbird_errors::evolved_success(socket)) => {
                    let socket = Arc::new(socket);
                    self.udp_sockets.insert(port, socket.clone());
                    self.allocated_ports.push(port);
                    self.next_port = port + 1;

                    debug!("Allocated UDP socket on port {}", port);
                    return Ok(songbird_errors::evolved_success(songbird_errors::success((port, socket))));
                }
                Err(e) => {
                    debug!("Port {} unavailable: {}", port, e);
                    continue;
                }
            }
        }

        Err(SongbirdError::internal_error(Network {
            message: "Unable to allocate UDP socket after 100 attempts".to_string(),
            operation: Some("udp_socket_allocation".to_string()),
            suggestion: Some("Check available ports and firewall settings".to_string()),
        })
    }

    /// Allocate a TCP listener on an available port
    pub fn allocate_tcp_listener(&mut self) -> SongbirdResult<(u16, Arc<TcpListener>)> {
        // Try to find an available port
        for attempt in 0..100 {
            let port = self.next_port + attempt;
            let addr = format!("0.0.0.0:{port}");

            match TcpListener::bind(&addr) {
                Ok(songbird_errors::evolved_success(listener)) => {
                    let listener = Arc::new(listener);
                    self.tcp_listeners.insert(port, listener.clone());
                    self.allocated_ports.push(port);
                    self.next_port = port + 1;

                    debug!("Allocated TCP listener on port {}", port);
                    return Ok(songbird_errors::evolved_success(songbird_errors::success((port, listener))));
                }
                Err(e) => {
                    debug!("Port {} unavailable: {}", port, e);
                    continue;
                }
            }
        }

        Err(SongbirdError::internal_error(Network {
            message: "Unable to allocate TCP listener after 100 attempts".to_string(),
            operation: Some("tcp_listener_allocation".to_string()),
            suggestion: Some("Check available ports and firewall settings".to_string()),
        })
    }

    /// Get an existing UDP socket by port
    pub fn get_udp_socket(&self, port: u16) -> Option<Arc<TokioUdpSocket>> {
        self.udp_sockets.get(&port).cloned()
    }

    /// Get an existing TCP listener by port
    pub fn get_tcp_listener(&self, port: u16) -> Option<Arc<TcpListener>> {
        self.tcp_listeners.get(&port).cloned()
    }

    /// Release a UDP socket
    pub fn release_udp_socket(&mut self, port: u16) -> bool {
        if self.udp_sockets.remove(&port).is_some() {
            self.allocated_ports.retain(|&p| p != port);
            debug!("Released UDP socket on port {}", port);
            true
        } else {
            warn!(
                "Attempted to release non-existent UDP socket on port {}",
                port
            );
            false
        }
    }

    /// Release a TCP listener
    pub fn release_tcp_listener(&mut self, port: u16) -> bool {
        if self.tcp_listeners.remove(&port).is_some() {
            self.allocated_ports.retain(|&p| p != port);
            debug!("Released TCP listener on port {}", port);
            true
        } else {
            warn!(
                "Attempted to release non-existent TCP listener on port {}",
                port
            );
            false
        }
    }

    /// Get all allocated ports
    pub fn allocated_ports(&self) -> &[u16] {
        &self.allocated_ports
    }

    /// Get the number of allocated UDP sockets
    pub fn udp_socket_count(&self) -> usize {
        self.udp_sockets.len()
    }

    /// Get the number of allocated TCP listeners
    pub fn tcp_listener_count(&self) -> usize {
        self.tcp_listeners.len()
    }

    /// Check if a port is allocated
    pub fn is_port_allocated(&self, port: u16) -> bool {
        self.allocated_ports.contains(&port)
    }

    /// Clear all allocated sockets (useful for cleanup)
    pub fn clear_all(&mut self) {
        let udp_count = self.udp_sockets.len();
        let tcp_count = self.tcp_listeners.len();

        self.udp_sockets.clear();
        self.tcp_listeners.clear();
        self.allocated_ports.clear();

        debug!(
            "Cleared {} UDP sockets and {} TCP listeners",
            udp_count, tcp_count
        );
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> SocketPoolStats {
        SocketPoolStats {
            total_allocated_ports: self.allocated_ports.len(),
            udp_sockets: self.udp_sockets.len(),
            tcp_listeners: self.tcp_listeners.len(),
            next_port: self.next_port,
            port_range_start: self.allocated_ports.iter().min().copied().unwrap_or(0),
            port_range_end: self.allocated_ports.iter().max().copied().unwrap_or(0),
        }
    }
}

/// Statistics for socket pool monitoring
#[derive(Debug, Clone)]
pub struct SocketPoolStats {
    pub total_allocated_ports: usize,
    pub udp_sockets: usize,
    pub tcp_listeners: usize,
    pub next_port: u16,
    pub port_range_start: u16,
    pub port_range_end: u16,
}
