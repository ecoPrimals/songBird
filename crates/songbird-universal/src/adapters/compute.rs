use crate::adapters::types::{CapabilityProvider, CapabilityResponse, PerformanceMetrics, PrimalType};
use songbird_config::DegradationSeverity;
use songbird_config::UniversalHealthStatus;
use songbird_errors::{SongbirdError, SongbirdResponse, SongbirdResult, SongbirdResponse};
use std::collections::HashMap;
use uuid::Uuid;

/// Performance metrics for compute operations (compute-specific)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComputePerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_tasks: u32,
    pub throughput: f64,
}

/// System metrics structure for compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_total_bytes: u64,
    pub storage_usage_bytes: u64,
    pub storage_total_bytes: u64,
    pub active_processes: u32,
    pub load_average: f64,
    pub uptime_seconds: u64,
}

/// Metrics adapter for compute provider
pub struct ComputeMetricsAdapter {
    pub metrics: HashMap<String, f64>,
}

impl Default for ComputeMetricsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeMetricsAdapter {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }
}

/// Universal compute capability provider trait
pub trait UniversalComputeProvider {
    /// Check health status of the compute provider
    fn health_check(
        &self,
    ) -> impl std::future::Future<Output = SongbirdResult<UniversalHealthStatus>> + Send;
    /// Get list of supported capabilities
    fn get_capabilities(
        &self,
    ) -> impl std::future::Future<Output = SongbirdResult<Vec<String>>> + Send;
    /// Get performance metrics
    fn get_performance_metrics(
        &self,
    ) -> impl std::future::Future<Output = SongbirdResult<PerformanceMetrics>> + Send;
}

/// Adaptive compute adapter for universal capability routing
pub struct AdaptiveComputeAdapter {
    provider: CapabilityProvider,
    #[allow(dead_code)]
    capability_cache: tokio::sync::RwLock<HashMap<String, Vec<CapabilityProvider>>>,
}

