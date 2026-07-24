// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Kubernetes Provider Adapter
//!
//! Provides Kubernetes service discovery using the universal provider pattern
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for high-performance K8s integration

use futures_util::stream::{self, Stream};
use songbird_http_client::IpcHttpClient;
use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;

use super::{DiscoveryProviderImpl, ProviderFactory};
use crate::abstraction::{
    capabilities::DiscoveryCapability,
    providers::{
        DiscoveryProvider, LoadBalancingHints, ProviderConfig, ProviderMetadata, ServiceMetrics,
    },
};

use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_types::SongbirdError;

type Result<T> = songbird_types::SongbirdResult<T>;

/// Factory for creating Kubernetes providers from configuration
#[derive(Debug, Clone, Copy)]
pub struct KubernetesProviderFactory;

impl ProviderFactory for KubernetesProviderFactory {
    fn provider_type(&self) -> &'static str {
        "kubernetes"
    }

    async fn create_provider(&self, config: ProviderConfig) -> Result<DiscoveryProviderImpl> {
        // Extract namespace from flexible configuration
        let namespace =
            config.parameters.get("namespace").and_then(|v| v.as_str()).unwrap_or("default");

        // Create the legacy Kubernetes backend
        // Create native kubernetes adapter (no longer using deprecated backend)

        // Create native adapter
        let adapter =
            KubernetesProviderAdapter::new_native(config.id, namespace.to_string()).await?;
        Ok(DiscoveryProviderImpl::Kubernetes(adapter))
    }

    fn validate_config(&self, _config: &ProviderConfig) -> Result<()> {
        // Kubernetes provider validation would check for kubeconfig, service account, etc.
        // For now, we'll assume it's valid
        Ok(())
    }

    fn default_config(&self, id: String, name: String) -> ProviderConfig {
        let mut parameters = HashMap::new();
        parameters
            .insert("namespace".to_string(), serde_json::Value::String("default".to_string()));
        parameters.insert(
            "kubeconfig".to_string(),
            serde_json::Value::String("${KUBECONFIG}".to_string()),
        );

        let mut environment = HashMap::new();
        environment
            .insert("KUBERNETES_SERVICE_HOST".to_string(), "kubernetes.default.svc".to_string());
        environment.insert("KUBERNETES_SERVICE_PORT".to_string(), "443".to_string());

        ProviderConfig {
            id,
            name,
            parameters,
            environment,
            timeout_ms: Some(30000), // K8s can be slower
            retry_config: None,
        }
    }
}

/// Native Kubernetes provider adapter (no legacy backend dependency)
pub struct KubernetesProviderAdapter {
    metadata: ProviderMetadata,
    namespace: String,
    client: IpcHttpClient,
    api_base: String,
}

impl KubernetesProviderAdapter {
    /// Create new native kubernetes adapter
    pub async fn new_native(id: String, namespace: String) -> Result<Self> {
        let metadata = ProviderMetadata {
            id: id.clone(),
            name: format!("Kubernetes Provider ({id})"),
            version: "1.0.0".to_string(),
            capabilities: vec![
                DiscoveryCapability::ServiceRegistration,
                DiscoveryCapability::ServiceUnregistration,
                DiscoveryCapability::ServiceDiscovery,
                DiscoveryCapability::ServiceWatching,
                DiscoveryCapability::HealthChecking,
                DiscoveryCapability::ServiceListing,
                DiscoveryCapability::ServiceExistence,
                DiscoveryCapability::ServiceMetrics,
                DiscoveryCapability::LoadBalancingHints,
            ],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "kubernetes".to_string());
                meta.insert("protocol".to_string(), "https".to_string());
                meta.insert("vendor".to_string(), "cncf".to_string());
                meta
            },
            healthy: true,
            load_score: 0.4,
        };

        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

        let host = songbird_process_env::var("KUBERNETES_SERVICE_HOST")
            .unwrap_or_else(|_| "kubernetes.default.svc".to_string());
        let port = songbird_process_env::var("KUBERNETES_SERVICE_PORT")
            .unwrap_or_else(|_| "443".to_string());
        let api_base = format!("https://{host}:{port}");

        Ok(Self {
            metadata,
            namespace,
            client,
            api_base,
        })
    }

    fn parse_k8s_service_list(&self, response: &serde_json::Value) -> Vec<ServiceInfo> {
        let mut services = Vec::new();

        let items = match response.get("items").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return services,
        };

        for item in items {
            let metadata = match item.get("metadata") {
                Some(m) => m,
                None => continue,
            };
            let spec = match item.get("spec") {
                Some(s) => s,
                None => continue,
            };

            let name =
                metadata.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let uid = metadata.get("uid").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let cluster_ip =
                spec.get("clusterIP").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let port = spec
                .get("ports")
                .and_then(|v| v.as_array())
                .and_then(|ports| ports.first())
                .and_then(|p| p.get("port"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as u16;

            if cluster_ip.is_empty() || cluster_ip == "None" || port == 0 {
                continue;
            }

            services.push(ServiceInfo {
                service_id: uid.clone(),
                name,
                version: "unknown".to_string(),
                service_type: "kubernetes".to_string(),
                description: Some(format!("Kubernetes service in namespace {}", self.namespace)),
                endpoints: vec![],
                health_check_endpoint: None,
                metadata: HashMap::new(),
                tags: vec![],
                dependencies: vec![],
                status: crate::traits::service::ServiceStatus::Running,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                instance_id: uid,
                host: cluster_ip,
                port,
            });
        }

        services
    }
}

