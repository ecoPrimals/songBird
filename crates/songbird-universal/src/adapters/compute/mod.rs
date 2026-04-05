// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Generic Compute Capability Adapter
//!
//! **SOVEREIGNTY PRINCIPLE**: This adapter is capability-based and works with
//! ANY service providing compute capabilities. It does not hardcode primal names;
//! it only knows about "compute capability providers".
//!
//! ## Ecological Model
//!
//! Like in ecology, each organism exists independently:
//! - Songbird does not bind to a particular primal identity for compute
//! - Songbird only knows "something provides compute capability"
//! - A given ecosystem primal may implement this capability
//! - But it could be ANY compute provider
//!
//! ## Example
//!
//! ```rust,ignore
//! # tokio_test::block_on(async {
//! use songbird_universal::adapters::ComputeAdapter;
//!
//! // Discovers whoever provides compute capability
//! let adapter = ComputeAdapter::new_from_discovery().await?;
//! let metrics = adapter.collect_metrics().await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # });
//! ```

// Allow async_fn_in_trait warning - our traits guarantee Send + Sync
#![expect(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]

mod adapter;
mod metrics;

pub use adapter::{ComputeAdapter, ComputeMetricsProvider};
pub use metrics::{ComputeMetrics, HealthStatus};

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::Duration;

    static DISCOVERY_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn lock_discovery_env() -> std::sync::MutexGuard<'static, ()> {
        DISCOVERY_ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn test_compute_metrics_calculations() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 45.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 6_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.total_memory_bytes(), 8_000_000_000);
        assert!((metrics.memory_usage_percent() - 25.0).abs() < 0.1);
        assert!(!metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_high_load_detection() -> SongbirdResult<()> {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 96.0,
            memory_usage_bytes: 7_600_000_000,
            memory_available_bytes: 400_000_000,
            active_containers: 20,
            queued_jobs: 15,
            performance_score: 0.45,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Unhealthy);
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
        let adapter =
            ComputeAdapter::new("http://localhost:8080".to_string()).await.map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        assert_eq!(adapter.endpoint(), "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_eq!(HealthStatus::Degraded, HealthStatus::Degraded);
        assert_eq!(HealthStatus::Unhealthy, HealthStatus::Unhealthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
        assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_memory_usage_zero_total() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 0,
            memory_available_bytes: 0,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.total_memory_bytes(), 0);
        assert_eq!(metrics.memory_usage_percent(), 0.0);
    }

    #[test]
    fn test_high_load_cpu_boundary() {
        // Just below threshold
        let metrics_below = ComputeMetrics {
            cpu_usage_percent: 80.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_high_load());

        // Just above threshold
        let metrics_above = ComputeMetrics {
            cpu_usage_percent: 80.1,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_load());
    }

    #[test]
    fn test_high_load_memory_boundary() {
        // Just below threshold (85%)
        let metrics_below = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 8_499_000_000,
            memory_available_bytes: 1_501_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_high_load());

        // Just above threshold
        let metrics_above = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 8_501_000_000,
            memory_available_bytes: 1_499_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_load());
    }

    #[test]
    fn test_high_load_queued_jobs_boundary() {
        // At threshold
        let metrics_at = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 5,
            queued_jobs: 10,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_at.is_high_load());

        // Above threshold
        let metrics_above = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 5,
            queued_jobs: 11,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_load());
    }

    #[test]
    fn test_health_status_degraded_cpu() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 85.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 8_000_000_000,
            active_containers: 10,
            queued_jobs: 5,
            performance_score: 0.7,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_health_status_degraded_memory() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 9_000_000_000,
            memory_available_bytes: 1_000_000_000,
            active_containers: 10,
            queued_jobs: 5,
            performance_score: 0.7,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_health_status_unhealthy_cpu() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 96.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 8_000_000_000,
            active_containers: 10,
            queued_jobs: 5,
            performance_score: 0.3,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_status_unhealthy_memory() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 9_600_000_000,
            memory_available_bytes: 400_000_000,
            active_containers: 10,
            queued_jobs: 5,
            performance_score: 0.3,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_status_boundary_95_cpu() {
        // Exactly at threshold
        let metrics = ComputeMetrics {
            cpu_usage_percent: 95.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 8_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.7,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_health_status_boundary_95_memory() {
        // Exactly at threshold
        let metrics = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 9_500_000_000,
            memory_available_bytes: 500_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.7,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_compute_metrics_serialization() -> SongbirdResult<()> {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 45.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 6_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&metrics).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("cpu_usage_percent"));
        assert!(json.contains("memory_usage_bytes"));
        Ok(())
    }

    #[test]
    fn test_health_status_serialization() -> SongbirdResult<()> {
        assert_eq!(
            serde_json::to_string(&HealthStatus::Healthy).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Healthy\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Degraded).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Degraded\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Unhealthy).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Unhealthy\""
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_with_timeout() -> SongbirdResult<()> {
        let adapter = ComputeAdapter::new("http://compute-service:8080".to_string())
            .await
            .map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?
            .with_timeout(Duration::from_secs(25));
        assert_eq!(adapter.timeout, Duration::from_secs(25));
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_default_timeout() -> SongbirdResult<()> {
        let adapter =
            ComputeAdapter::new("http://compute-service:8080".to_string()).await.map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        assert_eq!(adapter.timeout, Duration::from_secs(5));
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_endpoint_access() -> SongbirdResult<()> {
        let adapter =
            ComputeAdapter::new("http://test-compute:9000".to_string()).await.map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        assert_eq!(adapter.endpoint(), "http://test-compute:9000");
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_debug_format() -> SongbirdResult<()> {
        let adapter =
            ComputeAdapter::new("http://compute:8080".to_string()).await.map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        let debug_str = format!("{:?}", adapter);
        assert!(debug_str.contains("ComputeAdapter"));
        assert!(debug_str.contains("http://compute:8080"));
        Ok(())
    }

    #[test]
    fn test_compute_metrics_perfect_conditions() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 10.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 2,
            queued_jobs: 0,
            performance_score: 0.99,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_compute_metrics_all_zero() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            memory_available_bytes: 0,
            active_containers: 0,
            queued_jobs: 0,
            performance_score: 0.0,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.total_memory_bytes(), 0);
        assert_eq!(metrics.memory_usage_percent(), 0.0);
        assert!(!metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Healthy);
    }

    #[test]
    fn compute_metrics_json_roundtrip_preserves_fields() {
        let m = ComputeMetrics {
            cpu_usage_percent: 12.5,
            memory_usage_bytes: 100,
            memory_available_bytes: 300,
            active_containers: 3,
            queued_jobs: 1,
            performance_score: 0.9,
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&m).expect("serialize");
        let back: ComputeMetrics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.cpu_usage_percent, m.cpu_usage_percent);
        assert_eq!(back.total_memory_bytes(), m.total_memory_bytes());
        assert_eq!(back.active_containers, m.active_containers);
        assert_eq!(back.queued_jobs, m.queued_jobs);
    }

    #[test]
    fn health_status_json_roundtrip() {
        for status in [HealthStatus::Healthy, HealthStatus::Degraded, HealthStatus::Unhealthy] {
            let json = serde_json::to_string(&status).expect("serialize");
            let back: HealthStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, status);
        }
    }

    #[test]
    fn degraded_health_not_high_load_boundary() {
        let m = ComputeMetrics {
            cpu_usage_percent: 85.0,
            memory_usage_bytes: 4_000_000_000,
            memory_available_bytes: 6_000_000_000,
            active_containers: 1,
            queued_jobs: 10,
            performance_score: 0.5,
            timestamp: chrono::Utc::now(),
        };
        assert!(m.is_high_load());
        assert_eq!(m.health_status(), HealthStatus::Degraded);
    }

    // --- ComputeAdapter protocol detection & discovery (no live services) ---

    #[tokio::test]
    async fn test_compute_adapter_new_tarpc_localhost_port() -> SongbirdResult<()> {
        let adapter = ComputeAdapter::new("tarpc://localhost:1234".to_string()).await?;
        assert_eq!(adapter.endpoint(), "tarpc://localhost:1234");
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_new_unix_tmp_test_sock() -> SongbirdResult<()> {
        let adapter = ComputeAdapter::new("unix:///tmp/test.sock".to_string()).await?;
        assert_eq!(adapter.endpoint(), "unix:///tmp/test.sock");
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_new_tarpc_invalid_hostname_err() {
        let err = ComputeAdapter::new("tarpc://test:1234".to_string())
            .await
            .expect_err("tarpc hostname must be localhost or IP");
        assert!(
            err.to_string().contains("Invalid hostname")
                || err.to_string().contains("configuration"),
            "unexpected: {err}"
        );
    }

    #[tokio::test]
    async fn test_compute_adapter_from_discovery_resolver_injected_tarpc() -> SongbirdResult<()> {
        let mut m = HashMap::new();
        m.insert(CapabilityType::Compute, "tarpc://127.0.0.1:9102".to_string());
        let adapter = ComputeAdapter::new_from_discovery_with_resolver(
            CapabilityEndpointResolver::with_endpoint_overrides(m),
        )
        .await?;
        assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9102");
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_from_discovery_resolver_injected_unix() -> SongbirdResult<()> {
        let mut m = HashMap::new();
        m.insert(CapabilityType::Compute, "unix:///tmp/injected-compute.sock".to_string());
        let adapter = ComputeAdapter::new_from_discovery_with_resolver(
            CapabilityEndpointResolver::with_endpoint_overrides(m),
        )
        .await?;
        assert_eq!(adapter.endpoint(), "unix:///tmp/injected-compute.sock");
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_from_discovery_fallback_songbird_compute_endpoint()
    -> SongbirdResult<()> {
        let _g = lock_discovery_env();
        songbird_process_env::reset_overlay();
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
        songbird_process_env::set_var(
            "SONGBIRD_COMPUTE_ENDPOINT",
            "http://from-songbird-compute:6688",
        );

        let adapter =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await
                .expect("adapter from SONGBIRD_COMPUTE_ENDPOINT");
        assert_eq!(adapter.endpoint(), "http://from-songbird-compute:6688");

        songbird_process_env::reset_overlay();
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_from_discovery_fallback_compute_capability_endpoint()
    -> SongbirdResult<()> {
        let _g = lock_discovery_env();
        songbird_process_env::reset_overlay();
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
        songbird_process_env::set_var(
            "COMPUTE_CAPABILITY_ENDPOINT",
            "http://from-legacy-compute:6699",
        );

        let adapter =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await
                .expect("adapter from COMPUTE_CAPABILITY_ENDPOINT");
        assert_eq!(adapter.endpoint(), "http://from-legacy-compute:6699");

        songbird_process_env::reset_overlay();
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_from_discovery_fallback_toadstool_endpoint() -> SongbirdResult<()>
    {
        let _g = lock_discovery_env();
        songbird_process_env::reset_overlay();
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
        songbird_process_env::set_var("TOADSTOOL_ENDPOINT", "http://from-toadstool:6600");

        let adapter =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await
                .expect("adapter from TOADSTOOL_ENDPOINT");
        assert_eq!(adapter.endpoint(), "http://from-toadstool:6600");

        songbird_process_env::reset_overlay();
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_from_discovery_fallback_host_and_port_env() -> SongbirdResult<()>
    {
        let _g = lock_discovery_env();
        songbird_process_env::reset_overlay();
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
        songbird_process_env::set_var("SONGBIRD_HOST", "http://custom-compute-host");
        songbird_process_env::set_var("SONGBIRD_COMPUTE_PORT", "9922");

        let adapter =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await
                .expect("adapter from host+port fallback");
        assert_eq!(adapter.endpoint(), "http://custom-compute-host:9922");

        songbird_process_env::reset_overlay();
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_from_discovery_fallback_prefers_songbird_compute_env()
    -> SongbirdResult<()> {
        let _g = lock_discovery_env();
        songbird_process_env::reset_overlay();
        songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
        songbird_process_env::set_var("SONGBIRD_COMPUTE_ENDPOINT", "http://songbird-wins:1111");
        songbird_process_env::set_var("COMPUTE_CAPABILITY_ENDPOINT", "http://legacy-loses:2222");

        let adapter =
            ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::new())
                .await
                .expect("adapter");
        assert_eq!(adapter.endpoint(), "http://songbird-wins:1111");

        songbird_process_env::reset_overlay();
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_adapter_with_timeout_and_endpoint_tarpc() -> SongbirdResult<()> {
        let adapter = ComputeAdapter::new("tarpc://127.0.0.1:9000".to_string())
            .await?
            .with_timeout(Duration::from_millis(350));
        assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9000");
        assert_eq!(adapter.timeout, Duration::from_millis(350));
        Ok(())
    }
}
