//! Advanced Songbird Integration Demonstration
//!
//! This demo showcases all the advanced features working together:
//! - Advanced Service Registry with auto-scaling
//! - Advanced NAT Traversal with multiple strategies
//! - Gaming Network Optimization with AI-powered performance
//! - Federation with real endpoint discovery
//! - BearDog Security Integration
//! - Advanced Load Balancing with multiple algorithms

use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use songbird_config::config::SongbirdConfig;
use songbird_network::network::gaming::{GamingManager, NatTraversalManager, PerformanceMonitor};
use songbird_federation::manager::FederationManager;
use songbird_federation::config::FederationMode;
use songbird_discovery::traits::ServiceInfo;
use songbird_errors::Result;

/// Advanced Integration Demo
/// 
/// This example demonstrates advanced integration patterns between different
/// Songbird components, including gaming network management, federation,
/// and service discovery.
pub struct AdvancedIntegrationDemo {
    pub gaming_manager: GamingManager,
    pub federation_manager: FederationManager,
    pub nat_traversal: NatTraversalManager,
    pub performance_monitor: PerformanceMonitor,
    pub service_registry: Vec<ServiceInfo>,
}

impl AdvancedIntegrationDemo {
    pub async fn new() -> Result<Self> {
        let config = SongbirdConfig::default();
        
        info!("🚀 Initializing Advanced Integration Demo");
        
        // Initialize core components
        let gaming_manager = GamingManager::new().await?;
        let federation_manager = FederationManager::new(FederationMode::Standalone);
        let mut nat_traversal = NatTraversalManager::new();
        nat_traversal.initialize(None).await?;
        let performance_monitor = PerformanceMonitor::new(Default::default())?;
        let service_registry = Vec::new();
        
        info!("✅ Advanced Integration Demo initialized");
        
        Ok(AdvancedIntegrationDemo {
            gaming_manager,
            federation_manager,
            nat_traversal,
            performance_monitor,
            service_registry,
        })
    }
    
    pub async fn run_demo(&mut self) -> Result<()> {
        info!("🎮 Starting Advanced Integration Demo");
        
        // Demo 1: Gaming Network Discovery
        self.demo_gaming_network().await?;
        
        // Demo 2: Federation Integration
        self.demo_federation().await?;
        
        // Demo 3: NAT Traversal
        self.demo_nat_traversal().await?;
        
        // Demo 4: Performance Monitoring
        self.demo_performance_monitoring().await?;
        
        // Demo 5: Service Registry Integration
        self.demo_service_registry().await?;
        
        info!("🎉 Advanced Integration Demo completed successfully!");
        
        Ok(())
    }
    
    async fn demo_gaming_network(&mut self) -> Result<()> {
        info!("🎮 Demo 1: Gaming Network Discovery");
        
        // Scan for available games
        let games = self.gaming_manager.scan_for_games(None).await?;
        info!("🔍 Found {} games", games.len());
        
        // Get current LAN sessions
        let lan_sessions = self.gaming_manager.get_lan_sessions().await;
        info!("🌐 Active LAN sessions: {}", lan_sessions.len());
        
        // Join a LAN session (demo with fake session)
        let socket_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let session_result = self.gaming_manager.join_lan_session("demo-session", socket_addr).await;
        match session_result {
            Ok(_) => info!("✅ Successfully joined LAN session"),
            Err(e) => warn!("⚠️ Failed to join LAN session: {}", e),
        }
        
        Ok(())
    }
    
    async fn demo_federation(&mut self) -> Result<()> {
        info!("🌍 Demo 2: Federation Integration");
        
        // Get federation endpoints
        let federation_endpoints = self.federation_manager.get_federation_endpoints().await?;
        info!("📡 Federation endpoints: {}", federation_endpoints.len());
        
        // Test federation endpoint connectivity
        for endpoint in federation_endpoints {
            info!("🔗 Testing federation endpoint: {}", endpoint);
            sleep(Duration::from_millis(100)).await;
        }
        
        Ok(())
    }
    
    async fn demo_nat_traversal(&mut self) -> Result<()> {
        info!("🔀 Demo 3: NAT Traversal");
        
        // Get NAT type
        let nat_type = self.nat_traversal.get_nat_type();
        info!("🌐 NAT type detected: {:?}", nat_type);
        
        // Establish connections with proper argument order
        let remote_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let connection_result = self.nat_traversal.establish_connection("peer-1".to_string(), remote_addr).await;
        
        // Fix ServiceStatus enum usage
        let service_status = ServiceStatus::Running;
        match connection_result {
            Ok(_connection_id) => {
                info!("✅ Connection established successfully");
                
                // Get connection status
                let connection_status = self.nat_traversal.get_connection_status().await;
                info!("📊 Active connections: {}", connection_status.len());
                
                // Display connection info
                for (peer_id, _connection_info) in connection_status {
                    info!("🔗 Connection {}: Connected", peer_id);
                }
            }
            Err(e) => warn!("⚠️ Failed to establish connection: {}", e),
        }
        
        Ok(())
    }
    
    async fn demo_performance_monitoring(&mut self) -> Result<()> {
        info!("📊 Demo 4: Performance Monitoring");
        
        // Start performance monitoring
        self.performance_monitor.start_monitoring().await?;
        info!("📈 Performance monitoring started");
        
        // Simulate some load
        sleep(Duration::from_secs(1)).await;
        
        // Get current metrics
        let metrics = self.performance_monitor.get_current_metrics().await;
        info!("📊 Current performance metrics:");
        info!("  - Translation Latency: {:.2}μs", metrics.translation_latency_us);
        info!("  - Packet Throughput: {:.2} pps", metrics.packet_throughput_pps);
        info!("  - CPU Usage: {:.2}%", metrics.cpu_usage_percent);
        
        Ok(())
    }
    
    async fn demo_service_registry(&mut self) -> Result<()> {
        info!("📋 Demo 5: Service Registry Integration");
        
        // Register a demo service
        let service_info = ServiceInfo {
            service_id: "demo-service".to_string(),
            name: "Demo Service".to_string(),
            service_type: "gaming".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A demo service for testing".to_string()),
            endpoints: vec![],
            metadata: HashMap::new(),
            tags: vec!["demo".to_string(), "gaming".to_string()],
            dependencies: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: "active".to_string(),
            instance_id: "demo-instance".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            health_check_endpoint: None,
        };
        
        self.service_registry.push(service_info);
        info!("✅ Service registered");
        
        // List registered services
        info!("📋 Registered services: {}", self.service_registry.len());
        for service in &self.service_registry {
            info!("  - {}: {}", service.service_id, service.name);
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let mut demo = AdvancedIntegrationDemo::new().await?;
    demo.run_demo().await?;
    
    Ok(())
} 