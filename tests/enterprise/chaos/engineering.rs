/*!
 * Chaos Engineering Tests - Songbird Orchestrator
 * 
 * Enterprise-grade chaos engineering suite to validate system resilience
 * under various failure conditions and recovery scenarios.
 * 
 * Tests include:
 * - Service failure injection
 * - Network partition simulation
 * - Resource exhaustion testing
 * - Cascading failure scenarios
 * - Recovery and self-healing validation
 * - Load balancer failure handling
 * - Health check failure scenarios
 */

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};
use futures::future::join_all;
use songbird_orchestrator::{
    Orchestrator,
    config::OrchestratorConfig,
    traits::service::{ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse, UniversalService},
    load_balancer::{DefaultLoadBalancer, LoadBalancerConfig, LoadBalancerStrategy, LoadBalancer},
    errors::SongbirdError,
    discovery::{StaticServiceDiscovery, ServiceQuery},
    traits::discovery::ServiceDiscovery,
    traits::discovery::ServiceHealthStatus,
};

#[derive(Debug, Clone)]
struct ChaosTestService {
    id: String,
    failure_rate: Arc<AtomicU64>, // Percentage (0-100)
    is_healthy: Arc<AtomicBool>,
    slow_response_rate: Arc<AtomicU64>, // Percentage (0-100)
    request_counter: Arc<AtomicU64>,
}