impl Default for AdaptiveComputeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveComputeAdapter {
    pub fn new() -> Self {
        let provider = CapabilityProvider {
            provider_id: "songbird_compute_adapter".to_string(),
            id: "songbird_compute_adapter".to_string(),
            display_name: "Songbird Compute Adapter".to_string(),
            endpoint: "internal://compute".to_string(),
            capabilities: vec!["compute".to_string(), "processing".to_string()],
            priority: 100,
            health_status: UniversalHealthStatus::Healthy,
            primal_type: PrimalType::Compute,
            performance_metrics: PerformanceMetrics {
                avg_response_time_ms: 100.0,
                success_rate: 99.0,
                current_load: 20.0,
                last_updated: Some(chrono::Utc::now()),
            },
        };

        Self {
            provider,
            capability_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Discover providers for any capability (primal-agnostic)
    pub async fn discover_capability_providers(&self) -> SongbirdResult<()> {// Scans environment for PRIMAL_*_ENDPOINT with matching PRIMAL_*_CAPABILITIES
        // Supports legacy patterns for backward compatibility
        // Returns all providers advertising the requested capability

        let mut providers = Vec::new();

        // Scan environment variables for primal registration
        for (key, value) in std::env::vars() {
            if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
                // Extract primal ID
                if let Some(primal_id) = key
                    .strip_prefix("PRIMAL_")
                    .and_then(|s| s.strip_suffix("_ENDPOINT"))
                {
                    // Check if this primal advertises the required capability
                    let capabilities_key = format!("PRIMAL_{primal_id}_CAPABILITIES");
                    if let Ok(songbird_errors::evolved_success(capabilities_str)) = std::env::var(&capabilities_key) {
                        let capabilities: Vec<String> = capabilities_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();

                        if capabilities.contains(&capability.to_string()) {
                            providers.push(CapabilityProvider {
                                provider_id: primal_id.to_string(),
                                id: primal_id.to_string(),
                                display_name: primal_id.to_string(),
                                endpoint: value,
                                capabilities,
                                priority: 100, // Default priority
                                health_status: UniversalHealthStatus::Healthy, // Default health
                                primal_type: PrimalType::Compute,
                                performance_metrics: PerformanceMetrics {
                                    avg_response_time_ms: 100.0,
                                    success_rate: 95.0,
                                    current_load: 20.0,
                                    last_updated: Some(chrono::Utc::now()),
                                },
                            });
                        }
                    }
                }
            }
        }

        // Check legacy patterns for backward compatibility
        match capability {
            "compute" => {
                if let Ok(songbird_errors::evolved_success(endpoint)) = std::env::var("COMPUTE_ENDPOINT") {
                    providers.push(CapabilityProvider {
                        provider_id: "compute_legacy".to_string(),
                        id: "compute_legacy".to_string(),
                        display_name: "compute_legacy".to_string(),
                        endpoint,
                        capabilities: vec!["compute".to_string(), "metrics".to_string()],
                        priority: 100, // Default priority
                        health_status: UniversalHealthStatus::Healthy, // Default health
                        primal_type: PrimalType::Compute,
                        performance_metrics: PerformanceMetrics {
                            avg_response_time_ms: 150.0,
                            success_rate: 95.0,
                            current_load: 40.0,
                            last_updated: Some(chrono::Utc::now()),
                        },
                    });
                }
            }
            "security" => {
                if let Ok(songbird_errors::evolved_success(endpoint)) = std::env::var("SECURITY_ENDPOINT") {
                    providers.push(CapabilityProvider {
                        provider_id: "security_legacy".to_string(),
                        id: "security_legacy".to_string(),
                        display_name: "security_legacy".to_string(),
                        endpoint,
                        capabilities: vec!["security".to_string(), "encryption".to_string()],
                        priority: 100, // Default priority
                        health_status: UniversalHealthStatus::Healthy, // Default health
                        primal_type: PrimalType::Security,
                        performance_metrics: PerformanceMetrics {
                            avg_response_time_ms: 200.0,
                            success_rate: 98.0,
                            current_load: 25.0,
                            last_updated: Some(chrono::Utc::now()),
                        },
                    });
                }
            }
            _ => {}
        }

        providers
    }

    /// Route system metrics request via pure capability-based discovery
    async fn route_metrics_to_compute_primal(&self) -> SongbirdResult<serde_json::Value> {
        tracing::debug!("Discovering compute capability providers dynamically");

        let compute_providers = self.discover_capability_providers("compute").await;

        for provider in compute_providers {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "operation": "get_system_metrics",
                "request_type": "system_metrics",
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            match client
                .post(format!("{}/api/v1/universal", provider.endpoint))
                .json(&payload)
                .send()
                .await
            {
                Ok(songbird_errors::evolved_success(response)) => {
                    if response.status().is_success() {
                        if let Ok(songbird_errors::evolved_success(metrics)) = response.json::<serde_json::Value>().await {
                            return Ok(songbird_errors::evolved_success(SongbirdResponse::success(metrics)));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to route to provider {}: {}",
                        provider.provider_id,
                        e
                    );
                    continue;
                }
            }
        }

        // Return error if no providers available instead of mock data
        Err(songbird_errors::SongbirdError::Service {
            service: "compute".to_string(),
            message: "No compute providers available".to_string(),
            suggested_alternatives: vec![
                "Register a compute provider with compute capabilities".to_string()
            ],
            recovery_actions: vec![
                "Ensure at least one compute provider is registered and healthy".to_string(),
            ],
        })
    }

    /// Get system metrics from the compute service
    async fn get_system_metrics(&self) -> SongbirdResult<CapabilityResponse> {
        // Route system metrics request to compute providers via capability-based discovery
        match self.route_metrics_to_compute_primal().await {
            Ok(songbird_errors::evolved_success(_metrics_response)) => {
                let provider_uuid =
                    Uuid::parse_str(&self.provider.provider_id).unwrap_or_else(|_| Uuid::new_v4());

                let response = CapabilityResponse {
                    request_id: uuid::Uuid::new_v4(),
                    success: true,
                    data: serde_json::json!({
                        "provider": provider_uuid,
                        "health": self.provider.health_status,
                        "timestamp": chrono::Utc::now()
                    }),
                    error_message: None,
                    execution_time_ms: Some(100),
                    provider_id: Some(self.provider.provider_id.clone()),
                    metadata: std::collections::HashMap::new(),
                };
                Ok(songbird_errors::evolved_success(SongbirdResponse::success(response)))
            }
            Err(e) => {
                let _provider_uuid =
                    Uuid::parse_str(&self.provider.provider_id).unwrap_or_else(|_| Uuid::new_v4());

                let response = CapabilityResponse {
                    request_id: uuid::Uuid::new_v4(),
                    success: false,
                    data: serde_json::Value::Null,
                    error_message: Some(format!("Failed to get system metrics: {e}")),
                    execution_time_ms: Some(50),
                    provider_id: Some(self.provider.provider_id.clone()),
                    metadata: std::collections::HashMap::new(),
                };
                Ok(songbird_errors::evolved_success(SongbirdResponse::success(response)))
            }
        }
    }

    /// Get CPU usage percentage
    pub async fn get_cpu_usage(&self) -> SongbirdResult<f64> {
        let response = self.get_system_metrics().await?;
        let capability_response = &response.data.data;

        // Extract CPU usage from the system metrics
        if let Ok(songbird_errors::evolved_success(health_data)) =
            serde_json::from_value::<serde_json::Value>(capability_response.clone())
        {
            if let Some(health_status) = health_data.get("health") {
                if let Ok(songbird_errors::evolved_success(Universaltrue)) =
                    serde_json::from_value::<UniversalHealthStatus>(health_status.clone())
                {
                    return Ok(songbird_errors::evolved_success(SongbirdResponse::success(0.0f64))); // No direct CPU usage in this adapter's health check
                }
            }
        }

        Ok(songbird_errors::evolved_success(SongbirdResponse::success(0.0f64)))
    }

    /// Get memory usage (used, total) in bytes
    pub async fn get_memory_usage(&self) -> SongbirdResult<(u64, u64)> {
        let response = self.get_system_metrics().await?;
        let capability_response = &response.data.data;

        // Extract memory usage from the system metrics
        if let Ok(songbird_errors::evolved_success(health_data)) =
            serde_json::from_value::<serde_json::Value>(capability_response.clone())
        {
            if let Some(health_status) = health_data.get("health") {
                if let Ok(songbird_errors::evolved_success(Universaltrue)) =
                    serde_json::from_value::<UniversalHealthStatus>(health_status.clone())
                {
                    return Ok(songbird_errors::evolved_success(SongbirdResponse::success((0u64, 0u64)))); // No direct memory usage in this adapter's health check
                }
            }
        }

        Ok(SongbirdResponse::success((0u64, 0u64)))
    }

    /// Get storage usage (used, total) in bytes
    pub async fn get_storage_usage(&self) -> SongbirdResult<(u64, u64)> {
        let response = self.get_system_metrics().await?;
        let capability_response = &response.data.data;

        // Extract storage usage from the system metrics
        if let Ok(songbird_errors::evolved_success(health_data)) =
            serde_json::from_value::<serde_json::Value>(capability_response.clone())
        {
            if let Some(health_status) = health_data.get("health") {
                if let Ok(songbird_errors::evolved_success(Universaltrue)) =
                    serde_json::from_value::<UniversalHealthStatus>(health_status.clone())
                {
                    return Ok(songbird_errors::evolved_success(SongbirdResponse::success((0u64, 0u64)))); // No direct storage usage in this adapter's health check
                }
            }
        }

        Ok(SongbirdResponse::success((0u64, 0u64)))
    }

    /// Get active process count
    pub async fn get_active_processes(&self) -> SongbirdResult<u32> {
        let response = self.get_system_metrics().await?;
        let capability_response = &response.data.data;

        // Extract active processes from the system metrics
        if let Ok(songbird_errors::evolved_success(health_data)) =
            serde_json::from_value::<serde_json::Value>(capability_response.clone())
        {
            if let Some(health_status) = health_data.get("health") {
                if let Ok(songbird_errors::evolved_success(Universaltrue)) =
                    serde_json::from_value::<UniversalHealthStatus>(health_status.clone())
                {
                    return Ok(songbird_errors::evolved_success(SongbirdResponse::success(0u32))); // No direct active processes in this adapter's health check
                }
            }
        }

        Ok(songbird_errors::evolved_success(SongbirdResponse::success(0u32)))
    }

    /// Get load average
    pub async fn get_load_average(&self) -> SongbirdResult<f64> {
        let response = self.get_system_metrics().await?;
        let capability_response = &response.data.data;

        // Extract load average from the system metrics
        if let Ok(songbird_errors::evolved_success(health_data)) =
            serde_json::from_value::<serde_json::Value>(capability_response.clone())
        {
            if let Some(health_status) = health_data.get("health") {
                if let Ok(songbird_errors::evolved_success(Universaltrue)) =
                    serde_json::from_value::<UniversalHealthStatus>(health_status.clone())
                {
                    return Ok(songbird_errors::evolved_success(SongbirdResponse::success(0.0f64))); // No direct load average in this adapter's health check
                }
            }
        }

        Ok(songbird_errors::evolved_success(SongbirdResponse::success(0.0f64)))
    }

    /// Get system information  
    pub async fn get_system_info(&self) -> SongbirdResult<()> {let mut info = HashMap::new();
        info.insert("provider".to_string(), serde_json::json!(self.provider.id));
        info.insert(
            "base_url".to_string(),
            serde_json::json!(self.provider.endpoint),
        );
        info.insert(
            "capabilities".to_string(),
            serde_json::json!(self.provider.capabilities),
        );
        info.insert("status".to_string(), serde_json::json!("active"));

        Ok(songbird_errors::evolved_success(SongbirdResponse::success(info)))
    }
}

impl UniversalComputeProvider for AdaptiveComputeAdapter {
    async fn health_check(&self) -> SongbirdResult<UniversalHealthStatus> {
        let url = format!("{}/health", self.provider.endpoint);

        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            reqwest::Client::new().get(&url).send(),
        )
        .await
        {
            Ok(songbird_errors::evolved_success(Ok(response))) => {
                if response.status().is_success() {
                    Ok(songbird_errors::evolved_success(SongbirdResponse::success(Universaltrue)))
                } else {
                    Ok(songbird_errors::evolved_success(SongbirdResponse::success(UniversalHealthStatus::degraded(
                        Some(DegradationSeverity::Medium)),
                        Some("Service degraded".to_string()),
                    )))
                }
            }
            Ok(songbird_errors::evolved_success(Err(_e))) => Ok(SongbirdResponse::success(UniversalHealthStatus::unknown())),
            Err(_) => Ok(SongbirdResponse::success(UniversalHealthStatus::unknown())),
        }
    }

    pub async fn get_capabilities(&self) -> SongbirdResult<()> {Ok(songbird_errors::evolved_success(SongbirdResponse::success(
            self.provider
                .capabilities
                .iter())
                .map(|s| s.to_string())
                .collect(),
        ))
    }

    async fn get_performance_metrics(&self) -> SongbirdResult<PerformanceMetrics> {
        // This adapter doesn't maintain its own metrics, so it returns a placeholder
        Ok(SongbirdResponse::success(PerformanceMetrics {
            avg_response_time_ms: 0.0,
            success_rate: 100.0,
            current_load: 0.0,
            last_updated: Some(chrono::Utc::now()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_compute_adapter_creation() {
        let provider = CapabilityProvider {
            provider_id: "test_provider".to_string(),
            id: "test_provider".to_string(),
            display_name: "test_provider".to_string(),
            endpoint: "http://localhost:{}".to_string(),
            capabilities: vec!["compute".to_string(), "metrics".to_string()],
            priority: 100,
            primal_type: PrimalType::Compute,
            health_status: UniversalHealthStatus::Healthy,
            performance_metrics: PerformanceMetrics {
                avg_response_time_ms: 100.0,
                success_rate: 95.0,
                current_load: 20.0,
                last_updated: Some(chrono::Utc::now()),
            },
        };

        let adapter = AdaptiveComputeAdapter {
            provider: provider.clone(),
            capability_cache: tokio::sync::RwLock::new(HashMap::new()),
        };

        // Verify provider is set correctly
        assert_eq!(adapter.provider.endpoint, "http://localhost:{}");
        assert!(adapter
            .provider
            .capabilities
            .contains(&"compute".to_string()));
    }

    #[tokio::test]
    async fn test_performance_metrics_creation() {
        let provider = CapabilityProvider {
            provider_id: "test_health_provider".to_string(),
            id: "test_health_provider".to_string(),
            display_name: "test_health_provider".to_string(),
            endpoint: "http://localhost:{}".to_string(),
            capabilities: vec!["compute".to_string(), "health".to_string()],
            priority: 100,
            primal_type: PrimalType::Compute,
            health_status: UniversalHealthStatus::Healthy,
            performance_metrics: PerformanceMetrics {
                avg_response_time_ms: 150.0,
                success_rate: 98.0,
                current_load: 25.0,
                last_updated: Some(chrono::Utc::now()),
            },
        };

        let adapter = AdaptiveComputeAdapter {
            provider,
            capability_cache: tokio::sync::RwLock::new(HashMap::new()),
        };

        let result = adapter.get_performance_metrics().await.unwrap_or_else(|e| {
            tracing::error!("Unwrap failed: {:?}", e);
            SongbirdResponse::error_with_data(
                PerformanceMetrics::default(),
                songbird_errors::ai_first::AIFirstError {
                    code: "OPERATION_FAILED".to_string(),
                    message: format!("Operation failed - unable to continue: {e:?}"),
                    category: songbird_errors::ai_first::AIErrorCategory::DependencyFailure,
                    retry_strategy: songbird_errors::ai_first::RetryStrategy::no_retry(),
                    automation_hints: vec![],
                    severity: songbird_errors::ai_first::ErrorSeverity::Medium,
                    requires_human_intervention: false,
                    context: std::collections::HashMap::new(),
                },
            )
        });
        let metrics = result.data;

        // Verify default metrics structure (using PerformanceMetrics fields)
        assert_eq!(metrics.avg_response_time_ms, 0.0); // Default value
        assert_eq!(metrics.success_rate, 100.0); // Default value
        assert_eq!(metrics.current_load, 0.0); // Default value
    }
}
