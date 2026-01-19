/// Integration tests for tarpc high-performance RPC
///
/// These tests verify that the tarpc server and client work together correctly,
/// and validate the performance characteristics.
///
/// Version: 0.2.1
/// Last Updated: November 11, 2025
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tarpc::{client, context, server, tokio_serde::formats::Bincode};
use tokio::sync::RwLock;

/// Service information for registration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceInfo {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Discovery query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryQuery {
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub filters: HashMap<String, String>,
}

/// Federation status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    pub total_services: usize,
    pub total_peers: usize,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Error types for tarpc RPC calls
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ServiceError {
    #[error("Service registration failed: {0}")]
    RegistrationFailed(String),

    #[error("Service discovery failed: {0}")]
    DiscoveryFailed(String),

    #[error("Failed to get federation status: {0}")]
    StatusFailed(String),

    #[error("Stream setup failed: {0}")]
    StreamFailed(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Songbird Federation RPC Service trait
#[tarpc::service]
pub trait SongbirdFederation {
    async fn register_service(service: ServiceInfo) -> Result<String, ServiceError>;
    async fn discover_services(query: DiscoveryQuery) -> Result<Vec<ServiceInfo>, ServiceError>;
    async fn get_federation_status() -> Result<FederationStatus, ServiceError>;
    async fn health_check() -> Result<bool, ServiceError>;
}

/// Mock tarpc server for testing
#[derive(Clone)]
pub struct MockTarpcServer {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    start_time: std::time::SystemTime,
}

impl MockTarpcServer {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            start_time: std::time::SystemTime::now(),
        }
    }
}

impl SongbirdFederation for MockTarpcServer {
    async fn register_service(
        self,
        _ctx: context::Context,
        service: ServiceInfo,
    ) -> Result<String, ServiceError> {
        let service_id = format!("{}-{}", service.name, service.port);
        self.services.write().await.insert(service_id.clone(), service);
        Ok(service_id)
    }

    async fn discover_services(
        self,
        _ctx: context::Context,
        query: DiscoveryQuery,
    ) -> Result<Vec<ServiceInfo>, ServiceError> {
        let services = self.services.read().await;

        let matching_services: Vec<ServiceInfo> = services
            .values()
            .filter(|service| {
                // Check if service has all required capabilities
                query.capabilities.iter().all(|cap| service.capabilities.contains(cap))
            })
            .cloned()
            .collect();

        Ok(matching_services)
    }

    async fn get_federation_status(
        self,
        _ctx: context::Context,
    ) -> Result<FederationStatus, ServiceError> {
        let services = self.services.read().await;
        let uptime = self.start_time.elapsed().unwrap_or(Duration::from_secs(0)).as_secs();

        Ok(FederationStatus {
            total_services: services.len(),
            total_peers: 0,
            uptime_seconds: uptime,
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    async fn health_check(self, _ctx: context::Context) -> Result<bool, ServiceError> {
        Ok(true)
    }
}

/// Start a test tarpc server on a random port
async fn start_test_server() -> Result<(u16, tokio::task::JoinHandle<()>)> {
    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let port = addr.port();

    let server = MockTarpcServer::new();

    let handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let transport = Bincode::default().framed(stream);
                    let server_clone = server.clone();

                    tokio::spawn(async move {
                        server::BaseChannel::with_defaults(transport)
                            .execute(server_clone.serve())
                            .for_each(|response| async move {
                                tokio::spawn(response);
                            })
                            .await;
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    break;
                }
            }
        }
    });

    // ✅ MODERN: Wait for server to actually be ready, not arbitrary timeout
    // Try connecting to verify server is up
    let mut retries = 0;
    loop {
        if retries > 20 {
            return Err("Server failed to start within timeout".into());
        }
        match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await {
            Ok(_) => break, // Server is ready
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(5)).await;
                retries += 1;
            }
        }
    }

    Ok((port, handle))
}

/// Connect to a tarpc server
async fn connect_client(port: u16) -> Result<SongbirdFederationClient> {
    let addr = format!("127.0.0.1:{}", port);
    let transport = tarpc::serde_transport::tcp::connect(&addr, Bincode::default).await?;
    let client = SongbirdFederationClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}

#[tokio::test]
async fn test_health_check() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = connect_client(port).await?;

    let result = client.health_check(context::current()).await??;
    assert!(result, "Health check should return true");

    Ok(())
}

#[tokio::test]
async fn test_service_registration() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = connect_client(port).await?;

    let service = ServiceInfo {
        name: "test-service".to_string(),
        address: "localhost".to_string(),
        port: 3000,
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        metadata: HashMap::new(),
    };

    let service_id = client.register_service(context::current(), service.clone()).await??;

    assert_eq!(service_id, "test-service-3000");

    Ok(())
}

