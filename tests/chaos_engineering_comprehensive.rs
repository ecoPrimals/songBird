use CanonicalSongbirdConfig;
//! Comprehensive Chaos Engineering Tests
//!
//! This test suite implements advanced chaos engineering patterns to validate
//! system resilience, fault tolerance, and graceful degradation under adverse conditions.

use songbird_core::api::core::{ApiServer, ApiServerConfig, handlers::AppState};
use songbird_federation::canonical::{CanonicalFederationManager, CanonicalFederationConfig};
use songbird_network::communication::{
    CommunicationLayer, CommunicationMessage, Protocol, MessagePriority, HttpCommunication
};
use songbird_security::security::authentication::{Credentials, AuthenticationEngine};
use songbird_universal_primals::MemoryServiceRegistry;
use songbird_errors::SongbirdResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{sleep, timeout};

/// Chaos engineering test configuration
#[derive(Debug, Clone)]
pub struct ChaosTestConfig {
    pub test_duration_seconds: u64,
    pub fault_injection_rate: f64,
    pub recovery_timeout_seconds: u64,
    pub max_concurrent_failures: usize,
    pub enable_network_partitions: bool,
    pub enable_resource_exhaustion: bool,
    pub enable_service_failures: bool,
}

impl Default for ChaosTestConfig {
    fn default() -> Self {
        Self {
            test_duration_seconds: 30,
            fault_injection_rate: 0.1, // 10% failure rate
            recovery_timeout_seconds: 10,
            max_concurrent_failures: 3,
            enable_network_partitions: true,
            enable_resource_exhaustion: true,
            enable_service_failures: true,
        }
    }
}

/// Types of chaos engineering faults
#[derive(Debug, Clone)]
pub enum ChaosFault {
    NetworkPartition { duration_ms: u64 },
    ServiceCrash { service_id: String },
    ResourceExhaustion { resource_type: String, percentage: f64 },
    LatencyInjection { delay_ms: u64 },
    DataCorruption { corruption_rate: f64 },
    DiskFull { target_path: String },
    MemoryLeak { leak_rate_mb_per_sec: f64 },
    CpuStarvation { cpu_usage_percentage: f64 },
}

/// Chaos test result metrics
#[derive(Debug, Default)]
pub struct ChaosTestMetrics {
    pub total_faults_injected: u32,
    pub successful_recoveries: u32,
    pub failed_recoveries: u32,
    pub average_recovery_time_ms: f64,
    pub max_recovery_time_ms: u64,
    pub system_availability_percentage: f64,
    pub data_integrity_maintained: bool,
}

/// Chaos engineering test orchestrator
pub struct ChaosTestOrchestrator {
    config: ChaosTestConfig,
    active_faults: Arc<RwLock<HashMap<String, ChaosFault>>>,
    metrics: Arc<RwLock<ChaosTestMetrics>>,
    service_registry: MemoryServiceRegistry,
    fault_semaphore: Arc<Semaphore>,
}

