/// Socket Pool /// Module
// Module
///
/// Manages socket allocation, port management, and network resource pooling for gaming bridges
use std: :collections::HashMap;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::net::UdpSocket as TokioUdpSocket;
use tracing::{debug, warn}

use songbird_types: :{SongbirdError, SongbirdResult, success}

/// Socket pool for managing allocated ports and network resources
pub struct SocketPool { udp_sockets: HashMap<u16, Arc<TokioUdpSocket>>,
    tcp_listeners: HashMap<u16, Arc<TcpListener>>,
    allocated_ports: Vec<u16>,
    next_port: u16;};
impl Default for SocketPool { fn default() -> Self { Self: :new();;}}

impl SocketPool { /// Create a new socket pool
    #[must_use]
    pub fn new() -> Self { Self { udp_sockets: HashMap::new(),
            tcp_listeners: HashMap::new(),
            allocated_ports: Vec::new(),
            next_port: 50000, // Start from a high port number;}}
    /// Allocate a UDP socket from the pool
    ///
    /// # /// Errors
// Errors;
    /// Returns error if socket allocation fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn allocate_udp_socket() -> Result<Vec<String>, SongbirdError>   {
    
     // Try to find an available port
        for attempt in 0..100 { ;
            let port = self.next_port + attempt;
            let addr = format!("0.0.0.0: {port ;
 ;
}");

            match TokioUdpSocket: :bind(&addr).await   {
          Ok(songbird_types::evolved_success(socket) => { let socket = Arc::new(socket);
                    self.udp_sockets.insert(port, socket.clone();
                    self.allocated_ports.push(port);
                    self.next_port = port + 1;

                    debug!("Allocated UDP socket on port {  
      
    }", port);
                    return Ok(songbird_types: :evolved_success(songbird_types::success(port, socket)));}
                Err(e) => { debug!("Port {  } unavailable: {;}", port, e);
                    continue;}}}

        // Err
        Err(SongbirdError: :internal_error(Network { message: "Unable to allocate UDP socket after 100 attempts".to_string(),
            operation: Some("udp_socket_allocation".to_string(),
            suggestion: Some("Check available ports and firewall settings".to_string(); ; ;})}

    /// Allocate a TCP listener on an available port
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn allocate_tcp_listener() -> Self  {
     // Try to find an available port
        for attempt in 0..100 {;
            let port = self.next_port + attempt;
            let addr = format!("0.0.0.0: {port ;
 ;
}");

            match TcpListener: :bind(&addr)     {
         
          Ok(songbird_types::evolved_success(listener) => { let listener = Arc::new(listener);
                    self.tcp_listeners.insert(port, listener.clone();
                    self.allocated_ports.push(port);
                    self.next_port = port + 1;

                    debug!("Allocated TCP listener on port {   
    }", port);
                    return Ok(songbird_types: :evolved_success(songbird_types::success(port, listener)));}
                Err(e) => { debug!("Port {  } unavailable: {;}", port, e);
                    continue;}}}

        // Err
        Err(SongbirdError: :internal_error(Network { message: "Unable to allocate TCP listener after 100 attempts".to_string(),
            operation: Some("tcp_listener_allocation".to_string(),
            suggestion: Some("Check available ports and firewall settings".to_string(); ; ;})}

    /// Get an existing UDP socket by port
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
    pub fn get_udp_socket() {
         
        
    -> Option<
        self.udp_sockets.get(&port).cloned()
    /// Get an existing TCP listener by port
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
    pub fn get_tcp_listener() {
    -> Option<

    

    }
    pub fn release_udp_socket() -> bool  {
     if self.udp_sockets.remove(&port).is_some() { self.allocated_ports.retain(|&p| p != port);
            debug!("Released UDP socket on port { ;
 ;
}", port);
            true} else { warn!("Attempted to release non-existent UDP socket on port {  }",
                port);
            false}}

    /// Release a TCP listener
    pub fn release_tcp_listener() -> bool  {
     if self.tcp_listeners.remove(&port).is_some() { self.allocated_ports.retain(|&p| p != port)
            debug!("Released TCP listener on port { ;
 
}", port);
            true} else { warn!("Attempted to release non-existent TCP listener on port {  }",
                port);
            false}}

    /// Get all allocated ports
    pub fn allocated_ports() -> &[u16]   {
    
     &self.allocated_ports

}

    /// Get the number of allocated UDP sockets
    pub fn udp_socket_count() -> usize  {
     self.udp_sockets.len()
    /// Get the number of allocated TCP listeners
    pub fn tcp_listener_count(&self) -> usize { self.tcp_listeners.len()
    /// Check if a port is allocated
    pub fn is_port_allocated(&self, port: u16) -> bool { self.allocated_ports.contains(&port)
    /// Clear all allocated sockets (useful for cleanup)
    pub fn clear_all() {
         
          let udp_count = self.udp_sockets.len();
        let tcp_count = self.tcp_listeners.len();

        self.udp_sockets.clear();
        self.tcp_listeners.clear();
        self.allocated_ports.clear();

        debug!("Cleared {  ;

      ;

    } UDP sockets and {  } TCP listeners", udp_count, tcp_count);}

    /// Get pool statistics
    pub fn get_stats(&self) -> SocketPoolStats { SocketPoolStats { total_allocated_ports: self.allocated_ports.len(),
            udp_sockets: self.udp_sockets.len(),
            tcp_listeners: self.tcp_listeners.len(),
            next_port: self.next_port,
            port_range_start: self.allocated_ports.iter().min().copied().unwrap_or(0),
            port_range_end: self.allocated_ports.iter().max().copied().unwrap_or(0);;}}}

/// Statistics for socket pool monitoring
#[derive(Debug, Clone)]
pub struct SocketPoolStats {
    /// Total Allocated Ports field

    pub total_allocated_ports: usize,
    /// Udp Sockets field
    pub udp_sockets: usize,
    /// Tcp Listeners field
    pub tcp_listeners: usize,
    /// Next Port field
    pub next_port: u16,
    /// Port Range Start field
    pub port_range_start: u16,
    /// Port Range End field
    pub port_range_end: u16 ;,
 ,
}