#[tokio::test]
async fn test_service_discovery() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = connect_client(port).await?;

    // Register multiple services
    let service1 = ServiceInfo {
        name: "ml-service".to_string(),
        address: "localhost".to_string(),
        port: 8093,
        capabilities: vec!["ml".to_string(), "training".to_string()],
        metadata: HashMap::new(),
    };

    let service2 = ServiceInfo {
        name: "storage-service".to_string(),
        address: "localhost".to_string(),
        port: 8094,
        capabilities: vec!["storage".to_string()],
        metadata: HashMap::new(),
    };

    let service3 = ServiceInfo {
        name: "compute-service".to_string(),
        address: "localhost".to_string(),
        port: 8095,
        capabilities: vec!["ml".to_string(), "compute".to_string()],
        metadata: HashMap::new(),
    };

    client.register_service(context::current(), service1.clone()).await??;
    client.register_service(context::current(), service2.clone()).await??;
    client.register_service(context::current(), service3.clone()).await??;

    // Discover services with "ml" capability
    let query = DiscoveryQuery {
        capabilities: vec!["ml".to_string()],
        filters: HashMap::new(),
    };

    let found_services = client.discover_services(context::current(), query).await??;

    assert_eq!(found_services.len(), 2, "Should find 2 services with 'ml' capability");
    assert!(found_services.iter().any(|s| s.name == "ml-service"));
    assert!(found_services.iter().any(|s| s.name == "compute-service"));

    Ok(())
}

#[tokio::test]
async fn test_federation_status() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = connect_client(port).await?;

    // Register a service
    let service = ServiceInfo {
        name: "test".to_string(),
        address: "localhost".to_string(),
        port: 3000,
        capabilities: vec!["test".to_string()],
        metadata: HashMap::new(),
    };

    client.register_service(context::current(), service).await??;

    // Get status
    let status = client.get_federation_status(context::current()).await??;

    assert_eq!(status.total_services, 1);
    assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    assert!(status.uptime_seconds >= 0);

    Ok(())
}

#[tokio::test]
async fn test_performance_latency() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = connect_client(port).await?;

    // Warm up
    for _ in 0..10 {
        let _ = client.health_check(context::current()).await;
    }

    // Measure latency over 100 calls
    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        client.health_check(context::current()).await??;
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed.as_micros() / iterations;

    println!("Average latency: {}μs per call", avg_latency);

    // tarpc should be fast! Target: < 1000μs (1ms) for local connections
    assert!(avg_latency < 1000, "Average latency should be < 1ms, got {}μs", avg_latency);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_requests() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = Arc::new(connect_client(port).await?);

    // Spawn 10 concurrent health check requests
    let mut handles = vec![];

    for i in 0..10 {
        let client_clone = Arc::clone(&client);
        let handle =
            tokio::spawn(async move { client_clone.health_check(context::current()).await });
        handles.push((i, handle));
    }

    // Wait for all to complete
    for (i, handle) in handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Request {} should succeed", i);
    }

    Ok(())
}

#[tokio::test]
async fn test_service_discovery_with_multiple_capabilities() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = connect_client(port).await?;

    // Register service with multiple capabilities
    let service = ServiceInfo {
        name: "multi-service".to_string(),
        address: "localhost".to_string(),
        port: 9000,
        capabilities: vec!["ml".to_string(), "training".to_string(), "inference".to_string()],
        metadata: HashMap::new(),
    };

    client.register_service(context::current(), service.clone()).await??;

    // Search for services with multiple required capabilities
    let query = DiscoveryQuery {
        capabilities: vec!["ml".to_string(), "training".to_string()],
        filters: HashMap::new(),
    };

    let found = client.discover_services(context::current(), query).await??;

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "multi-service");

    // Search with a capability the service doesn't have
    let query2 = DiscoveryQuery {
        capabilities: vec!["ml".to_string(), "nonexistent".to_string()],
        filters: HashMap::new(),
    };

    let not_found = client.discover_services(context::current(), query2).await??;
    assert_eq!(not_found.len(), 0, "Should not find service without all capabilities");

    Ok(())
}

#[tokio::test]
async fn test_multiple_service_registrations() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = connect_client(port).await?;

    // Register 5 different services
    for i in 0..5 {
        let service = ServiceInfo {
            name: format!("service-{}", i),
            address: "localhost".to_string(),
            port: 8000 + i,
            capabilities: vec![format!("cap-{}", i)],
            metadata: HashMap::new(),
        };

        let service_id = client.register_service(context::current(), service).await??;
        assert_eq!(service_id, format!("service-{}-{}", i, 8000 + i));
    }

    // Verify total count
    let status = client.get_federation_status(context::current()).await??;
    assert_eq!(status.total_services, 5);

    Ok(())
}

#[tokio::test]
async fn test_throughput() -> Result<()> {
    let (port, _handle) = start_test_server().await?;
    let client = connect_client(port).await?;

    // Warm up
    for _ in 0..10 {
        let _ = client.health_check(context::current()).await;
    }

    // Measure throughput: how many requests in 1 second
    let duration = Duration::from_secs(1);
    let start = Instant::now();
    let mut count = 0;

    while start.elapsed() < duration {
        client.health_check(context::current()).await??;
        count += 1;
    }

    let requests_per_sec = count;
    println!("Throughput: {} requests/second", requests_per_sec);

    // Should handle at least 1000 requests/second locally
    assert!(requests_per_sec >= 1000, "Should handle >= 1000 req/s, got {}", requests_per_sec);

    Ok(())
}