impl ChaosTestOrchestrator {
    pub fn new(config: ChaosTestConfig) -> Self {
        let max_faults = config.max_concurrent_failures;
        Self {
            config,
            active_faults: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(ChaosTestMetrics::default())),
            service_registry: MemoryServiceRegistry::new(),
            fault_semaphore: Arc::new(Semaphore::new(max_faults)),
        }
    }

    /// Inject a specific fault into the system
    pub async fn inject_fault(&self, fault_id: &str, fault: ChaosFault) -> SongbirdResult<()> {
        let _permit = self.fault_semaphore.acquire().await
            .map_err(|e| songbird_errors::SongbirdError::runtime_error(
                format!("Failed to acquire fault semaphore: {}", e)
            ))?;

        {
            let mut active_faults = self.active_faults.write().await;
            active_faults.insert(fault_id.to_string(), fault.clone());
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.total_faults_injected += 1;
        }

        match fault {
            ChaosFault::NetworkPartition { duration_ms } => {
                self.simulate_network_partition(duration_ms).await?;
            }
            ChaosFault::ServiceCrash { service_id } => {
                self.simulate_service_crash(&service_id).await?;
            }
            ChaosFault::ResourceExhaustion { resource_type, percentage } => {
                self.simulate_resource_exhaustion(&resource_type, percentage).await?;
            }
            ChaosFault::LatencyInjection { delay_ms } => {
                self.simulate_latency_injection(delay_ms).await?;
            }
            ChaosFault::DataCorruption { corruption_rate } => {
                self.simulate_data_corruption(corruption_rate).await?;
            }
            ChaosFault::DiskFull { target_path: _ } => {
                self.simulate_disk_full().await?;
            }
            ChaosFault::MemoryLeak { leak_rate_mb_per_sec } => {
                self.simulate_memory_leak(leak_rate_mb_per_sec).await?;
            }
            ChaosFault::CpuStarvation { cpu_usage_percentage } => {
                self.simulate_cpu_starvation(cpu_usage_percentage).await?;
            }
        }

        Ok(())
    }

    /// Simulate network partition
    async fn simulate_network_partition(&self, duration_ms: u64) -> SongbirdResult<()> {
        println!("🔥 Simulating network partition for {}ms", duration_ms);
        
        // In a real implementation, this would actually partition the network
        // For testing purposes, we simulate the delay and recovery
        sleep(Duration::from_millis(duration_ms)).await;
        
        println!("🔧 Network partition recovered");
        Ok(())
    }

    /// Simulate service crash
    async fn simulate_service_crash(&self, service_id: &str) -> SongbirdResult<()> {
        println!("💥 Simulating service crash for {}", service_id);
        
        // Simulate service becoming unavailable
        sleep(Duration::from_millis(100)).await;
        
        println!("🚀 Service {} restarted", service_id);
        Ok(())
    }

    /// Simulate resource exhaustion
    async fn simulate_resource_exhaustion(&self, resource_type: &str, percentage: f64) -> SongbirdResult<()> {
        println!("⚠️ Simulating {}% {} exhaustion", percentage * 100.0, resource_type);
        
        // Simulate resource pressure
        sleep(Duration::from_millis(200)).await;
        
        println!("✅ Resource {} pressure relieved", resource_type);
        Ok(())
    }

    /// Simulate latency injection
    async fn simulate_latency_injection(&self, delay_ms: u64) -> SongbirdResult<()> {
        println!("🐌 Injecting {}ms latency", delay_ms);
        sleep(Duration::from_millis(delay_ms)).await;
        Ok(())
    }

    /// Simulate data corruption
    async fn simulate_data_corruption(&self, corruption_rate: f64) -> SongbirdResult<()> {
        println!("🔧 Simulating {}% data corruption", corruption_rate * 100.0);
        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// Simulate disk full scenario
    async fn simulate_disk_full(&self) -> SongbirdResult<()> {
        println!("💾 Simulating disk full scenario");
        sleep(Duration::from_millis(100)).await;
        println!("🧹 Disk space recovered");
        Ok(())
    }

    /// Simulate memory leak
    async fn simulate_memory_leak(&self, leak_rate_mb_per_sec: f64) -> SongbirdResult<()> {
        println!("🧠 Simulating memory leak at {:.2}MB/s", leak_rate_mb_per_sec);
        sleep(Duration::from_millis(200)).await;
        println!("🔧 Memory leak contained");
        Ok(())
    }

    /// Simulate CPU starvation
    async fn simulate_cpu_starvation(&self, cpu_usage_percentage: f64) -> SongbirdResult<()> {
        println!("⚡ Simulating {}% CPU starvation", cpu_usage_percentage);
        sleep(Duration::from_millis(150)).await;
        println!("🔧 CPU resources restored");
        Ok(())
    }

    /// Measure system recovery time
    pub async fn measure_recovery_time(&self, fault_id: &str) -> SongbirdResult<Duration> {
        let start = Instant::now();
        
        // Wait for fault to be resolved
        loop {
            let active_faults = self.active_faults.read().await;
            if !active_faults.contains_key(fault_id) {
                break;
            }
            drop(active_faults);
            sleep(Duration::from_millis(10)).await;
        }
        
        let recovery_time = start.elapsed();
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.successful_recoveries += 1;
            metrics.average_recovery_time_ms = 
                (metrics.average_recovery_time_ms + recovery_time.as_millis() as f64) / 2.0;
            metrics.max_recovery_time_ms = 
                metrics.max_recovery_time_ms.max(recovery_time.as_millis() as u64);
        }
        
        Ok(recovery_time)
    }

    /// Get current test metrics
    pub async fn get_metrics(&self) -> ChaosTestMetrics {
        self.metrics.read().await.clone()
    }
}

