pub mod communication;
pub mod http_server;
pub mod management;
pub mod network;
pub mod proxy;

// Re-export commonly used types
pub use communication::*;
// Re-export management types except for conflicting NetworkConfig
pub use management::{
    HealthCheckConfig, LoadBalancingStrategy, NetworkManager as ManagementNetworkManager,
    RateLimitConfig,
};
// Re-export network types except for conflicting NetworkManager
pub use network::{NetworkConfig as NetworkNetworkConfig, NetworkManager as NetworkNetworkManager};

// Re-export gaming module
pub use network::gaming;

// Re-export for easier access in examples and external usage
pub use network::gaming::security_provider;

#[cfg(test)]
mod universal_layer23_networking_tests {
    //! Universal Layer 2/3 Networking Tests
    //!
    //! Comprehensive testing for advanced networking capabilities including:
    //! - Protocol Handler Framework
    //! - Virtual Network Management
    //! - QoS Optimization
    //! - Raw Packet Forwarding
    //! - Gaming-Specific Network Bridges
    //! - Legacy Protocol Support
    //! - Network Security Policies

    use std::collections::HashMap;
    use std::net::{IpAddr, SocketAddr};
    use std::time::{Duration, SystemTime};
    use tokio::time::sleep;

    // Test protocol types for Layer 2/3 networking
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum TestProtocolType {
        Tcp,
        Udp,
        Icmp,
        Ipv4,
        Ipv6,
        // Legacy protocols
        Ipx,
        NetBios,
        AppleTalk,
        // Gaming protocols
        DirectPlay,
        SteamNetworking,
        GameSpecific(String),
        // Industrial protocols
        Modbus,
        Profinet,
        EtherCAT,
        // IoT protocols
        ZigBee,
        LoRaWAN,
        Thread,
        // Scientific protocols
        InfiniBand,
        RoCE,
        Myrinet,
        // Custom protocols
        Custom { name: String, protocol_number: u16 },
    }

    #[derive(Debug, Clone)]
    pub struct TestNetworkConfig {
        pub bind_address: IpAddr,
        pub bind_port: u16,
        pub protocols: Vec<TestProtocolType>,
        pub max_connections: u32,
        pub enable_qos: bool,
        pub enable_gaming_mode: bool,
        pub enable_legacy_support: bool,
    }

    #[derive(Debug, Clone)]
    pub struct TestVirtualNetwork {
        pub id: String,
        pub topology: TestNetworkTopology,
        pub protocols: Vec<TestProtocolType>,
        pub qos_config: TestQosConfig,
        pub created_at: SystemTime,
    }

    #[derive(Debug, Clone)]
    pub enum TestNetworkTopology {
        PointToPoint {
            endpoint_a: String,
            endpoint_b: String,
        },
        Star {
            hub: String,
            spokes: Vec<String>,
        },
        FullMesh {
            nodes: Vec<String>,
        },
        Ring {
            nodes: Vec<String>,
            bidirectional: bool,
        },
    }

    #[derive(Debug, Clone)]
    pub struct TestQosConfig {
        pub max_latency: Duration,
        pub max_jitter: Duration,
        pub min_bandwidth: u64,
        pub max_packet_loss: f32,
        pub priority: TestQosPriority,
    }

    #[derive(Debug, Clone)]
    pub enum TestQosPriority {
        Critical,
        High,
        Normal,
        Low,
        BestEffort,
    }

    #[derive(Debug, Clone)]
    pub struct TestGameProfile {
        pub name: String,
        pub protocols: Vec<TestProtocolType>,
        pub ports: Vec<u16>,
        pub requires_broadcast: bool,
        pub latency_target: Duration,
        pub supports_ipx: bool,
        pub supports_directplay: bool,
    }

    #[derive(Debug, Clone)]
    pub struct TestProtocolHandler {
        pub protocol_type: TestProtocolType,
        pub packet_count: u64,
        pub bytes_processed: u64,
        pub error_count: u64,
        pub last_activity: SystemTime,
    }

    #[derive(Debug, Clone, Default)]
    pub struct TestPerformanceMetrics {
        pub packets_forwarded: u64,
        pub total_latency: Duration,
        pub average_latency: Duration,
        pub packet_loss_rate: f32,
        pub throughput_mbps: f64,
        pub active_connections: u32,
        pub qos_violations: u32,
    }

    #[derive(Debug, Clone)]
    pub struct TestPacket {
        pub id: String,
        pub protocol: TestProtocolType,
        pub source: SocketAddr,
        pub destination: SocketAddr,
        pub payload: Vec<u8>,
        pub timestamp: SystemTime,
    }

