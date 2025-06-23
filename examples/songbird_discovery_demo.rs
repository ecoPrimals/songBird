use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use chrono::Utc;

use songbird_orchestrator::{
    discovery::{
        SongbirdDiscovery, SongbirdDiscoveryConfig, 
        ResourceQuery, TrustLevel, ResourceUsage,
        NodeInfo, NodeType, ComputeResources,
        types::{NetworkLocation, StorageInfo, DatasetInfo, DatasetType,
               AccessLevel, GpuInfo, StorageDevice, StoragePerformanceTier}
    },
    traits::discovery::ServiceDiscovery,
    traits::service::ServiceInfo,
};

async fn demonstrate_songbird_discovery() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎼 Songbird Discovery Service Demo");
    println!("==================================");
    println!("🧬 Scientific Computing Federation");
    println!("🔍 Resource-Aware Service Discovery");
    println!("🌐 Multi-Institutional Support");
    println!("");

    // Create Songbird Discovery with federation enabled
    let config = SongbirdDiscoveryConfig {
        node_id: Some("demo-node".to_string()),
        node_type: songbird_orchestrator::discovery::NodeType::Orchestrator,
        institution: Some("Demo University".to_string()),
        federation_enabled: true,
        health_check_interval_secs: 30,
        node_discovery_interval_secs: 60,
        trust_verification_enabled: true,
        max_federation_nodes: 100,
        network: songbird_orchestrator::discovery::NetworkConfig::default(),
        monitoring: songbird_orchestrator::discovery::MonitoringConfig::default(),
        trust: songbird_orchestrator::discovery::TrustConfig::default(),
    };

    let discovery = SongbirdDiscovery::new(config);
    
    // Start federation services
    discovery.start_federation().await?;

    println!("📊 Local Node Information:");
    let local_node = discovery.local_node();
    println!("   ID: {}", local_node.id);
    println!("   Type: {:?}", local_node.node_type);
    println!("   Institution: {:?}", local_node.institution);
    println!("   CPU Cores: {}", local_node.resources.cpu_cores);
    println!("   Memory: {} GB", local_node.resources.memory_total_gb);
    println!("");

    // Register sample federation nodes
    println!("🌐 Registering Federation Nodes...");
    
    // High-performance compute node
    let compute_node = NodeInfo {
        id: "hpc-cluster-01".to_string(),
        address: "hpc01.mit.edu:8080".to_string(),
        node_type: NodeType::Compute,
        institution: Some("MIT".to_string()),
        resources: ComputeResources {
            cpu_cores: 128,
            cpu_architecture: "x86_64".to_string(),
            memory_total_gb: 512,
            memory_available_gb: 256,
            gpu_info: vec![
                GpuInfo {
                    model: "NVIDIA A100".to_string(),
                    memory_gb: 80,
                    compute_capability: Some("8.0".to_string()),
                    utilization_percent: 25.0,
                },
                GpuInfo {
                    model: "NVIDIA A100".to_string(),
                    memory_gb: 80,
                    compute_capability: Some("8.0".to_string()),
                    utilization_percent: 30.0,
                },
            ],
            storage_devices: vec![
                StorageDevice {
                    device_type: "NVMe".to_string(),
                    capacity_gb: 2000,
                    available_gb: 1500,
                    mount_point: "/scratch".to_string(),
                    performance_tier: StoragePerformanceTier::HighPerformance,
                }
            ],
            network_bandwidth_mbps: 100_000.0, // 100 Gbps
        },
        current_load: ResourceUsage {
            cpu_utilization_percent: 45.0,
            memory_used_gb: 256,
            gpu_utilization: vec![25.0, 30.0],
            storage_used_gb: 500,
            network_utilization_percent: 15.0,
            active_jobs: 12,
        },
        available_datasets: vec![
            DatasetInfo {
                id: "human-genome-ref".to_string(),
                name: "Human Reference Genome GRCh38".to_string(),
                dataset_type: DatasetType::Genomic,
                size_bytes: 3_200_000_000, // ~3.2 GB
                format: "FASTA".to_string(),
                checksum: "sha256:abc123...".to_string(),
                access_level: AccessLevel::Public,
                last_updated: Utc::now(),
            }
        ],
        storage_capacity: StorageInfo {
            total_capacity_gb: 50_000,
            available_capacity_gb: 35_000,
            performance_tier_breakdown: {
                let mut breakdown = HashMap::new();
                breakdown.insert(StoragePerformanceTier::HighPerformance, 10_000);
                breakdown.insert(StoragePerformanceTier::Standard, 25_000);
                breakdown.insert(StoragePerformanceTier::Archive, 15_000);
                breakdown
            },
        },
        trust_level: TrustLevel::Institutional,
        reputation_score: 0.95,
        network_location: NetworkLocation {
            region: "us-east-1".to_string(),
            institution: Some("MIT".to_string()),
            subnet: Some("10.0.1.0/24".to_string()),
            external_ip: Some("18.62.1.100".to_string()),
            internal_ip: Some("10.0.1.100".to_string()),
        },
        bandwidth_measurements: HashMap::new(),
        latency_measurements: HashMap::new(),
        last_seen: Utc::now(),
        health_status: songbird_orchestrator::traits::discovery::ServiceHealthStatus::Healthy,
        services: vec!["genomics-pipeline".to_string(), "protein-folding".to_string()],
    };

    discovery.register_node(compute_node).await?;

    // Storage-focused node
    let storage_node = NodeInfo {
        id: "data-lake-01".to_string(),
        address: "storage01.harvard.edu:8080".to_string(),
        node_type: NodeType::Storage,
        institution: Some("Harvard".to_string()),
        resources: ComputeResources {
            cpu_cores: 32,
            cpu_architecture: "x86_64".to_string(),
            memory_total_gb: 128,
            memory_available_gb: 64,
            gpu_info: Vec::new(),
            storage_devices: vec![
                StorageDevice {
                    device_type: "Archive".to_string(),
                    capacity_gb: 1_000_000, // 1 PB
                    available_gb: 750_000,
                    mount_point: "/data".to_string(),
                    performance_tier: StoragePerformanceTier::Archive,
                }
            ],
            network_bandwidth_mbps: 40_000.0, // 40 Gbps
        },
        current_load: ResourceUsage {
            cpu_utilization_percent: 15.0,
            memory_used_gb: 64,
            gpu_utilization: Vec::new(),
            storage_used_gb: 250_000,
            network_utilization_percent: 35.0,
            active_jobs: 3,
        },
        available_datasets: vec![
            DatasetInfo {
                id: "tcga-cancer-genome".to_string(),
                name: "TCGA Cancer Genome Atlas".to_string(),
                dataset_type: DatasetType::Genomic,
                size_bytes: 2_500_000_000_000, // 2.5 TB
                format: "BAM/VCF".to_string(),
                checksum: "sha256:def456...".to_string(),
                access_level: AccessLevel::Consortium,
                last_updated: Utc::now(),
            },
            DatasetInfo {
                id: "uk-biobank-imaging".to_string(),
                name: "UK Biobank Medical Imaging".to_string(),
                dataset_type: DatasetType::Imaging,
                size_bytes: 10_000_000_000_000, // 10 TB
                format: "DICOM".to_string(),
                checksum: "sha256:ghi789...".to_string(),
                access_level: AccessLevel::Institutional,
                last_updated: Utc::now(),
            }
        ],
        storage_capacity: StorageInfo {
            total_capacity_gb: 1_000_000,
            available_capacity_gb: 750_000,
            performance_tier_breakdown: {
                let mut breakdown = HashMap::new();
                breakdown.insert(StoragePerformanceTier::Archive, 1_000_000);
                breakdown
            },
        },
        trust_level: TrustLevel::Consortium,
        reputation_score: 0.88,
        network_location: NetworkLocation {
            region: "us-east-1".to_string(),
            institution: Some("Harvard".to_string()),
            subnet: Some("10.1.0.0/24".to_string()),
            external_ip: Some("128.103.1.50".to_string()),
            internal_ip: Some("10.1.0.50".to_string()),
        },
        bandwidth_measurements: HashMap::new(),
        latency_measurements: HashMap::new(),
        last_seen: Utc::now(),
        health_status: songbird_orchestrator::traits::discovery::ServiceHealthStatus::Healthy,
        services: vec!["data-repository".to_string(), "backup-service".to_string()],
    };

    discovery.register_node(storage_node).await?;

    // Gateway node (connects institutions)
    let gateway_node = NodeInfo {
        id: "inter-institutional-gateway".to_string(),
        address: "gateway.science-federation.org:8080".to_string(),
        node_type: NodeType::Gateway,
        institution: Some("NIH".to_string()),
        resources: ComputeResources {
            cpu_cores: 64,
            cpu_architecture: "x86_64".to_string(),
            memory_total_gb: 256,
            memory_available_gb: 128,
            gpu_info: Vec::new(),
            storage_devices: vec![
                StorageDevice {
                    device_type: "SSD".to_string(),
                    capacity_gb: 5000,
                    available_gb: 3000,
                    mount_point: "/cache".to_string(),
                    performance_tier: StoragePerformanceTier::Standard,
                }
            ],
            network_bandwidth_mbps: 200_000.0, // 200 Gbps
        },
        current_load: ResourceUsage {
            cpu_utilization_percent: 25.0,
            memory_used_gb: 128,
            gpu_utilization: Vec::new(),
            storage_used_gb: 2000,
            network_utilization_percent: 60.0, // High network usage
            active_jobs: 8,
        },
        available_datasets: Vec::new(), // Gateway doesn't store data
        storage_capacity: StorageInfo {
            total_capacity_gb: 5_000,
            available_capacity_gb: 3_000,
            performance_tier_breakdown: {
                let mut breakdown = HashMap::new();
                breakdown.insert(StoragePerformanceTier::Standard, 5_000);
                breakdown
            },
        },
        trust_level: TrustLevel::Consortium,
        reputation_score: 0.92,
        network_location: NetworkLocation {
            region: "us-central".to_string(),
            institution: Some("NIH".to_string()),
            subnet: Some("192.168.0.0/16".to_string()),
            external_ip: Some("129.43.1.1".to_string()),
            internal_ip: Some("192.168.1.1".to_string()),
        },
        bandwidth_measurements: HashMap::new(),
        latency_measurements: HashMap::new(),
        last_seen: Utc::now(),
        health_status: songbird_orchestrator::traits::discovery::ServiceHealthStatus::Healthy,
        services: vec!["routing-service".to_string(), "auth-proxy".to_string()],
    };

    discovery.register_node(gateway_node).await?;

    println!("✅ Registered 3 federation nodes");
    
    // Wait for registration to complete
    sleep(Duration::from_millis(500)).await;

    // Test resource-aware discovery
    println!("\n🔍 Testing Resource-Aware Discovery...");
    
    // Query 1: Find high-performance compute nodes
    println!("\n📊 Query 1: High-Performance Compute Nodes");
    let compute_query = ResourceQuery {
        min_cpu_cores: Some(64),
        min_memory_gb: Some(256),
        required_node_type: Some(NodeType::Compute),
        min_trust_level: TrustLevel::Institutional,
        ..Default::default()
    };

    let compute_nodes = discovery.find_optimal_nodes(compute_query).await?;
    println!("   Found {} matching compute nodes:", compute_nodes.len());
    for node in &compute_nodes {
        println!("   - {} ({:?}) - {} cores, {} GB RAM, {} GPUs", 
                node.id, node.node_type, node.resources.cpu_cores, 
                node.resources.memory_total_gb, node.resources.gpu_info.len());
    }

    // Query 2: Find storage nodes with specific datasets
    println!("\n💾 Query 2: Storage Nodes");
    let storage_query = ResourceQuery {
        required_node_type: Some(NodeType::Storage),
        min_trust_level: TrustLevel::Consortium,
        ..Default::default()
    };

    let storage_nodes = discovery.find_optimal_nodes(storage_query).await?;
    println!("   Found {} storage nodes:", storage_nodes.len());
    for node in &storage_nodes {
        println!("   - {} ({}) - {} datasets, {} GB total storage", 
                node.id, node.institution.as_deref().unwrap_or("Unknown"),
                node.available_datasets.len(), node.storage_capacity.total_capacity_gb);
        for dataset in &node.available_datasets {
            println!("     * {} ({:?}) - {} - {:.2} GB", 
                    dataset.name, dataset.dataset_type, dataset.format,
                    dataset.size_bytes as f64 / 1_000_000_000.0);
        }
    }

    // Query 3: Find gateway nodes for inter-institutional routing
    println!("\n🌐 Query 3: Gateway Nodes");
    let gateway_query = ResourceQuery {
        required_node_type: Some(NodeType::Gateway),
        min_trust_level: TrustLevel::Consortium,
        ..Default::default()
    };

    let gateway_nodes = discovery.find_optimal_nodes(gateway_query).await?;
    println!("   Found {} gateway nodes:", gateway_nodes.len());
    for node in &gateway_nodes {
        println!("   - {} ({}) - {:.0} Gbps network, {:.1}% utilization", 
                node.id, node.institution.as_deref().unwrap_or("Unknown"),
                node.resources.network_bandwidth_mbps / 1000.0,
                node.current_load.network_utilization_percent);
    }

    // Register some scientific services
    println!("\n🧪 Registering Scientific Services...");
    
    let genomics_service = ServiceInfo {
        id: "genomics-pipeline".to_string(),
        name: "Genomics Analysis Pipeline".to_string(),
        version: "2.1.0".to_string(),
        service_type: "scientific-computing".to_string(),
        description: "High-throughput genomics analysis pipeline".to_string(),
        endpoints: vec![],
        capabilities: vec![
            "variant-calling".to_string(),
            "genome-assembly".to_string(),
            "annotation".to_string(),
        ],
        tags: {
            let mut tags = HashMap::new();
            tags.insert("domain".to_string(), "genomics".to_string());
            tags.insert("gpu-accelerated".to_string(), "true".to_string());
            tags.insert("min-memory-gb".to_string(), "64".to_string());
            tags
        },
        metadata: HashMap::new(),
    };

    discovery.register(genomics_service).await?;

    let protein_service = ServiceInfo {
        id: "protein-folding".to_string(),
        name: "Protein Structure Prediction".to_string(),
        version: "1.5.0".to_string(),
        service_type: "scientific-computing".to_string(),
        description: "AI-powered protein structure prediction".to_string(),
        endpoints: vec![],
        capabilities: vec![
            "alphafold".to_string(),
            "structure-prediction".to_string(),
            "molecular-dynamics".to_string(),
        ],
        tags: {
            let mut tags = HashMap::new();
            tags.insert("domain".to_string(), "structural-biology".to_string());
            tags.insert("ai-accelerated".to_string(), "true".to_string());
            tags.insert("min-gpu-memory-gb".to_string(), "40".to_string());
            tags
        },
        metadata: HashMap::new(),
    };

    discovery.register(protein_service).await?;

    println!("✅ Registered 2 scientific computing services");

    // Get federation statistics
    println!("\n📈 Federation Statistics:");
    let stats = discovery.get_federation_stats().await;
    println!("   Total Nodes: {}", stats.total_nodes);
    println!("   Total Services: {}", stats.total_services);
    println!("   Compute Nodes: {}", stats.compute_nodes);
    println!("   Storage Nodes: {}", stats.storage_nodes);
    println!("   Gateway Nodes: {}", stats.gateway_nodes);
    println!("   Total CPU Cores: {}", stats.total_cpu_cores);
    println!("   Total Memory: {} GB", stats.total_memory_gb);
    println!("   Total Storage: {} TB", stats.total_storage_gb / 1000);

    // Test service discovery
    println!("\n🔍 Service Discovery Test:");
    let services = discovery.list_all().await?;
    println!("   Found {} local services:", services.len());
    for service in &services {
        println!("   - {} v{} ({})", service.name, service.version, service.service_type);
        println!("     Capabilities: {:?}", service.capabilities);
        if !service.tags.is_empty() {
            println!("     Tags: {:?}", service.tags);
        }
    }

    println!("\n🎉 Songbird Discovery Demo Complete!");
    println!("✅ Federation nodes registered and discoverable");
    println!("✅ Resource-aware node selection working");
    println!("✅ Scientific service discovery functional");
    println!("✅ Trust-based filtering operational");
    println!("✅ Multi-institutional support demonstrated");

    Ok(())
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    demonstrate_songbird_discovery().await
} 