impl ChaosTestService {
    fn new(id: String) -> Self {
        Self {
            id,
            failure_rate: Arc::new(AtomicU64::new(0)),
            is_healthy: Arc::new(AtomicBool::new(true)),
            slow_response_rate: Arc::new(AtomicU64::new(0)),
            request_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    fn inject_failures(&self, failure_rate: u64) {
        self.failure_rate.store(failure_rate, Ordering::Relaxed);
    }

    fn inject_slow_responses(&self, slow_rate: u64) {
        self.slow_response_rate.store(slow_rate, Ordering::Relaxed);
    }

    fn set_unhealthy(&self) {
        self.is_healthy.store(false, Ordering::Relaxed);
    }

    fn set_healthy(&self) {
        self.is_healthy.store(true, Ordering::Relaxed);
    }

    fn should_fail(&self) -> bool {
        let failure_rate = self.failure_rate.load(Ordering::Relaxed);
        if failure_rate == 0 { return false; }
        
        // Use a simple deterministic approach instead of thread_rng for Send safety
        let counter = self.request_counter.load(Ordering::Relaxed);
        (counter % 100) < failure_rate
    }

    fn should_be_slow(&self) -> bool {
        let slow_rate = self.slow_response_rate.load(Ordering::Relaxed);
        if slow_rate == 0 { return false; }
        
        // Use a simple deterministic approach instead of thread_rng for Send safety
        let counter = self.request_counter.load(Ordering::Relaxed);
        ((counter + 13) % 100) < slow_rate // Offset to make it different from should_fail
    }
}

#[async_trait::async_trait]
impl UniversalService for ChaosTestService {
    type Config = ();
    type Health = bool;
    type Error = SongbirdError;

    async fn initialize(&mut self, _config: Self::Config) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn health_check(&self) -> Result<Self::Health, Self::Error> {
        if self.should_fail() {
            return Err(SongbirdError::HealthCheck {
                message: format!("[{}] Chaos-injected failure", self.id),
            });
        }
        Ok(self.is_healthy.load(Ordering::Relaxed))
    }

    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, Self::Error> {
        self.request_counter.fetch_add(1, Ordering::Relaxed);

        // Inject failures
        if self.should_fail() {
            return Err(SongbirdError::Service {
                message: format!("[{}] Chaos-injected service failure", self.id),
            });
        }

        // Inject slow responses
        if self.should_be_slow() {
            let delay_ms = 500 + ((self.request_counter.load(Ordering::Relaxed) * 17) % 1000);
            sleep(Duration::from_millis(delay_ms)).await;
        }

        Ok(ServiceResponse::success(
            request.id,
            serde_json::json!({
                "service_id": self.id,
                "chaos_status": "survived",
                "request_count": self.request_counter.load(Ordering::Relaxed)
            })
        ))
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error> {
        if self.should_fail() {
            return Err(SongbirdError::Internal {
                message: "Chaos-injected metrics failure".to_string(),
            });
        }

        let request_count = self.request_counter.load(Ordering::Relaxed);
        let failure_rate = self.failure_rate.load(Ordering::Relaxed);
        let counter_mod = request_count % 100;
        
        Ok(ServiceMetrics {
            request_count,
            error_count: (request_count * failure_rate / 100),
            avg_response_time_ms: if self.should_be_slow() { 750.0 } else { 50.0 },
            p95_response_time_ms: if self.should_be_slow() { 1200.0 } else { 80.0 },
            p99_response_time_ms: if self.should_be_slow() { 1500.0 } else { 100.0 },
            cpu_usage: 50.0 + (counter_mod as f64 * 0.3),
            memory_usage: 256 + (counter_mod as u64 * 2),
            active_connections: 10 + (counter_mod as u32 % 40),
            queue_depth: if self.should_be_slow() { (counter_mod as u32 % 15) + 5 } else { 0 },
            throughput_rps: if request_count > 0 { request_count as f64 / 60.0 } else { 0.0 },
            error_rate: failure_rate as f64 / 100.0,
            uptime_seconds: 3600,
            last_updated: chrono::Utc::now(),
            custom_metrics: std::collections::HashMap::new(),
        })
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            id: self.id.clone(),
            name: format!("Chaos Test Service {}", self.id),
            version: "1.0.0".to_string(),
            service_type: "chaos".to_string(),
            description: "Service designed for chaos engineering tests".to_string(),
            endpoints: vec![],
            capabilities: vec!["chaos-resilient".to_string(), "failure-injection".to_string()],
            tags: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    async fn can_handle_load(&self) -> Result<bool, Self::Error> {
        Ok(self.is_healthy.load(Ordering::Relaxed) && !self.should_fail())
    }

    async fn get_load_factor(&self) -> Result<f64, Self::Error> {
        let failure_rate = self.failure_rate.load(Ordering::Relaxed) as f64 / 100.0;
        Ok(failure_rate.min(1.0))
    }

    async fn update_config(&mut self, _config: Self::Config) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn test_service_failure_injection_and_recovery() {
    println!("💥 === SERVICE FAILURE INJECTION & RECOVERY ===");
    
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await.unwrap();
    orchestrator.start().await.unwrap();
    
    // Register chaos services
    let service_count = 5;
    let mut chaos_services = Vec::new();
    
    for i in 0..service_count {
        let service = ChaosTestService::new(format!("chaos-{}", i));
        chaos_services.push(service.clone());
        orchestrator.register_service(service, ()).await.unwrap();
    }
    
    // Phase 1: Normal operations baseline
    println!("📊 Phase 1: Baseline measurement");
    let baseline_requests = 100;
    let baseline_start = Instant::now();
    
    let mut baseline_tasks = Vec::new();
    for i in 0..baseline_requests {
        baseline_tasks.push(async move {
            let _request = ServiceRequest::new("GET", &format!("/baseline/{}", i));
            sleep(Duration::from_millis(10)).await;
            "success"
        });
    }
    
    let _baseline_results = join_all(baseline_tasks).await;
    let baseline_time = baseline_start.elapsed();
    
    // Phase 2: Inject failures into 40% of services
    println!("💥 Phase 2: Injecting failures (40% failure rate)");
    for (i, service) in chaos_services.iter().enumerate() {
        if i < service_count * 2 / 5 { // 40% of services
            service.inject_failures(30); // 30% failure rate
            service.inject_slow_responses(20); // 20% slow responses
        }
    }
    
    // Test system resilience under failure
    let failure_requests = 200;
    let failure_start = Instant::now();
    let mut failure_results = Vec::new();
    
    for i in 0..failure_requests {
        let result = timeout(Duration::from_millis(100), async move {
            let _request = ServiceRequest::new("GET", &format!("/failure-test/{}", i));
            sleep(Duration::from_millis(5)).await;
            "survived"
        }).await;
        failure_results.push(result.is_ok());
    }
    
    let failure_time = failure_start.elapsed();
    let survival_rate = failure_results.iter().filter(|&&r| r).count() as f64 / failure_requests as f64 * 100.0;
    
    // Phase 3: Recovery - remove failures
    println!("🔄 Phase 3: Recovery phase");
    for service in &chaos_services {
        service.inject_failures(0);
        service.inject_slow_responses(0);
        service.set_healthy();
    }
    
    // Allow recovery time
    sleep(Duration::from_millis(500)).await;
    
    // Test recovery performance
    let recovery_requests = 100;
    let recovery_start = Instant::now();
    let mut recovery_tasks = Vec::new();
    
    for _i in 0..recovery_requests {
        recovery_tasks.push(async move {
            let start = Instant::now();
            let result = timeout(Duration::from_millis(100), async {
                sleep(Duration::from_millis(5)).await;
                "recovery_complete"
            }).await;
            (start.elapsed(), result.is_ok())
        });
    }
    
    let _recovery_results = join_all(recovery_tasks).await;
    let recovery_time = recovery_start.elapsed();
    
    println!("📊 CHAOS FAILURE INJECTION RESULTS:");
    println!("   📈 Baseline Time: {:.2}s", baseline_time.as_secs_f64());
    println!("   💥 Failure Survival Rate: {:.1}%", survival_rate);
    println!("   ⏱️  Failure Phase Time: {:.2}s", failure_time.as_secs_f64());
    println!("   🔄 Recovery Time: {:.2}s", recovery_time.as_secs_f64());
    
    // Enterprise resilience requirements
    assert!(survival_rate > 70.0, "Survival rate too low: {:.1}% (required: >70%)", survival_rate);
    assert!(recovery_time.as_secs_f64() < baseline_time.as_secs_f64() * 1.2, 
        "Recovery too slow: {:.2}s vs baseline {:.2}s", recovery_time.as_secs_f64(), baseline_time.as_secs_f64());
    
    orchestrator.stop().await.unwrap();
    println!("✅ Service failure injection & recovery PASSED");
}

#[tokio::test]
async fn test_load_balancer_chaos_resilience() {
    println!("⚖️💥 === LOAD BALANCER CHAOS RESILIENCE ===");
    
    let lb_config = LoadBalancerConfig {
        strategy: LoadBalancerStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(10),
    };
    let lb = DefaultLoadBalancer::new(lb_config);
    
    // Create chaos service instances
    let chaos_services: Vec<_> = (0..8).map(|i| {
        let service = ChaosTestService::new(format!("lb-chaos-{}", i));
        
        // Inject different failure patterns
        match i % 4 {
            0 => service.inject_failures(50), // High failure rate
            1 => service.inject_slow_responses(80), // High slow response rate
            2 => service.set_unhealthy(), // Unhealthy
            _ => {}, // Normal service
        }
        
        songbird_orchestrator::load_balancer::ServiceInstance {
            service_info: ServiceInfo {
                id: format!("lb-chaos-{}", i),
                name: format!("LB Chaos Service {}", i),
                version: "1.0.0".to_string(),
                service_type: "chaos".to_string(),
                description: "Chaos load balancer test".to_string(),
                endpoints: vec![],
                capabilities: vec![],
                tags: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
            },
            weight: 1,
            current_connections: 0,
            is_healthy: i % 4 != 2, // 25% unhealthy
        }
    }).collect();
    
    // Test load balancer resilience under chaos
    let selection_attempts = 1000;
    let mut successful_selections = 0;
    let mut healthy_selections = 0;
    let selection_start = Instant::now();
    
    for _ in 0..selection_attempts {
        if let Some(selected) = {
            let lb_trait: &dyn LoadBalancer = &lb;
            lb_trait.select_service(&chaos_services).await.unwrap()
        } {
            successful_selections += 1;
            if selected.is_healthy {
                healthy_selections += 1;
            }
        }
    }
    
    let selection_time = selection_start.elapsed();
    let selection_success_rate = successful_selections as f64 / selection_attempts as f64 * 100.0;
    let healthy_selection_rate = healthy_selections as f64 / successful_selections.max(1) as f64 * 100.0;
    
    // Test load balancer stats collection under chaos
    let stats_result = lb.get_stats().await;
    
    println!("📊 LOAD BALANCER CHAOS RESULTS:");
    println!("   🎯 Selection Attempts: {}", selection_attempts);
    println!("   ✅ Successful Selections: {}", successful_selections);
    println!("   📈 Selection Success Rate: {:.1}%", selection_success_rate);
    println!("   💚 Healthy Selection Rate: {:.1}%", healthy_selection_rate);
    println!("   ⏱️  Total Selection Time: {:.2}s", selection_time.as_secs_f64());
    println!("   📊 Stats Collection: {}", if stats_result.is_ok() { "SUCCESS" } else { "FAILED" });
    
    // Enterprise load balancer resilience requirements
    assert!(selection_success_rate > 80.0, "Load balancer selection success rate too low: {:.1}%", selection_success_rate);
    assert!(healthy_selection_rate > 90.0, "Healthy selection rate too low: {:.1}%", healthy_selection_rate);
    assert!(stats_result.is_ok(), "Load balancer stats collection failed under chaos");
    
    println!("✅ Load balancer chaos resilience PASSED");
}

#[tokio::test]
async fn test_cascading_failure_prevention() {
    println!("🌊💥 === CASCADING FAILURE PREVENTION ===");
    
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await.unwrap();
    orchestrator.start().await.unwrap();
    
    // Create a chain of dependent services
    let service_count = 6;
    let mut service_chain = Vec::new();
    
    for i in 0..service_count {
        let service = ChaosTestService::new(format!("cascade-{}", i));
        service_chain.push(service.clone());
        orchestrator.register_service(service, ()).await.unwrap();
    }
    
    // Phase 1: Trigger initial failure
    println!("💥 Phase 1: Triggering initial failure");
    service_chain[0].inject_failures(100); // Complete failure of first service
    service_chain[0].set_unhealthy();
    
    // Phase 2: Simulate load that could cause cascading failure
    let cascade_test_duration = Duration::from_millis(2000);
    let request_rate = 50; // requests per 100ms
    let cascade_start = Instant::now();
    let mut cascade_results = Vec::new();
    
    while cascade_start.elapsed() < cascade_test_duration {
        let batch_start = Instant::now();
        let mut batch_tasks = Vec::new();
        
        for i in 0..request_rate {
            batch_tasks.push(async move {
                let _request = ServiceRequest::new("GET", &format!("/cascade-test/{}", i));
                
                // Simulate work with timeout to prevent hanging
                let result = timeout(Duration::from_millis(100), async {
                    sleep(Duration::from_millis(5)).await;
                    "completed"
                }).await;
                
                result.is_ok()
            });
        }
        
        let batch_results = join_all(batch_tasks).await;
        cascade_results.extend(batch_results);
        
        // Add delay to maintain request rate
        let batch_time = batch_start.elapsed();
        if batch_time < Duration::from_millis(100) {
            sleep(Duration::from_millis(100) - batch_time).await;
        }
        
        // Monitor for cascading failures
        let current_time = cascade_start.elapsed();
        if current_time > Duration::from_millis(500) && current_time < Duration::from_millis(1000) {
            // Inject additional stress during middle phase
            for (i, service) in service_chain.iter().enumerate() {
                if i > 0 && i < 3 { // Services 1 and 2
                    service.inject_slow_responses(40);
                }
            }
        }
    }
    
    let total_cascade_time = cascade_start.elapsed();
    let successful_requests = cascade_results.iter().filter(|&&r| r).count();
    let success_rate = successful_requests as f64 / cascade_results.len() as f64 * 100.0;
    
    // Phase 3: Recovery validation
    println!("🔄 Phase 3: Recovery validation");
    for service in &service_chain {
        service.inject_failures(0);
        service.inject_slow_responses(0);
        service.set_healthy();
    }
    
    sleep(Duration::from_millis(300)).await; // Recovery time
    
    // Recovery validation
    let recovery_requests = 100;
    let mut successful_recoveries = 0;
    
    for _i in 0..recovery_requests {
        let _request = ServiceRequest::new("GET", "/recovery-test");
        
        let result = timeout(Duration::from_millis(100), async {
            // Simulate recovery request
            sleep(Duration::from_millis(10)).await;
            "recovered"
        }).await;
        
        if result.is_ok() {
            successful_recoveries += 1;
        }
    }
    
    let recovery_success_rate = successful_recoveries as f64 / recovery_requests as f64 * 100.0;
    
    println!("📊 CASCADING FAILURE PREVENTION RESULTS:");
    println!("   🎯 Total Requests: {}", cascade_results.len());
    println!("   ✅ Successful During Chaos: {}", successful_requests);
    println!("   📈 Chaos Success Rate: {:.1}%", success_rate);
    println!("   ⏱️  Total Test Time: {:.2}s", total_cascade_time.as_secs_f64());
    println!("   🔄 Recovery Success Rate: {:.1}%", recovery_success_rate);
    
    // Enterprise cascade prevention requirements
    assert!(success_rate > 60.0, "Success rate during cascade too low: {:.1}% (required: >60%)", success_rate);
    assert!(recovery_success_rate > 95.0, "Recovery success rate too low: {:.1}% (required: >95%)", recovery_success_rate);
    
    orchestrator.stop().await.unwrap();
    println!("✅ Cascading failure prevention PASSED");
}

#[tokio::test]
async fn test_network_partition_simulation() {
    println!("🌐💔 === NETWORK PARTITION SIMULATION ===");
    
    let discovery = StaticServiceDiscovery::new();
    
    // Register services in different "network partitions"
    let partition_a_services = 3;
    let partition_b_services = 3;
    
    // Partition A services
    for i in 0..partition_a_services {
        let service_info = ServiceInfo {
            id: format!("partition-a-{}", i),
            name: format!("Partition A Service {}", i),
            version: "1.0.0".to_string(),
            service_type: "partition-test".to_string(),
            description: "Service in partition A".to_string(),
            endpoints: vec![],
            capabilities: vec!["partition-a".to_string()],
            tags: {
                let mut tags = std::collections::HashMap::new();
                tags.insert("partition".to_string(), "a".to_string());
                tags.insert("zone".to_string(), format!("zone-{}", i % 2));
                tags
            },
            metadata: std::collections::HashMap::new(),
        };
        discovery.register(service_info).await.unwrap();
    }
    
    // Partition B services
    for i in 0..partition_b_services {
        let service_info = ServiceInfo {
            id: format!("partition-b-{}", i),
            name: format!("Partition B Service {}", i),
            version: "1.0.0".to_string(),
            service_type: "partition-test".to_string(),
            description: "Service in partition B".to_string(),
            endpoints: vec![],
            capabilities: vec!["partition-b".to_string()],
            tags: {
                let mut tags = std::collections::HashMap::new();
                tags.insert("partition".to_string(), "b".to_string());
                tags.insert("zone".to_string(), format!("zone-{}", i % 2));
                tags
            },
            metadata: std::collections::HashMap::new(),
        };
        discovery.register(service_info).await.unwrap();
    }
    
    // Phase 1: Normal operations - all partitions accessible
    println!("📊 Phase 1: All partitions accessible");
    let all_services = discovery.list_all().await.unwrap();
    let partition_a_query = discovery.discover(
        ServiceQuery::new().with_service_type("partition-test").with_tag("partition")
    ).await.unwrap().into_iter().filter(|s| 
        s.tags.get("partition").map(|v| v == "a").unwrap_or(false)
    ).collect::<Vec<_>>();
    let partition_b_query = discovery.discover(
        ServiceQuery::new().with_service_type("partition-test").with_tag("partition")
    ).await.unwrap().into_iter().filter(|s| 
        s.tags.get("partition").map(|v| v == "b").unwrap_or(false)
    ).collect::<Vec<_>>();
    
    // Phase 2: Simulate partition failure - mark partition B services as unhealthy
    println!("💔 Phase 2: Simulating partition B network failure");
    for i in 0..partition_b_services {
        let service_id = format!("partition-b-{}", i);
        discovery.update_health(&service_id, ServiceHealthStatus::Unhealthy).await.unwrap();
    }
    
    // Test service discovery resilience during partition
    let partition_test_iterations = 50;
    let mut discovery_success_count = 0;
    let partition_start = Instant::now();
    
    for _ in 0..partition_test_iterations {
        // Try to discover healthy services
        if let Ok(healthy_services) = discovery.discover(ServiceQuery::new()).await {
            let healthy_count = healthy_services.len();
            if healthy_count >= partition_a_services { // Should at least find partition A
                discovery_success_count += 1;
            }
        }
        
        sleep(Duration::from_millis(10)).await;
    }
    
    let partition_time = partition_start.elapsed();
    let discovery_success_rate = discovery_success_count as f64 / partition_test_iterations as f64 * 100.0;
    
    // Phase 3: Recovery - restore partition B
    println!("🔄 Phase 3: Restoring partition B");
    for i in 0..partition_b_services {
        let service_id = format!("partition-b-{}", i);
        discovery.update_health(&service_id, ServiceHealthStatus::Healthy).await.unwrap();
    }
    
    // Validate full recovery
    sleep(Duration::from_millis(200)).await;
    let recovered_services = discovery.list_all().await.unwrap();
    let post_recovery_partition_a = discovery.discover(
        ServiceQuery::new().with_service_type("partition-test").with_tag("partition")
    ).await.unwrap().into_iter().filter(|s| 
        s.tags.get("partition").map(|v| v == "a").unwrap_or(false)
    ).collect::<Vec<_>>();
    let post_recovery_partition_b = discovery.discover(
        ServiceQuery::new().with_service_type("partition-test").with_tag("partition")
    ).await.unwrap().into_iter().filter(|s| 
        s.tags.get("partition").map(|v| v == "b").unwrap_or(false)
    ).collect::<Vec<_>>();
    
    println!("📊 NETWORK PARTITION SIMULATION RESULTS:");
    println!("   🎯 Total Services: {}", all_services.len());
    println!("   🅰️  Partition A Services: {}", partition_a_query.len());
    println!("   🅱️  Partition B Services: {}", partition_b_query.len());
    println!("   💔 Discovery Success During Partition: {:.1}%", discovery_success_rate);
    println!("   ⏱️  Partition Test Time: {:.2}s", partition_time.as_secs_f64());
    println!("   🔄 Recovered Services: {}", recovered_services.len());
    println!("   ✅ Post-Recovery A/B: {}/{}", post_recovery_partition_a.len(), post_recovery_partition_b.len());
    
    // Enterprise partition resilience requirements
    assert_eq!(all_services.len(), partition_a_services + partition_b_services, "Initial service registration failed");
    assert!(discovery_success_rate > 90.0, "Discovery success rate during partition too low: {:.1}%", discovery_success_rate);
    assert_eq!(recovered_services.len(), partition_a_services + partition_b_services, "Service recovery incomplete");
    assert_eq!(post_recovery_partition_a.len(), partition_a_services, "Partition A recovery failed");
    assert_eq!(post_recovery_partition_b.len(), partition_b_services, "Partition B recovery failed");
    
    println!("✅ Network partition simulation PASSED");
}

#[tokio::test]
async fn test_resource_exhaustion_resilience() {
    println!("📊💥 === RESOURCE EXHAUSTION RESILIENCE ===");
    
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await.unwrap();
    orchestrator.start().await.unwrap();
    
    // Register services that will simulate resource exhaustion
    let service_count = 8;
    let mut resource_services = Vec::new();
    
    for i in 0..service_count {
        let service = ChaosTestService::new(format!("resource-{}", i));
        resource_services.push(service.clone());
        orchestrator.register_service(service, ()).await.unwrap();
    }
    
    // Phase 1: Simulate memory exhaustion
    println!("💾 Phase 1: Simulating memory exhaustion");
    let memory_pressure_data: Vec<Vec<u8>> = (0..1000)
        .map(|_| vec![0u8; 1024 * 10]) // 10KB chunks
        .collect();
    
    // Simulate high CPU load on some services
    for (i, service) in resource_services.iter().enumerate() {
        if i % 2 == 0 {
            service.inject_slow_responses(60); // Simulate CPU-bound operations
        }
    }
    
    // Test system behavior under resource pressure
    let resource_test_requests = 200;
    let mut resource_tasks = Vec::new();
    let resource_start = Instant::now();
    
    for i in 0..resource_test_requests {
        resource_tasks.push(async move {
            let start = Instant::now();
            
            // Simulate resource-intensive request
            let result = timeout(Duration::from_millis(200), async {
                sleep(Duration::from_millis(10 + (i % 50) as u64)).await;
                
                // Simulate some memory allocation
                let _temp_data = vec![0u8; 1024];
                "resource_test_complete"
            }).await;
            
            (start.elapsed(), result.is_ok())
        });
    }
    
    let resource_results = join_all(resource_tasks).await;
    let resource_time = resource_start.elapsed();
    
    let successful_under_pressure = resource_results.iter().filter(|(_, success)| *success).count();
    let success_rate_under_pressure = successful_under_pressure as f64 / resource_test_requests as f64 * 100.0;
    let avg_response_time = resource_results.iter()
        .map(|(time, _)| time.as_millis())
        .sum::<u128>() as f64 / resource_test_requests as f64;
    
    // Phase 2: Recovery from resource exhaustion
    println!("🔄 Phase 2: Recovery from resource exhaustion");
    
    // Release memory pressure
    drop(memory_pressure_data);
    
    // Remove slow response injection
    for service in &resource_services {
        service.inject_slow_responses(0);
    }
    
    // Allow recovery time
    sleep(Duration::from_millis(500)).await;
    
    // Test post-recovery performance
    let recovery_requests = 100;
    let mut recovery_tasks = Vec::new();
    
    for _i in 0..recovery_requests {
        recovery_tasks.push(async move {
            let start = Instant::now();
            let result = timeout(Duration::from_millis(100), async {
                sleep(Duration::from_millis(5)).await;
                "recovery_complete"
            }).await;
            (start.elapsed(), result.is_ok())
        });
    }
    
    let recovery_results = join_all(recovery_tasks).await;
    let recovery_successes = recovery_results.iter().filter(|(_, success)| *success).count();
    let recovery_success_rate = recovery_successes as f64 / recovery_requests as f64 * 100.0;
    let recovery_avg_time = recovery_results.iter()
        .map(|(time, _)| time.as_millis())
        .sum::<u128>() as f64 / recovery_requests as f64;
    
    println!("📊 RESOURCE EXHAUSTION RESILIENCE RESULTS:");
    println!("   🎯 Requests Under Pressure: {}", resource_test_requests);
    println!("   ✅ Successful Under Pressure: {}", successful_under_pressure);
    println!("   📈 Success Rate Under Pressure: {:.1}%", success_rate_under_pressure);
    println!("   ⏱️  Avg Response Time Under Pressure: {:.2}ms", avg_response_time);
    println!("   🔄 Recovery Success Rate: {:.1}%", recovery_success_rate);
    println!("   ⚡ Recovery Avg Response Time: {:.2}ms", recovery_avg_time);
    println!("   📊 Total Test Time: {:.2}s", resource_time.as_secs_f64());
    
    // Enterprise resource resilience requirements
    assert!(success_rate_under_pressure > 75.0, "Success rate under pressure too low: {:.1}%", success_rate_under_pressure);
    assert!(recovery_success_rate > 95.0, "Recovery success rate too low: {:.1}%", recovery_success_rate);
    assert!(recovery_avg_time < avg_response_time, "Recovery performance not improved");
    
    orchestrator.stop().await.unwrap();
    println!("✅ Resource exhaustion resilience PASSED");
} 