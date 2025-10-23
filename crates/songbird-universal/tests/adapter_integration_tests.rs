//! # 🔌 Comprehensive Adapter Integration Tests
//!
//! Tests all 4 production adapters working together in various scenarios.
//! This validates the complete adapter system for `ToadStool`, `BearDog`, `NestGate`, and Squirrel.

use songbird_types::SongbirdError;
use songbird_universal::adapters::{
    BearDogSecurityAdapter, NestGateStorageAdapter, SquirrelAIAdapter, ToadStoolMetricsAdapter,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test: All adapters can be created successfully
    #[test]
    fn test_all_adapters_creation() {
        // Create all 4 production adapters
        let toadstool = ToadStoolMetricsAdapter::new("http://localhost:8080".to_string())
            .expect("ToadStool adapter");
        let beardog = BearDogSecurityAdapter::new("http://localhost:8081".to_string())
            .expect("BearDog adapter");
        let nestgate = NestGateStorageAdapter::new("http://localhost:8082".to_string())
            .expect("NestGate adapter");
        let squirrel =
            SquirrelAIAdapter::new("http://localhost:8083".to_string()).expect("Squirrel adapter");

        // Verify endpoints
        assert_eq!(toadstool.endpoint(), "http://localhost:8080");
        assert_eq!(beardog.endpoint(), "http://localhost:8081");
        assert_eq!(nestgate.endpoint(), "http://localhost:8082");
        assert_eq!(squirrel.endpoint(), "http://localhost:8083");
    }

    /// Test: All adapters support custom timeouts
    #[test]
    fn test_all_adapters_custom_timeouts() {
        use std::time::Duration;

        let toadstool = ToadStoolMetricsAdapter::new("http://localhost:8080".to_string())
            .expect("ToadStool adapter")
            .with_timeout(Duration::from_secs(10));

        let beardog = BearDogSecurityAdapter::new("http://localhost:8081".to_string())
            .expect("BearDog adapter")
            .with_timeout(Duration::from_secs(10));

        let nestgate = NestGateStorageAdapter::new("http://localhost:8082".to_string())
            .expect("NestGate adapter")
            .with_timeout(Duration::from_secs(10));

        let squirrel = SquirrelAIAdapter::new("http://localhost:8083".to_string())
            .expect("Squirrel adapter")
            .with_timeout(Duration::from_secs(20));

        assert_eq!(toadstool.endpoint(), "http://localhost:8080");
        assert_eq!(beardog.endpoint(), "http://localhost:8081");
        assert_eq!(nestgate.endpoint(), "http://localhost:8082");
        assert_eq!(squirrel.endpoint(), "http://localhost:8083");
    }

    /// Test: Adapter factory pattern for dynamic creation
    #[test]
    fn test_adapter_factory_pattern() {
        enum AdapterType {
            Compute,
            Security,
            Storage,
            AI,
        }

        fn create_adapter_endpoint(adapter_type: AdapterType) -> String {
            match adapter_type {
                AdapterType::Compute => "http://toadstool:8080".to_string(),
                AdapterType::Security => "http://beardog:8081".to_string(),
                AdapterType::Storage => "http://nestgate:8082".to_string(),
                AdapterType::AI => "http://squirrel:8083".to_string(),
            }
        }

        let compute_endpoint = create_adapter_endpoint(AdapterType::Compute);
        let security_endpoint = create_adapter_endpoint(AdapterType::Security);
        let storage_endpoint = create_adapter_endpoint(AdapterType::Storage);
        let ai_endpoint = create_adapter_endpoint(AdapterType::AI);

        assert_eq!(compute_endpoint, "http://toadstool:8080");
        assert_eq!(security_endpoint, "http://beardog:8081");
        assert_eq!(storage_endpoint, "http://nestgate:8082");
        assert_eq!(ai_endpoint, "http://squirrel:8083");
    }

    /// Test: Metrics type system completeness
    #[test]
    fn test_metrics_type_system() {
        use songbird_universal::adapters::{
            beardog::SecurityMetrics, nestgate::StorageMetrics, squirrel::AIMetrics,
            toadstool::ComputeMetrics,
        };

        // Compute metrics
        let compute = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 4_000_000_000,
            memory_available_bytes: 4_000_000_000,
            active_containers: 10,
            queued_jobs: 3,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };

        // Security metrics
        let security = SecurityMetrics {
            active_sessions: 100,
            failed_auth_attempts: 5,
            blocked_ips: 2,
            security_score: 0.95,
            timestamp: chrono::Utc::now(),
        };

        // Storage metrics
        let storage = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 400_000_000_000,
            available_bytes: 600_000_000_000,
            object_count: 5_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 30.0,
            timestamp: chrono::Utc::now(),
        };

        // AI metrics
        let ai = AIMetrics {
            active_models: 5,
            total_requests: 10_000,
            avg_latency_ms: 200.0,
            accuracy_score: 0.92,
            gpu_utilization_percent: 60.0,
            timestamp: chrono::Utc::now(),
        };

        // Verify all metrics are healthy
        assert_eq!(
            compute.health_status(),
            songbird_universal::adapters::toadstool::HealthStatus::Healthy
        );
        assert_eq!(
            security.health_status(),
            songbird_universal::adapters::beardog::SecurityHealth::Healthy
        );
        assert_eq!(
            storage.health_status(),
            songbird_universal::adapters::nestgate::StorageHealth::Healthy
        );
        assert_eq!(ai.health_status(), songbird_universal::adapters::squirrel::AIHealth::Healthy);
    }

    /// Test: Health status aggregation across all adapters
    #[test]
    fn test_ecosystem_health_aggregation() {
        use songbird_universal::adapters::{
            beardog::SecurityMetrics, nestgate::StorageMetrics, squirrel::AIMetrics,
            toadstool::ComputeMetrics,
        };

        struct EcosystemHealth {
            compute_healthy: bool,
            security_healthy: bool,
            storage_healthy: bool,
            ai_healthy: bool,
        }

        impl EcosystemHealth {
            fn is_fully_operational(&self) -> bool {
                self.compute_healthy
                    && self.security_healthy
                    && self.storage_healthy
                    && self.ai_healthy
            }

            fn health_percentage(&self) -> f64 {
                let healthy_count = [
                    self.compute_healthy,
                    self.security_healthy,
                    self.storage_healthy,
                    self.ai_healthy,
                ]
                .iter()
                .filter(|&&x| x)
                .count();
                (healthy_count as f64 / 4.0) * 100.0
            }
        }

        // Create healthy metrics for all services
        let compute = ComputeMetrics {
            cpu_usage_percent: 30.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 6_000_000_000,
            active_containers: 5,
            queued_jobs: 1,
            performance_score: 0.95,
            timestamp: chrono::Utc::now(),
        };

        let security = SecurityMetrics {
            active_sessions: 50,
            failed_auth_attempts: 2,
            blocked_ips: 1,
            security_score: 0.98,
            timestamp: chrono::Utc::now(),
        };

        let storage = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 300_000_000_000,
            available_bytes: 700_000_000_000,
            object_count: 2_000,
            avg_read_latency_ms: 10.0,
            avg_write_latency_ms: 15.0,
            timestamp: chrono::Utc::now(),
        };

        let ai = AIMetrics {
            active_models: 3,
            total_requests: 5_000,
            avg_latency_ms: 150.0,
            accuracy_score: 0.94,
            gpu_utilization_percent: 45.0,
            timestamp: chrono::Utc::now(),
        };

        let ecosystem = EcosystemHealth {
            compute_healthy: !compute.is_high_load(),
            security_healthy: !security.is_under_attack(),
            storage_healthy: !storage.is_nearly_full(),
            ai_healthy: !ai.is_high_gpu_load(),
        };

        assert!(ecosystem.is_fully_operational());
        assert_eq!(ecosystem.health_percentage(), 100.0);
    }

    /// Test: Degraded service detection
    #[test]
    fn test_degraded_service_detection() {
        use songbird_universal::adapters::toadstool::ComputeMetrics;

        // Create metrics for a degraded service
        let degraded_compute = ComputeMetrics {
            cpu_usage_percent: 85.0, // Above 80% threshold
            memory_usage_bytes: 7_000_000_000,
            memory_available_bytes: 1_000_000_000,
            active_containers: 25,
            queued_jobs: 15, // Above 10 threshold
            performance_score: 0.60,
            timestamp: chrono::Utc::now(),
        };

        assert!(degraded_compute.is_high_load());
        assert_eq!(
            degraded_compute.health_status(),
            songbird_universal::adapters::toadstool::HealthStatus::Degraded
        );
    }

    /// Test: Critical service detection
    #[test]
    fn test_critical_service_detection() {
        use songbird_universal::adapters::beardog::SecurityMetrics;

        // Create metrics for a critical security state
        let critical_security = SecurityMetrics {
            active_sessions: 200,
            failed_auth_attempts: 500, // Way above threshold
            blocked_ips: 100,          // Way above threshold
            security_score: 0.30,      // Below 0.5 threshold
            timestamp: chrono::Utc::now(),
        };

        assert!(critical_security.is_under_attack());
        assert_eq!(
            critical_security.health_status(),
            songbird_universal::adapters::beardog::SecurityHealth::Critical
        );
    }

    /// Test: Capability-based adapter selection
    #[test]
    fn test_capability_based_adapter_selection() {
        #[derive(Debug, PartialEq)]
        enum Capability {
            Compute,
            Security,
            Storage,
            AI,
        }

        fn select_adapter_for_capability(cap: Capability) -> &'static str {
            match cap {
                Capability::Compute => "ToadStoolMetricsAdapter",
                Capability::Security => "BearDogSecurityAdapter",
                Capability::Storage => "NestGateStorageAdapter",
                Capability::AI => "SquirrelAIAdapter",
            }
        }

        assert_eq!(select_adapter_for_capability(Capability::Compute), "ToadStoolMetricsAdapter");
        assert_eq!(select_adapter_for_capability(Capability::Security), "BearDogSecurityAdapter");
        assert_eq!(select_adapter_for_capability(Capability::Storage), "NestGateStorageAdapter");
        assert_eq!(select_adapter_for_capability(Capability::AI), "SquirrelAIAdapter");
    }

    /// Test: Multi-service orchestration scenario
    #[test]
    fn test_multi_service_orchestration() {
        use songbird_universal::adapters::{
            beardog::SecurityMetrics, nestgate::StorageMetrics, toadstool::ComputeMetrics,
        };

        struct OrchestrationDecision {
            should_scale_compute: bool,
            should_increase_security: bool,
            should_archive_data: bool,
        }

        fn make_orchestration_decision(
            compute: &ComputeMetrics,
            security: &SecurityMetrics,
            storage: &StorageMetrics,
        ) -> OrchestrationDecision {
            OrchestrationDecision {
                should_scale_compute: compute.is_high_load(),
                should_increase_security: security.is_under_attack(),
                should_archive_data: storage.is_nearly_full(),
            }
        }

        // Scenario: High load compute, security attack, storage nearly full
        let compute = ComputeMetrics {
            cpu_usage_percent: 90.0,
            memory_usage_bytes: 7_500_000_000,
            memory_available_bytes: 500_000_000,
            active_containers: 30,
            queued_jobs: 20,
            performance_score: 0.50,
            timestamp: chrono::Utc::now(),
        };

        let security = SecurityMetrics {
            active_sessions: 150,
            failed_auth_attempts: 120,
            blocked_ips: 60,
            security_score: 0.40,
            timestamp: chrono::Utc::now(),
        };

        let storage = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 950_000_000_000, // 95%
            available_bytes: 50_000_000_000,
            object_count: 100_000,
            avg_read_latency_ms: 50.0,
            avg_write_latency_ms: 100.0,
            timestamp: chrono::Utc::now(),
        };

        let decision = make_orchestration_decision(&compute, &security, &storage);

        assert!(decision.should_scale_compute);
        assert!(decision.should_increase_security);
        assert!(decision.should_archive_data);
    }

    /// Test: Adapter resilience - graceful handling of missing services
    #[test]
    fn test_adapter_endpoint_validation() {
        // All adapters should accept various endpoint formats
        let adapters = [
            ToadStoolMetricsAdapter::new("http://localhost:8080".to_string())
                .expect("ToadStool adapter")
                .endpoint()
                .to_string(),
            BearDogSecurityAdapter::new("https://secure-beardog:8081".to_string())
                .expect("BearDog adapter")
                .endpoint()
                .to_string(),
            NestGateStorageAdapter::new("http://nestgate.local:8082".to_string())
                .expect("NestGate adapter")
                .endpoint()
                .to_string(),
            SquirrelAIAdapter::new("http://192.168.1.100:8083".to_string())
                .expect("Squirrel adapter")
                .endpoint()
                .to_string(),
        ];

        assert_eq!(adapters[0], "http://localhost:8080");
        assert_eq!(adapters[1], "https://secure-beardog:8081");
        assert_eq!(adapters[2], "http://nestgate.local:8082");
        assert_eq!(adapters[3], "http://192.168.1.100:8083");
    }

    /// Test: Time-series metrics collection pattern
    #[test]
    fn test_metrics_time_series_pattern() {
        use songbird_universal::adapters::toadstool::ComputeMetrics;

        struct MetricsTimeSeries {
            metrics: Vec<ComputeMetrics>,
        }

        impl MetricsTimeSeries {
            fn new() -> Self {
                Self {
                    metrics: Vec::new(),
                }
            }

            fn add_sample(&mut self, metrics: ComputeMetrics) {
                self.metrics.push(metrics);
            }

            fn average_cpu_usage(&self) -> f64 {
                if self.metrics.is_empty() {
                    return 0.0;
                }
                let sum: f64 = self.metrics.iter().map(|m| m.cpu_usage_percent).sum();
                sum / self.metrics.len() as f64
            }

            fn is_trending_up(&self) -> Result<bool, SongbirdError> {
                if self.metrics.len() < 2 {
                    return Ok(false);
                }
                let last = self
                    .metrics
                    .last()
                    .ok_or_else(|| SongbirdError::configuration("No metrics found".to_string()))?
                    .cpu_usage_percent;
                let prev = self.metrics[self.metrics.len() - 2].cpu_usage_percent;
                Ok(last > prev)
            }
        }

        let mut time_series = MetricsTimeSeries::new();

        // Simulate 5 samples
        for i in 0..5 {
            let metrics = ComputeMetrics {
                cpu_usage_percent: 30.0 + (f64::from(i) * 10.0),
                memory_usage_bytes: 2_000_000_000,
                memory_available_bytes: 6_000_000_000,
                active_containers: 5,
                queued_jobs: 1,
                performance_score: 0.90,
                timestamp: chrono::Utc::now(),
            };
            time_series.add_sample(metrics);
        }

        assert_eq!(time_series.average_cpu_usage(), 50.0); // (30+40+50+60+70)/5
        assert!(time_series.is_trending_up().expect("trend check should work"));
    }
}