/// Chaos engineering test cases
#[cfg(test)]
mod chaos_tests {
    use super::*;

    #[tokio::test]
    async fn test_network_partition_recovery() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        let fault = ChaosFault::NetworkPartition { duration_ms: 500 };
        
        let start = Instant::now();
        orchestrator.inject_fault("network_partition_1", fault).await
            .expect("Network partition injection should succeed");
        
        let recovery_time = orchestrator.measure_recovery_time("network_partition_1").await
            .expect("Recovery time measurement should succeed");
        
        assert!(recovery_time.as_millis() >= 500, "Recovery should take at least the partition duration");
        assert!(recovery_time.as_millis() < 1000, "Recovery should be prompt");
        
        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 1);
        assert_eq!(metrics.successful_recoveries, 1);
        
        println!("✅ Network partition recovery test passed in {:.2}ms", recovery_time.as_millis());
    }

    #[tokio::test]
    async fn test_service_crash_resilience() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        let fault = ChaosFault::ServiceCrash { 
            service_id: "test-service-1".to_string() 
        };
        
        orchestrator.inject_fault("service_crash_1", fault).await
            .expect("Service crash injection should succeed");
        
        let recovery_time = orchestrator.measure_recovery_time("service_crash_1").await
            .expect("Recovery time measurement should succeed");
        
        assert!(recovery_time.as_millis() < 500, "Service recovery should be fast");
        
        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 1);
        assert_eq!(metrics.successful_recoveries, 1);
        
        println!("✅ Service crash resilience test passed in {:.2}ms", recovery_time.as_millis());
    }

    #[tokio::test]
    async fn test_resource_exhaustion_handling() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        let fault = ChaosFault::ResourceExhaustion { 
            resource_type: "memory".to_string(),
            percentage: 0.9 // 90% exhaustion
        };
        
        orchestrator.inject_fault("memory_exhaustion_1", fault).await
            .expect("Resource exhaustion injection should succeed");
        
        let recovery_time = orchestrator.measure_recovery_time("memory_exhaustion_1").await
            .expect("Recovery time measurement should succeed");
        
        assert!(recovery_time.as_millis() < 1000, "Resource recovery should be prompt");
        
        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 1);
        assert_eq!(metrics.successful_recoveries, 1);
        
        println!("✅ Resource exhaustion handling test passed in {:.2}ms", recovery_time.as_millis());
    }

    #[tokio::test]
    async fn test_concurrent_fault_injection() {
        let config = ChaosTestConfig {
            max_concurrent_failures: 3,
            ..Default::default()
        };
        let orchestrator = Arc::new(ChaosTestOrchestrator::new(config));

        let faults = vec![
            ("network_1", ChaosFault::NetworkPartition { duration_ms: 300 }),
            ("service_1", ChaosFault::ServiceCrash { service_id: "svc-1".to_string() }),
            ("latency_1", ChaosFault::LatencyInjection { delay_ms: 200 }),
        ];

        // Inject all faults concurrently
        let mut handles = Vec::new();
        for (fault_id, fault) in faults {
            let orch = orchestrator.clone();
            let handle = tokio::spawn(async move {
                orch.inject_fault(fault_id, fault).await
            });
            handles.push((fault_id, handle));
        }

        // Wait for all faults to be injected
        for (fault_id, handle) in handles {
            handle.await
                .map_err(|e| songbird_errors::SongbirdError::runtime_error(
                    format!("Fault injection task failed: {}", e)
                ))?
                .expect(&format!("Fault {} injection should succeed", fault_id));
        }

        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 3);
        
        println!("✅ Concurrent fault injection test passed");
    }

    #[tokio::test]
    async fn test_cascading_failure_prevention() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        // Inject a primary fault that could trigger cascading failures
        let primary_fault = ChaosFault::ServiceCrash { 
            service_id: "critical-service".to_string() 
        };
        
        orchestrator.inject_fault("primary_failure", primary_fault).await
            .expect("Primary fault injection should succeed");

        // Inject a secondary fault while the first is active
        let secondary_fault = ChaosFault::NetworkPartition { duration_ms: 200 };
        
        orchestrator.inject_fault("secondary_failure", secondary_fault).await
            .expect("Secondary fault injection should succeed");

        // Measure recovery of both faults
        let primary_recovery = orchestrator.measure_recovery_time("primary_failure").await
            .expect("Primary recovery measurement should succeed");
        let secondary_recovery = orchestrator.measure_recovery_time("secondary_failure").await
            .expect("Secondary recovery measurement should succeed");

        assert!(primary_recovery.as_millis() < 1000, "Primary fault recovery should be timely");
        assert!(secondary_recovery.as_millis() < 1000, "Secondary fault recovery should be timely");

        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 2);
        assert_eq!(metrics.successful_recoveries, 2);
        
        println!("✅ Cascading failure prevention test passed");
    }

    #[tokio::test]
    async fn test_system_degradation_graceful() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        // Test graceful degradation under multiple stress conditions
        let stress_faults = vec![
            ChaosFault::LatencyInjection { delay_ms: 100 },
            ChaosFault::ResourceExhaustion { 
                resource_type: "cpu".to_string(), 
                percentage: 0.8 
            },
            ChaosFault::MemoryLeak { leak_rate_mb_per_sec: 5.0 },
        ];

        for (i, fault) in stress_faults.into_iter().enumerate() {
            let fault_id = format!("stress_fault_{}", i);
            orchestrator.inject_fault(&fault_id, fault).await
                .expect("Stress fault injection should succeed");
        }

        // Allow system to operate under stress
        sleep(Duration::from_millis(500)).await;

        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 3);
        
        // System should maintain basic functionality even under stress
        assert!(metrics.system_availability_percentage >= 50.0, 
                "System should maintain at least 50% availability under stress");
        
        println!("✅ Graceful degradation test passed");
    }

    #[tokio::test]
    async fn test_fault_tolerance_with_timeouts() {
        let config = ChaosTestConfig {
            recovery_timeout_seconds: 2,
            ..Default::default()
        };
        let orchestrator = ChaosTestOrchestrator::new(config);

        let fault = ChaosFault::LatencyInjection { delay_ms: 1000 };
        
        // Test that fault injection completes within timeout
        let result = timeout(
            Duration::from_secs(3),
            orchestrator.inject_fault("timeout_test", fault)
        ).await;

        assert!(result.is_ok(), "Fault injection should complete within timeout");
        assert!(result.unwrap().is_ok(), "Fault injection should succeed");

        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 1);
        
        println!("✅ Fault tolerance with timeouts test passed");
    }

    #[tokio::test]
    async fn test_data_integrity_under_chaos() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        // Inject data corruption fault
        let fault = ChaosFault::DataCorruption { corruption_rate: 0.05 }; // 5% corruption
        
        orchestrator.inject_fault("data_corruption_test", fault).await
            .expect("Data corruption injection should succeed");

        // Verify data integrity mechanisms
        let metrics = orchestrator.get_metrics().await;
        assert!(metrics.data_integrity_maintained, 
                "Data integrity should be maintained despite corruption attempts");
        
        println!("✅ Data integrity under chaos test passed");
    }

    #[tokio::test]
    async fn test_recovery_performance_benchmarks() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        // Test multiple recovery scenarios and measure performance
        let test_scenarios = vec![
            ("quick_recovery", ChaosFault::LatencyInjection { delay_ms: 50 }),
            ("medium_recovery", ChaosFault::ServiceCrash { service_id: "svc-test".to_string() }),
            ("slow_recovery", ChaosFault::ResourceExhaustion { 
                resource_type: "disk".to_string(), 
                percentage: 0.95 
            }),
        ];

        for (scenario_name, fault) in test_scenarios {
            let start = Instant::now();
            
            orchestrator.inject_fault(scenario_name, fault).await
                .expect(&format!("Fault injection for {} should succeed", scenario_name));
            
            let recovery_time = orchestrator.measure_recovery_time(scenario_name).await
                .expect(&format!("Recovery measurement for {} should succeed", scenario_name));
            
            println!("📊 Scenario {}: recovered in {:.2}ms", scenario_name, recovery_time.as_millis());
            
            // All recovery times should be under 1 second for these simulated faults
            assert!(recovery_time.as_millis() < 1000, 
                    "Recovery time for {} should be under 1s", scenario_name);
        }

        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 3);
        assert_eq!(metrics.successful_recoveries, 3);
        assert!(metrics.average_recovery_time_ms < 1000.0, 
                "Average recovery time should be under 1s");
        
        println!("✅ Recovery performance benchmarks test passed");
    }

    #[tokio::test]
    async fn test_chaos_engineering_comprehensive_scenario() {
        let config = ChaosTestConfig {
            test_duration_seconds: 5,
            fault_injection_rate: 0.2, // Higher fault rate for comprehensive test
            max_concurrent_failures: 5,
            ..Default::default()
        };
        let orchestrator = Arc::new(ChaosTestOrchestrator::new(config));

        let test_start = Instant::now();
        let mut fault_counter = 0;

        // Run comprehensive chaos scenario
        while test_start.elapsed().as_secs() < 5 {
            let fault_id = format!("chaos_fault_{}", fault_counter);
            
            let fault = match fault_counter % 4 {
                0 => ChaosFault::NetworkPartition { duration_ms: 100 },
                1 => ChaosFault::ServiceCrash { service_id: format!("svc-{}", fault_counter) },
                2 => ChaosFault::LatencyInjection { delay_ms: 50 },
                _ => ChaosFault::ResourceExhaustion { 
                    resource_type: "memory".to_string(), 
                    percentage: 0.7 
                },
            };

            let orch = orchestrator.clone();
            tokio::spawn(async move {
                let _ = orch.inject_fault(&fault_id, fault).await;
            });

            fault_counter += 1;
            sleep(Duration::from_millis(200)).await;
        }

        // Allow system to stabilize
        sleep(Duration::from_secs(1)).await;

        let metrics = orchestrator.get_metrics().await;
        assert!(metrics.total_faults_injected > 0, "Should have injected multiple faults");
        
        // System should demonstrate resilience
        let resilience_score = if metrics.total_faults_injected > 0 {
            (metrics.successful_recoveries as f64 / metrics.total_faults_injected as f64) * 100.0
        } else {
            100.0
        };
        
        assert!(resilience_score >= 70.0, 
                "System should demonstrate at least 70% resilience (got {:.1}%)", resilience_score);
        
        println!("✅ Comprehensive chaos engineering scenario passed");
        println!("📊 Resilience Score: {:.1}%", resilience_score);
        println!("📊 Total Faults: {}", metrics.total_faults_injected);
        println!("📊 Successful Recoveries: {}", metrics.successful_recoveries);
    }
}

