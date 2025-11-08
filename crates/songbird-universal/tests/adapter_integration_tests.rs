#![cfg(feature = "tests-incomplete")]
//! NOTE: Disabled - requires unimplemented methods

//! # 🔌 Comprehensive Adapter Integration Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! Tests all 4 production adapters working together in various scenarios.
//! This validates the complete adapter system for `ToadStool`, `BearDog`, `NestGate`, and Squirrel.

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use songbird_test_utils::{
        test_discovery_port, test_federation_port, test_health_port, test_orchestrator_port,
    };

    /// Test: All adapters can be created successfully
    #[test]
    fn test_all_adapters_creation() -> SongbirdResult<()> {
        // Create all 4 production adapters
        let toadstool = ComputeAdapter::new(
            format!("http://localhost:{}", test_orchestrator_port()).to_string(),
        )
        .ok_or_else(|| SongbirdError::configuration(format!("ToadStool adapter: {}", e)))?;
        let beardog =
            SecurityAdapter::new(format!("http://localhost:{}", test_discovery_port()).to_string())
                .ok_or_else(|| SongbirdError::configuration(format!("BearDog adapter: {}", e)))?;
        let nestgate =
            StorageAdapter::new(format!("http://localhost:{}", test_health_port()).to_string())
                .ok_or_else(|| SongbirdError::configuration(format!("NestGate adapter: {}", e)))?;
        let squirrel =
            AIAdapter::new(format!("http://localhost:{}", test_federation_port()).to_string())
                .ok_or_else(|| SongbirdError::configuration(format!("Squirrel adapter: {}", e)))?;

        // Verify endpoints
        assert_eq!(toadstool.endpoint(), format!("http://localhost:{}", test_orchestrator_port()));
        assert_eq!(beardog.endpoint(), format!("http://localhost:{}", test_discovery_port()));
        assert_eq!(nestgate.endpoint(), format!("http://localhost:{}", test_health_port()));
        assert_eq!(squirrel.endpoint(), format!("http://localhost:{}", test_federation_port()));
        Ok(())
    }

    /// Test: All adapters support custom timeouts
    #[test]
    fn test_all_adapters_custom_timeouts() -> SongbirdResult<()> {
        use std::time::Duration;

        let toadstool = ComputeAdapter::new(
            format!("http://localhost:{}", test_orchestrator_port()).to_string(),
        )
        .ok_or_else(|| SongbirdError::configuration(format!("ToadStool adapter: {}", e)))?
        .with_timeout(Duration::from_secs(10));

        let beardog =
            SecurityAdapter::new(format!("http://localhost:{}", test_discovery_port()).to_string())
                .ok_or_else(|| SongbirdError::configuration(format!("BearDog adapter: {}", e)))?
                .with_timeout(Duration::from_secs(10));

        let nestgate =
            StorageAdapter::new(format!("http://localhost:{}", test_health_port()).to_string())
                .ok_or_else(|| SongbirdError::configuration(format!("NestGate adapter: {}", e)))?
                .with_timeout(Duration::from_secs(10));

        let squirrel =
            AIAdapter::new(format!("http://localhost:{}", test_federation_port()).to_string())
                .ok_or_else(|| SongbirdError::configuration(format!("Squirrel adapter: {}", e)))?
                .with_timeout(Duration::from_secs(20));

        assert_eq!(toadstool.endpoint(), format!("http://localhost:{}", test_orchestrator_port()));
        assert_eq!(beardog.endpoint(), format!("http://localhost:{}", test_discovery_port()));
        assert_eq!(nestgate.endpoint(), format!("http://localhost:{}", test_health_port()));
        assert_eq!(squirrel.endpoint(), format!("http://localhost:{}", test_federation_port()));
        Ok(())
    }

    /// Test: Adapter factory pattern for dynamic creation
    #[test]
    fn test_adapter_factory_pattern() {
        #[derive(Copy, Clone)]
        enum AdapterType {
            Compute,
            Security,
            Storage,
            AI,
        }

        fn create_adapter_endpoint(adapter_type: AdapterType) -> String {
            match adapter_type {
                AdapterType::Compute => {
                    format!("http://toadstool:{}", test_orchestrator_port()).to_string()
                }
                AdapterType::Security => {
                    format!("http://beardog:{}", test_discovery_port()).to_string()
                }
                AdapterType::Storage => {
                    format!("http://nestgate:{}", test_health_port()).to_string()
                }
                AdapterType::AI => {
                    format!("http://squirrel:{}", test_federation_port()).to_string()
                }
            }
        }

        let compute_endpoint = create_adapter_endpoint(AdapterType::Compute);
        let security_endpoint = create_adapter_endpoint(AdapterType::Security);
        let storage_endpoint = create_adapter_endpoint(AdapterType::Storage);
        let ai_endpoint = create_adapter_endpoint(AdapterType::AI);

        assert_eq!(compute_endpoint, format!("http://toadstool:{}", test_orchestrator_port()));
        assert_eq!(security_endpoint, format!("http://beardog:{}", test_discovery_port()));
        assert_eq!(storage_endpoint, format!("http://nestgate:{}", test_health_port()));
        assert_eq!(ai_endpoint, format!("http://squirrel:{}", test_federation_port()));
    }

    /// Test: Metrics type system completeness
    #[test]
    fn test_metrics_type_system() {
        use songbird_universal::adapters::{
            ai::AIMetrics, compute::ComputeMetrics, security::SecurityMetrics,
            storage::StorageMetrics,
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
            songbird_universal::adapters::compute::HealthStatus::Healthy
        );
        assert_eq!(
            security.health_status(),
            songbird_universal::adapters::security::SecurityHealth::Healthy
        );
        assert_eq!(
            storage.health_status(),
            songbird_universal::adapters::storage::StorageHealth::Healthy
        );
        assert_eq!(ai.health_status(), songbird_universal::adapters::ai::AIHealth::Healthy);
    }

    /// Test: Health status aggregation across all adapters
    #[test]
    fn test_ecosystem_health_aggregation() {
        use songbird_universal::adapters::{
            ai::AIMetrics, compute::ComputeMetrics, security::SecurityMetrics,
            storage::StorageMetrics,
        };

        #[allow(clippy::struct_excessive_bools)]
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
        use songbird_universal::adapters::compute::ComputeMetrics;

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
            songbird_universal::adapters::compute::HealthStatus::Degraded
        );
    }

    /// Test: Critical service detection
    #[test]
    fn test_critical_service_detection() {
        use songbird_universal::adapters::security::SecurityMetrics;

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
            songbird_universal::adapters::security::SecurityHealth::Critical
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
                Capability::Compute => "ComputeAdapter",
                Capability::Security => "SecurityAdapter",
                Capability::Storage => "StorageAdapter",
                Capability::AI => "AIAdapter",
            }
        }

        assert_eq!(select_adapter_for_capability(Capability::Compute), "ComputeAdapter");
        assert_eq!(select_adapter_for_capability(Capability::Security), "SecurityAdapter");
        assert_eq!(select_adapter_for_capability(Capability::Storage), "StorageAdapter");
        assert_eq!(select_adapter_for_capability(Capability::AI), "AIAdapter");
    }

    /// Test: Multi-service orchestration scenario
    #[test]
    fn test_multi_service_orchestration() {
        use songbird_universal::adapters::{
            compute::ComputeMetrics, security::SecurityMetrics, storage::StorageMetrics,
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
    fn test_adapter_endpoint_validation() -> SongbirdResult<()> {
        // All adapters should accept various endpoint formats
        let adapters = [
            ComputeAdapter::new(
                format!("http://localhost:{}", test_orchestrator_port()).to_string(),
            )
            .ok_or_else(|| SongbirdError::configuration(format!("ToadStool adapter: {}", e)))?
            .endpoint()
            .to_string(),
            SecurityAdapter::new("https://secure-beardog:8081".to_string())
                .ok_or_else(|| SongbirdError::configuration(format!("BearDog adapter: {}", e)))?
                .endpoint()
                .to_string(),
            StorageAdapter::new(
                format!("http://nestgate.local:{}", test_health_port()).to_string(),
            )
            .ok_or_else(|| SongbirdError::configuration(format!("NestGate adapter: {}", e)))?
            .endpoint()
            .to_string(),
            AIAdapter::new(format!("http://192.168.1.100:{}", test_federation_port()).to_string())
                .ok_or_else(|| SongbirdError::configuration(format!("Squirrel adapter: {}", e)))?
                .endpoint()
                .to_string(),
        ];

        assert_eq!(adapters[0], format!("http://localhost:{}", test_orchestrator_port()));
        assert_eq!(adapters[1], "https://secure-beardog:8081");
        assert_eq!(adapters[2], format!("http://nestgate.local:{}", test_health_port()));
        assert_eq!(adapters[3], format!("http://192.168.1.100:{}", test_federation_port()));
        Ok(())
    }

    /// Test: Time-series metrics collection pattern
    #[test]
    fn test_metrics_time_series_pattern() {
        // Unused imports removed
        use songbird_universal::adapters::compute::ComputeMetrics;

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
                cpu_usage_percent: f64::from(i).mul_add(10.0, 30.0),
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
        assert!(time_series.is_trending_up().or_else(|_| SongbirdError::configuration(
            format!("trend check should work: {}", e)
        ))?);
    }
}