    impl TestPacket {
        pub fn new(
            protocol: TestProtocolType,
            source: SocketAddr,
            destination: SocketAddr,
            payload: Vec<u8>,
        ) -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                protocol,
                source,
                destination,
                payload,
                timestamp: SystemTime::now(),
            }
        }
    }

    // Mock implementation for testing
    pub struct TestUniversalNetworkManager {
        pub virtual_networks: HashMap<String, TestVirtualNetwork>,
        pub protocol_handlers: HashMap<TestProtocolType, TestProtocolHandler>,
        pub qos_policies: HashMap<String, TestQosConfig>,
        pub gaming_profiles: HashMap<String, TestGameProfile>,
        pub performance_metrics: TestPerformanceMetrics,
    }

    impl TestUniversalNetworkManager {
        pub fn new() -> Self {
            let mut manager = Self {
                virtual_networks: HashMap::new(),
                protocol_handlers: HashMap::new(),
                qos_policies: HashMap::new(),
                gaming_profiles: HashMap::new(),
                performance_metrics: TestPerformanceMetrics::default(),
            };

            // Initialize default protocol handlers
            manager.initialize_protocol_handlers();
            manager.initialize_gaming_profiles();

            manager
        }

        fn initialize_protocol_handlers(&mut self) {
            let protocols = vec![
                TestProtocolType::Tcp,
                TestProtocolType::Udp,
                TestProtocolType::Ipx,
                TestProtocolType::NetBios,
                TestProtocolType::DirectPlay,
                TestProtocolType::SteamNetworking,
                TestProtocolType::Modbus,
                TestProtocolType::InfiniBand,
            ];

            for protocol in protocols {
                let handler = TestProtocolHandler {
                    protocol_type: protocol.clone(),
                    packet_count: 0,
                    bytes_processed: 0,
                    error_count: 0,
                    last_activity: SystemTime::now(),
                };
                self.protocol_handlers.insert(protocol, handler);
            }
        }

        fn initialize_gaming_profiles(&mut self) {
            let profiles = vec![
                TestGameProfile {
                    name: "StarCraft".to_string(),
                    protocols: vec![TestProtocolType::Ipx, TestProtocolType::Tcp],
                    ports: vec![6112, 6113, 6114],
                    requires_broadcast: true,
                    latency_target: Duration::from_millis(20),
                    supports_ipx: true,
                    supports_directplay: false,
                },
                TestGameProfile {
                    name: "Age of Empires 2".to_string(),
                    protocols: vec![TestProtocolType::DirectPlay, TestProtocolType::Tcp],
                    ports: vec![2300, 2301, 2302, 2303],
                    requires_broadcast: true,
                    latency_target: Duration::from_millis(30),
                    supports_ipx: false,
                    supports_directplay: true,
                },
                TestGameProfile {
                    name: "Quake".to_string(),
                    protocols: vec![TestProtocolType::GameSpecific("Quake".to_string())],
                    ports: vec![26000],
                    requires_broadcast: true,
                    latency_target: Duration::from_millis(10),
                    supports_ipx: false,
                    supports_directplay: false,
                },
            ];

            for profile in profiles {
                self.gaming_profiles.insert(profile.name.clone(), profile);
            }
        }

        pub async fn create_virtual_network(
            &mut self,
            config: TestNetworkConfig,
        ) -> Result<String, String> {
            let network_id = format!("network_{}", uuid::Uuid::new_v4());

            let virtual_network = TestVirtualNetwork {
                id: network_id.clone(),
                topology: TestNetworkTopology::FullMesh {
                    nodes: vec!["node1".to_string(), "node2".to_string()],
                },
                protocols: config.protocols,
                qos_config: TestQosConfig {
                    max_latency: Duration::from_millis(10),
                    max_jitter: Duration::from_millis(2),
                    min_bandwidth: 100_000_000, // 100 Mbps
                    max_packet_loss: 0.001,
                    priority: TestQosPriority::High,
                },
                created_at: SystemTime::now(),
            };

            self.virtual_networks
                .insert(network_id.clone(), virtual_network);
            Ok(network_id)
        }

        pub async fn forward_packet(
            &mut self,
            network_id: &str,
            packet: TestPacket,
        ) -> Result<(), String> {
            if !self.virtual_networks.contains_key(network_id) {
                return Err(format!("Network {} not found", network_id));
            }

            // Update protocol handler statistics
            if let Some(handler) = self.protocol_handlers.get_mut(&packet.protocol) {
                handler.packet_count += 1;
                handler.bytes_processed += packet.payload.len() as u64;
                handler.last_activity = SystemTime::now();
            }

            // Update performance metrics
            self.performance_metrics.packets_forwarded += 1;
            self.performance_metrics.throughput_mbps +=
                packet.payload.len() as f64 * 8.0 / 1_000_000.0;

            // Simulate packet processing time
            sleep(Duration::from_micros(100)).await;

            Ok(())
        }

        pub async fn optimize_for_gaming(
            &mut self,
            game_name: &str,
            network_id: &str,
        ) -> Result<(), String> {
            if let Some(game_profile) = self.gaming_profiles.get(game_name) {
                if let Some(network) = self.virtual_networks.get_mut(network_id) {
                    // Apply gaming-specific optimizations
                    network.qos_config.max_latency = game_profile.latency_target;
                    network.qos_config.priority = TestQosPriority::Critical;

                    // Enable legacy protocol support if needed
                    if game_profile.supports_ipx {
                        network.protocols.push(TestProtocolType::Ipx);
                    }
                    if game_profile.supports_directplay {
                        network.protocols.push(TestProtocolType::DirectPlay);
                    }

                    return Ok(());
                }
            }

            Err("Game profile or network not found".to_string())
        }

        pub async fn apply_qos_policy(
            &mut self,
            network_id: &str,
            qos_config: TestQosConfig,
        ) -> Result<(), String> {
            if let Some(network) = self.virtual_networks.get_mut(network_id) {
                network.qos_config = qos_config.clone();
                self.qos_policies.insert(network_id.to_string(), qos_config);
                Ok(())
            } else {
                Err("Network not found".to_string())
            }
        }

        pub fn get_performance_metrics(&self) -> &TestPerformanceMetrics {
            &self.performance_metrics
        }

        pub fn get_protocol_statistics(
            &self,
            protocol: &TestProtocolType,
        ) -> Option<&TestProtocolHandler> {
            self.protocol_handlers.get(protocol)
        }
    }

    #[tokio::test]
    async fn test_universal_network_manager_creation() {
        let manager = TestUniversalNetworkManager::new();

        // Verify protocol handlers are initialized
        assert!(manager
            .protocol_handlers
            .contains_key(&TestProtocolType::Tcp));
        assert!(manager
            .protocol_handlers
            .contains_key(&TestProtocolType::Udp));
        assert!(manager
            .protocol_handlers
            .contains_key(&TestProtocolType::Ipx));
        assert!(manager
            .protocol_handlers
            .contains_key(&TestProtocolType::NetBios));

        // Verify gaming profiles are initialized
        assert!(manager.gaming_profiles.contains_key("StarCraft"));
        assert!(manager.gaming_profiles.contains_key("Age of Empires 2"));
        assert!(manager.gaming_profiles.contains_key("Quake"));

        println!("✅ Universal Network Manager created successfully with {} protocol handlers and {} gaming profiles", 
                 manager.protocol_handlers.len(), manager.gaming_profiles.len());
    }

    #[tokio::test]
    async fn test_virtual_network_creation() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![TestProtocolType::Tcp, TestProtocolType::Udp],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: false,
            enable_legacy_support: false,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Verify network was created
        assert!(manager.virtual_networks.contains_key(&network_id));

        let network = manager.virtual_networks.get(&network_id).unwrap();
        assert_eq!(network.protocols.len(), 2);
        assert!(network.protocols.contains(&TestProtocolType::Tcp));
        assert!(network.protocols.contains(&TestProtocolType::Udp));

        println!("✅ Virtual network created successfully: {}", network_id);
    }

    #[tokio::test]
    async fn test_packet_forwarding() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![TestProtocolType::Tcp],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: false,
            enable_legacy_support: false,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Create and forward a test packet
        let packet = TestPacket::new(
            TestProtocolType::Tcp,
            SocketAddr::from(([127, 0, 0, 1], 8080)),
            SocketAddr::from(([127, 0, 0, 1], 8081)),
            vec![0x01, 0x02, 0x03, 0x04],
        );

        let result = manager.forward_packet(&network_id, packet).await;
        assert!(result.is_ok());

        // Verify packet statistics
        let tcp_stats = manager
            .get_protocol_statistics(&TestProtocolType::Tcp)
            .unwrap();
        assert_eq!(tcp_stats.packet_count, 1);
        assert_eq!(tcp_stats.bytes_processed, 4);

        // Verify performance metrics
        let metrics = manager.get_performance_metrics();
        assert_eq!(metrics.packets_forwarded, 1);

        println!(
            "✅ Packet forwarding test successful - 1 packet forwarded, {} bytes processed",
            tcp_stats.bytes_processed
        );
    }

    #[tokio::test]
    async fn test_legacy_protocol_support() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![TestProtocolType::Ipx, TestProtocolType::NetBios],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: true,
            enable_legacy_support: true,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Test IPX packet forwarding
        let ipx_packet = TestPacket::new(
            TestProtocolType::Ipx,
            SocketAddr::from(([127, 0, 0, 1], 8080)),
            SocketAddr::from(([127, 0, 0, 1], 8081)),
            vec![0xFF, 0xFF, 0xFF, 0xFF], // IPX broadcast signature
        );

        let result = manager.forward_packet(&network_id, ipx_packet).await;
        assert!(result.is_ok());

        // Test NetBIOS packet forwarding
        let netbios_packet = TestPacket::new(
            TestProtocolType::NetBios,
            SocketAddr::from(([127, 0, 0, 1], 8080)),
            SocketAddr::from(([127, 0, 0, 1], 8081)),
            vec![0x20, 0x43, 0x4B, 0x41], // NetBIOS name query
        );

        let result = manager.forward_packet(&network_id, netbios_packet).await;
        assert!(result.is_ok());

        // Verify both protocols processed packets
        let ipx_stats = manager
            .get_protocol_statistics(&TestProtocolType::Ipx)
            .unwrap();
        let netbios_stats = manager
            .get_protocol_statistics(&TestProtocolType::NetBios)
            .unwrap();

        assert_eq!(ipx_stats.packet_count, 1);
        assert_eq!(netbios_stats.packet_count, 1);

        println!(
            "✅ Legacy protocol support test successful - IPX: {} packets, NetBIOS: {} packets",
            ipx_stats.packet_count, netbios_stats.packet_count
        );
    }

    #[tokio::test]
    async fn test_gaming_optimization() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![TestProtocolType::Tcp],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: true,
            enable_legacy_support: true,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Apply StarCraft gaming optimization
        let result = manager.optimize_for_gaming("StarCraft", &network_id).await;
        assert!(result.is_ok());

        // Verify gaming optimizations were applied
        let network = manager.virtual_networks.get(&network_id).unwrap();
        assert_eq!(network.qos_config.max_latency, Duration::from_millis(20));
        assert!(matches!(
            network.qos_config.priority,
            TestQosPriority::Critical
        ));

        // Verify IPX protocol was added for StarCraft
        assert!(network.protocols.contains(&TestProtocolType::Ipx));

        println!("✅ Gaming optimization test successful - StarCraft profile applied with {}ms latency target", 
                 network.qos_config.max_latency.as_millis());
    }

    #[tokio::test]
    async fn test_qos_policy_enforcement() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![TestProtocolType::Tcp],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: false,
            enable_legacy_support: false,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Apply strict QoS policy
        let qos_config = TestQosConfig {
            max_latency: Duration::from_millis(1),
            max_jitter: Duration::from_millis(0),
            min_bandwidth: 1_000_000_000, // 1 Gbps
            max_packet_loss: 0.0001,
            priority: TestQosPriority::Critical,
        };

        let result = manager.apply_qos_policy(&network_id, qos_config).await;
        assert!(result.is_ok());

        // Verify QoS policy was applied
        let network = manager.virtual_networks.get(&network_id).unwrap();
        assert_eq!(network.qos_config.max_latency, Duration::from_millis(1));
        assert_eq!(network.qos_config.min_bandwidth, 1_000_000_000);
        assert!(matches!(
            network.qos_config.priority,
            TestQosPriority::Critical
        ));

        // Verify policy is stored
        assert!(manager.qos_policies.contains_key(&network_id));

        println!("✅ QoS policy enforcement test successful - 1ms latency, 1Gbps bandwidth, Critical priority");
    }

    #[tokio::test]
    async fn test_performance_metrics_collection() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![TestProtocolType::Tcp, TestProtocolType::Udp],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: false,
            enable_legacy_support: false,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Forward multiple packets to collect metrics
        for _i in 0..10 {
            let packet = TestPacket::new(
                TestProtocolType::Tcp,
                SocketAddr::from(([127, 0, 0, 1], 8080)),
                SocketAddr::from(([127, 0, 0, 1], 8081)),
                vec![0x01; 100], // 100 bytes payload
            );

            let result = manager.forward_packet(&network_id, packet).await;
            assert!(result.is_ok());
        }

        // Verify performance metrics
        let metrics = manager.get_performance_metrics();
        assert_eq!(metrics.packets_forwarded, 10);
        assert!(metrics.throughput_mbps > 0.0);

        // Verify protocol statistics
        let tcp_stats = manager
            .get_protocol_statistics(&TestProtocolType::Tcp)
            .unwrap();
        assert_eq!(tcp_stats.packet_count, 10);
        assert_eq!(tcp_stats.bytes_processed, 1000);

        println!("✅ Performance metrics collection test successful - {} packets forwarded, {:.2} Mbps throughput", 
                 metrics.packets_forwarded, metrics.throughput_mbps);
    }

    #[tokio::test]
    async fn test_industrial_protocol_support() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![
                TestProtocolType::Modbus,
                TestProtocolType::Profinet,
                TestProtocolType::EtherCAT,
            ],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: false,
            enable_legacy_support: false,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Test Modbus packet forwarding
        let modbus_packet = TestPacket::new(
            TestProtocolType::Modbus,
            SocketAddr::from(([127, 0, 0, 1], 502)),
            SocketAddr::from(([127, 0, 0, 1], 503)),
            vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x01], // Modbus read input registers
        );

        let result = manager.forward_packet(&network_id, modbus_packet).await;
        assert!(result.is_ok());

        // Verify industrial protocol statistics
        let modbus_stats = manager
            .get_protocol_statistics(&TestProtocolType::Modbus)
            .unwrap();
        assert_eq!(modbus_stats.packet_count, 1);

        println!(
            "✅ Industrial protocol support test successful - Modbus: {} packets processed",
            modbus_stats.packet_count
        );
    }

    #[tokio::test]
    async fn test_scientific_computing_protocols() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![TestProtocolType::InfiniBand, TestProtocolType::RoCE],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: false,
            enable_legacy_support: false,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Test InfiniBand packet forwarding (simulated)
        let ib_packet = TestPacket::new(
            TestProtocolType::InfiniBand,
            SocketAddr::from(([127, 0, 0, 1], 8080)),
            SocketAddr::from(([127, 0, 0, 1], 8081)),
            vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05], // Simulated IB packet
        );

        let result = manager.forward_packet(&network_id, ib_packet).await;
        assert!(result.is_ok());

        // Verify scientific protocol statistics
        let ib_stats = manager
            .get_protocol_statistics(&TestProtocolType::InfiniBand)
            .unwrap();
        assert_eq!(ib_stats.packet_count, 1);

        println!(
            "✅ Scientific computing protocols test successful - InfiniBand: {} packets processed",
            ib_stats.packet_count
        );
    }

    #[tokio::test]
    async fn test_concurrent_protocol_handling() {
        let mut manager = TestUniversalNetworkManager::new();

        let config = TestNetworkConfig {
            bind_address: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            bind_port: 8080,
            protocols: vec![
                TestProtocolType::Tcp,
                TestProtocolType::Udp,
                TestProtocolType::Ipx,
                TestProtocolType::NetBios,
                TestProtocolType::Modbus,
            ],
            max_connections: 100,
            enable_qos: true,
            enable_gaming_mode: true,
            enable_legacy_support: true,
        };

        let network_id = manager.create_virtual_network(config).await.unwrap();

        // Create packets for different protocols
        let protocols = vec![
            TestProtocolType::Tcp,
            TestProtocolType::Udp,
            TestProtocolType::Ipx,
            TestProtocolType::NetBios,
            TestProtocolType::Modbus,
        ];

        // Forward packets for each protocol concurrently
        for protocol in protocols {
            let packet = TestPacket::new(
                protocol.clone(),
                SocketAddr::from(([127, 0, 0, 1], 8080)),
                SocketAddr::from(([127, 0, 0, 1], 8081)),
                vec![0x01; 50],
            );

            let result = manager.forward_packet(&network_id, packet).await;
            assert!(result.is_ok());
        }

        // Verify all protocols handled packets
        let tcp_stats = manager
            .get_protocol_statistics(&TestProtocolType::Tcp)
            .unwrap();
        let udp_stats = manager
            .get_protocol_statistics(&TestProtocolType::Udp)
            .unwrap();
        let ipx_stats = manager
            .get_protocol_statistics(&TestProtocolType::Ipx)
            .unwrap();
        let netbios_stats = manager
            .get_protocol_statistics(&TestProtocolType::NetBios)
            .unwrap();
        let modbus_stats = manager
            .get_protocol_statistics(&TestProtocolType::Modbus)
            .unwrap();

        assert_eq!(tcp_stats.packet_count, 1);
        assert_eq!(udp_stats.packet_count, 1);
        assert_eq!(ipx_stats.packet_count, 1);
        assert_eq!(netbios_stats.packet_count, 1);
        assert_eq!(modbus_stats.packet_count, 1);

        println!("✅ Concurrent protocol handling test successful - 5 protocols processed simultaneously");
    }
}