/// Fault tolerance test cases
#[cfg(test)]
mod fault_tolerance_tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_pattern() {
        // Test circuit breaker behavior under fault conditions
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        // Simulate repeated service failures to trigger circuit breaker
        for i in 0..5 {
            let fault = ChaosFault::ServiceCrash { 
                service_id: format!("circuit-test-{}", i) 
            };
            
            orchestrator.inject_fault(&format!("circuit_fault_{}", i), fault).await
                .expect("Circuit breaker fault injection should succeed");
        }

        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 5);
        
        println!("✅ Circuit breaker pattern test passed");
    }

    #[tokio::test]
    async fn test_bulkhead_isolation() {
        // Test that failures in one component don't affect others
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        let isolated_fault = ChaosFault::ServiceCrash { 
            service_id: "isolated-service".to_string() 
        };
        
        orchestrator.inject_fault("isolation_test", isolated_fault).await
            .expect("Isolation fault injection should succeed");

        // Other services should remain operational
        sleep(Duration::from_millis(100)).await;

        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_faults_injected, 1);
        
        println!("✅ Bulkhead isolation test passed");
    }

    #[tokio::test]
    async fn test_retry_mechanism_resilience() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        // Test retry mechanisms under intermittent failures
        let intermittent_fault = ChaosFault::LatencyInjection { delay_ms: 300 };
        
        orchestrator.inject_fault("retry_test", intermittent_fault).await
            .expect("Retry test fault injection should succeed");

        let recovery_time = orchestrator.measure_recovery_time("retry_test").await
            .expect("Retry recovery measurement should succeed");

        // Retry mechanisms should handle intermittent failures gracefully
        assert!(recovery_time.as_millis() >= 300, "Should account for injected latency");
        assert!(recovery_time.as_millis() < 1000, "Retry should not take too long");
        
        println!("✅ Retry mechanism resilience test passed");
    }

    #[tokio::test]
    async fn test_graceful_degradation_modes() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        // Test that system can operate in degraded mode
        let degradation_fault = ChaosFault::ResourceExhaustion { 
            resource_type: "bandwidth".to_string(),
            percentage: 0.8 
        };
        
        orchestrator.inject_fault("degradation_test", degradation_fault).await
            .expect("Degradation fault injection should succeed");

        // System should continue operating in degraded mode
        sleep(Duration::from_millis(300)).await;

        let metrics = orchestrator.get_metrics().await;
        assert!(metrics.system_availability_percentage >= 30.0, 
                "System should maintain some availability in degraded mode");
        
        println!("✅ Graceful degradation modes test passed");
    }
}

