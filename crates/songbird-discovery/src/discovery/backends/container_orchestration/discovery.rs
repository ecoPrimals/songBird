// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Aggregated service discovery across orchestration methods and `ServiceInfo` construction helpers.

use super::types::{OrchestrationMethod, UniversalContainerOrchestration};
use crate::traits::ServiceQuery;
use crate::traits::service::{ServiceInfo, ServiceStatus};
use chrono::Utc;
use songbird_types::errors::SongbirdResult;
use std::collections::HashMap;

impl UniversalContainerOrchestration {
    /// Discover services using all available orchestration methods
    pub(super) async fn discover_services_universal(
        &self,
        query: ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut all_services = Vec::new();

        for method in &self.orchestration_methods {
            match method {
                OrchestrationMethod::KubernetesApi {
                    endpoint,
                    namespace,
                } => {
                    if let Ok(services) =
                        self.discover_from_kubernetes_api(endpoint, namespace, &query).await
                    {
                        all_services.extend(services);
                    }
                }
                OrchestrationMethod::DockerApi {
                    endpoint,
                } => {
                    if let Ok(services) = self.discover_from_docker_api(endpoint, &query).await {
                        all_services.extend(services);
                    }
                }
                OrchestrationMethod::ContainerEnvironment => {
                    if let Ok(services) = self.discover_from_container_environment(&query).await {
                        all_services.extend(services);
                    }
                }
                OrchestrationMethod::ProcessBased => {
                    if let Ok(services) = self.discover_from_process_based(&query).await {
                        all_services.extend(services);
                    }
                }
            }
        }

        // Deduplicate services by service_id
        let mut unique_services = HashMap::new();
        for service in all_services {
            unique_services.insert(service.service_id.clone(), service);
        }

        Ok(unique_services.into_values().collect())
    }

    /// Create a `ServiceInfo` from discovered service data
    pub(super) fn create_service_info(&self, name: &str, orchestration_type: &str) -> ServiceInfo {
        ServiceInfo {
            service_id: format!("{}-{}", name, uuid::Uuid::new_v4()),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: orchestration_type.to_string(),
            description: Some(format!("Service discovered via {orchestration_type}")),
            endpoints: Vec::new(), // Would be populated from actual discovery
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![orchestration_type.to_string(), "container".to_string()],
            dependencies: Vec::new(),
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: format!("{name}-instance"),
            host: songbird_config::canonical::constants::network::default_host(),
            port: 8080,
        }
    }
}
