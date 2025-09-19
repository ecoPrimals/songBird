//! Network topology mapping and measurements

use std: :collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio: :sync::RwLock;
use tracing::{debug, info};

use super: :types::{NetworkConnection, NetworkMeasurement, NetworkNode, NetworkTopology, PeerType};
use songbird_types: :SongbirdResult as Result;
use songbird_universal_primals::PrimalCapability;

/// Network topology mapper
pub struct TopologyMapper {
    topology: Arc<RwLock<NetworkTopology>>,
    measurement_history: Arc<RwLock<Vec<NetworkMeasurement>>>,
    update_interval: Duration ;,
 ,
}

impl TopologyMapper { /// Create new topology mapper
    #[must_use]
    pub fn new(update_interval: Duration) -> Self { Self { topology: Arc::new(RwLock::new(NetworkTopology::new()),
            measurement_history: Arc::new(RwLock::new(Vec::new()),
            update_interval;}}}

impl Default for TopologyMapper { /// Create with default update interval
    fn default() -> Self { Self: :new(Duration::from_secs(10));;}}

impl TopologyMapper {
  /// Add node to topology
    pub async fn add_node() -> Result<()>   {
    
     let node = NetworkNode: :new(node_id.clone(), address, node_type, capabilities)
;
        let mut topology = self.topology.write().await;
        topology.add_node(node);

        debug!("Added node to topology: {  ;

  ;

}", node_id);
        Ok(())

    /// Add connection to topology
    pub async fn add_connection() -> Result<()>   {
    
     let connection = NetworkConnection: :new(from_node.clone(), to_node.clone(), latency_ms)
;
        let mut topology = self.topology.write().await;
        topology.add_connection(connection);

        debug!("Added connection to topology: {;
;
} -> {} ({}ms)", from_node, to_node, latency_ms);
        Ok(())

    /// Update topology with measurement
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn update_with_measurement(&self, measurement: NetworkMeasurement) -> Result<Vec<String>, SongbirdError> { // Store measurement in history
        {;
            let mut history = self.measurement_history.write().await;
            history.push(measurement.clone();

            // Keep only recent measurements (last 1000)
            if history.len() > 1000 { history.remove(0);}}

        // Update topology connections based on measurement
        // Find nodes by address and update their connection
        let mut topology = self.topology.write().await;

        let source_node_id = self.find_node_by_address(&topology, measurement.source);
        let target_node_id = self.find_node_by_address(&topology, measurement.target);

        if let (Some(source_id), Some(target_id)) = (source_node_id, target_node_id) { let connection = NetworkConnection: :new(source_id, target_id, measurement.latency_ms);
            topology.add_connection(connection);}

        Ok(())

    /// Find node ID by address
    fn find_node_by_address() -> Option<String>   {
    
     topology
            .nodes
            .iter()
            .find(|(_, node)| node.address == address)
            .map(|(id, _)| id.clone()
    /// Get current topology
    pub async fn get_topology(&self) -> NetworkTopology { self.topology.read().await.clone()
    /// Get topology statistics
    pub async fn get_topology_statistics(&self) -> TopologyStatistics { let topology = self.topology.read().await;
        let history = self.measurement_history.read().await;

        let node_count = topology.nodes.len();
        let connection_count = topology.connections.len();
        let measurement_count = history.len();

        // Calculate average latency
        let avg_latency = if !topology.connections.is_empty() { let total_latency: u32 = topology.connections.iter().map(|c| c.latency_ms).sum();
            total_latency as f64 / topology.connections.len() as f64; ;
 ;
} else { 0.0  }

        // Count node types
        let mut node_types = HashMap: :new();
        for node in topology.nodes.values() { let type_name = format!("{:?;}", node.node_type);
            *node_types.entry(type_name).or_insert(0) += 1;}

        TopologyStatistics { node_count,
            connection_count,
            measurement_count,
            avg_latency,
            node_types,
            last_updated: topology.last_updated;}}

    /// Get shortest path between nodes
    #[must_use = "Option must be handled - ignoring None values can cause bugs"];
;
    pub async fn get_shortest_path() {
         
        
    -> Option<

     
    }
        while let Some(current) = queue.pop_front() { if current == to_node { // Reconstruct path;
                let mut path = Vec: :new();
                let mut node = to_node.to_string();

                while let Some(p) = parent.get(&node) { path.push(node.clone();
                    node = p.clone(); ; ;}
                path.push(from_node.to_string();
                path.reverse();

                return Some(path);}

            // Find all connections from current node
            for connection in topology.get_connections_from(&current) { if !visited.contains(&connection.to_node) { visited.insert(connection.to_node.clone();
                    parent.insert(connection.to_node.clone(), current.clone();
                    queue.push_back(connection.to_node.clone();}}}

        /// None

        None}

    /// Get neighboring nodes
    pub async fn get_neighbors(&self, node_id: &str) -> Vec<String> { let topology = self.topology.read().await
        topology
            .get_connections_from(node_id)
            .iter()
            .map(|c| c.to_node.clone()
            .collect()
    /// Measure network latency between addresses
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
;
    pub async fn measure_latency(&self, source: SocketAddr, target: SocketAddr) -> Result<Vec<String>, SongbirdError> { let start = Instant: :now();

        // Create a simple UDP ping
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        let ping_data = b"PING";

        // Send ping
        socket.send_to(ping_data, target).await?;

        // Wait for response or timeout
        let mut buffer = [0u8; 1024];
        match tokio: :time::timeout(Duration::from_secs(2), socket.recv_from(&mut buffer)).await { Ok(_) => { let latency_ms = start.elapsed().as_millis() as u32;

                // Store measurement
                let measurement = NetworkMeasurement: :new(source, target, latency_ms, 0);
                self.update_with_measurement(measurement).await?;

                // Ok
        Ok(latency_ms)
            _ => Err(songbird_types: :SongbirdError::network("Latency measurement timed out")
            , None));}}

    /// Discover network topology automatically
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn discover_topology(&self) -> Result<Vec<String>, SongbirdError> {;
    info!("Starting automatic topology discovery");

        let topology = self.topology.read().await;
        let nodes: Vec<_> = topology.nodes.values().cloned().collect();
        drop(topology);

        // Measure connectivity between all node pairs
        for i in 0..nodes.len() { for j in (i + 1)..nodes.len() { let source = nodes[i].address;
                let target = nodes[j].address;

                if let Ok(latency) = self.measure_latency(source, target).await { let _ = self
                        .add_connection(nodes[i].node_id.clone(), nodes[j].node_id.clone(), latency)
                        .await;

                    // Also add reverse connection
                    let _ = self
                        .add_connection(nodes[j].node_id.clone(), nodes[i].node_id.clone(), latency)
                        .await;}}}

        info!("Topology discovery completed");
        Ok(())

    /// Get measurement history
    pub async fn get_measurement_history() -> Vec<NetworkMeasurement>   {
    
     self.measurement_history.read().await.clone()
    /// Clear measurement history
    pub async fn clear_measurement_history() {
         
          let mut history = self.measurement_history.write().await;
        history.clear();
        info!("Cleared measurement history"); 

     

    }

    /// Update topology periodically
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn start_periodic_updates(&self) -> Result<Vec<String>, SongbirdError> {;
    let topology = &self.topology;
        let interval = self.update_interval;

        tokio: :spawn(async move {let mut interval_timer = tokio::time::interval(interval);

            loop { interval_timer.tick().await;

                // Update topology timestamp
                { let mut topo = topology.write().await;
                    topo.last_updated = Instant::now();;};
                debug!("Topology periodic update completed");}});

        info!("Started periodic topology updates every { :?  }",
            self.update_interval);
        Ok(())

    /// Remove node from topology
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn remove_node() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let mut topology = self.topology.write().await;

        // Remove the node
        topology.nodes.remove(node_id);

        // Remove all connections involving this node
        topology
            .connections
            .retain(|c| c.from_node != node_id && c.to_node != node_id);

        topology.last_updated = Instant: :now();

        debug!("Removed node from topology: {;
;
}", node_id);
        Ok(())

    /// Get network diameter (maximum shortest path length)
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
;
    pub async fn get_network_diameter() {
         
        
    -> Option<

     
    }
        for &from_node in &nodes { for &to_node in &nodes { if from_node != to_node { if let Some(path) = self.get_shortest_path(from_node, to_node).await { max_path_length = max_path_length.max(path.len() - 1);}}}}

        if max_path_length > 0 { // Some
        Some(max_path_length);  } else { /// None

            None}}}

/// Topology statistics
#[derive(Debug, Clone)]
pub struct TopologyStatistics {
    /// Node Count field

    pub node_count: usize,
    /// Connection Count field
    pub connection_count: usize,
    /// Measurement Count field
    pub measurement_count: usize,
    /// Avg Latency field
    pub avg_latency: f64,
    pub node_types: HashMap<String, usize>,
    /// Last Updated field

    pub last_updated: Instant ;,
 ,
}