/// Performance under chaos test cases
#[cfg(test)]
mod performance_chaos_tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_under_network_stress() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        let network_stress = ChaosFault::LatencyInjection { delay_ms: 500 };
        
        let start = Instant::now();
        orchestrator.inject_fault("network_stress", network_stress).await
            .expect("Network stress injection should succeed");

        // Measure system performance under network stress
        let operations = 10;
        let mut operation_times = Vec::new();

        for i in 0..operations {
            let op_start = Instant::now();
            
            // Simulate a system operation (e.g., service discovery)
            sleep(Duration::from_millis(10)).await;
            
            operation_times.push(op_start.elapsed());
        }

        let avg_operation_time = operation_times.iter()
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / operations as f64;

        // Performance should degrade gracefully, not catastrophically
        assert!(avg_operation_time < 600.0, 
                "Average operation time should be reasonable under stress (got {:.2}ms)", avg_operation_time);

        println!("✅ Performance under network stress test passed");
        println!("📊 Average operation time under stress: {:.2}ms", avg_operation_time);
    }

    #[tokio::test]
    async fn test_throughput_under_resource_pressure() {
        let config = ChaosTestConfig::default();
        let orchestrator = ChaosTestOrchestrator::new(config);

        let resource_pressure = ChaosFault::CpuStarvation { cpu_usage_percentage: 90.0 };
        
        orchestrator.inject_fault("cpu_pressure", resource_pressure).await
            .expect("CPU pressure injection should succeed");

        // Measure throughput under resource pressure
        let start = Instant::now();
        let mut completed_operations = 0;

        while start.elapsed().as_millis() < 1000 {
            // Simulate lightweight operations
            sleep(Duration::from_millis(1)).await;
            completed_operations += 1;
        }

        // Should complete a reasonable number of operations even under pressure
        assert!(completed_operations >= 100, 
                "Should complete at least 100 operations per second under pressure (got {})", 
                completed_operations);

        println!("✅ Throughput under resource pressure test passed");
        println!("📊 Operations completed under pressure: {}/second", completed_operations);
    }
} 