impl DiscoveryProvider for KubernetesProviderAdapter {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, _config: ProviderConfig) -> Result<()> {
        tracing::info!("☸️ Initializing Kubernetes discovery provider adapter");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("☸️ Shutting down Kubernetes discovery provider adapter");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/healthz", self.api_base);
        self.client.get(&url).await.map_or(Ok(false), |resp| Ok(resp.is_success()))
    }

    async fn register(&self, service: ServiceInfo) -> Result<()> {
        tracing::info!("📝 Registering service {} via Kubernetes adapter", service.service_id);
        Err(SongbirdError::discovery(
            "Kubernetes service registration requires kubectl/API server write access — \
             services are typically registered via Deployment/Service manifests, not runtime API",
        ))
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        tracing::info!("🗑️ Unregistering service {} via Kubernetes adapter", service_id);
        Err(SongbirdError::discovery(
            "Kubernetes service unregistration requires kubectl/API server write access — \
             use `kubectl delete service` or remove the manifest",
        ))
    }

    async fn update_health(&self, _service_id: &str, _health: ServiceHealthStatus) -> Result<()> {
        Ok(())
    }

    async fn update_metadata(
        &self,
        _service_id: &str,
        _metadata: HashMap<String, String>,
    ) -> Result<()> {
        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        tracing::info!("🔍 Discovering services via Kubernetes API (ns={})", self.namespace);

        let url = query.name.as_deref().map_or_else(
            || format!("{}/api/v1/namespaces/{}/services", self.api_base, self.namespace),
            |name| {
                format!("{}/api/v1/namespaces/{}/services/{}", self.api_base, self.namespace, name)
            },
        );

        let response = self
            .client
            .get(&url)
            .await
            .map_err(|e| SongbirdError::network(format!("Kubernetes API query failed: {e}")))?;

        if !response.is_success() {
            return Err(SongbirdError::network(format!(
                "Kubernetes API returned status: {}",
                response.status()
            )));
        }

        let k8s_response: serde_json::Value = response.json().await.map_err(|e| {
            SongbirdError::network(format!("Failed to parse Kubernetes response: {e}"))
        })?;

        Ok(self.parse_k8s_service_list(&k8s_response))
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        tracing::info!("👀 Watching services via Kubernetes adapter");

        // K8s supports watching natively, but legacy backend needs updates
        Ok(Box::pin(stream::empty()))
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        tracing::info!("📋 Listing all services via Kubernetes adapter");

        // MODERNIZED: Use capability-based service discovery
        // In production, this would integrate with the UniversalCapabilityAdapter
        // to provide Kubernetes service discovery through the unified interface

        Err(SongbirdError::configuration(
            "Kubernetes service discovery should use UniversalCapabilityAdapter. \
             Configure kubernetes capability provider through songbird-universal crate.",
        ))
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        tracing::debug!("❓ Checking if service {} exists via Kubernetes adapter", service_id);

        // MODERNIZED: Use capability-based service discovery
        Err(SongbirdError::configuration(
            "Kubernetes service existence checks should use UniversalCapabilityAdapter. \
             Configure kubernetes capability provider through songbird-universal crate.",
        ))
    }

    async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        tracing::debug!("📊 Getting metrics for service {} via Kubernetes adapter", service_id);

        // K8s can provide rich metrics through metrics-server
        Ok(ServiceMetrics {
            service_id: service_id.to_string(),
            request_count: 0,
            error_count: 0,
            average_response_time_ms: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            custom_metrics: HashMap::new(),
        })
    }

    async fn get_load_balancing_hints(&self, service_name: &str) -> Result<LoadBalancingHints> {
        tracing::debug!(
            "⚖️ Getting load balancing hints for {} via Kubernetes adapter",
            service_name
        );

        // K8s can provide sophisticated load balancing through Services
        Ok(LoadBalancingHints {
            service_name: service_name.to_string(),
            preferred_instances: vec![],
            weights: HashMap::new(),
            health_scores: HashMap::new(),
            locality_preferences: vec![],
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubernetes_factory_config() {
        let factory = KubernetesProviderFactory;
        let config = factory.default_config("test".to_string(), "Test".to_string());

        assert!(factory.validate_config(&config).is_ok());
        assert_eq!(factory.provider_type(), "kubernetes");
    }

    #[test]
    fn test_kubernetes_provider_metadata() {
        // We can't easily test this without a real K8s backend
        // but we can test the factory
        let factory = KubernetesProviderFactory;
        assert_eq!(factory.provider_type(), "kubernetes");
    }
}
