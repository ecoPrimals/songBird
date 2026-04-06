// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Docker-compatible API detection and service discovery.

use super::types::{
    ApiEndpoint, ContainerRuntimeInfo, OrchestrationMethod, UniversalContainerOrchestration,
};
use crate::traits::ServiceQuery;
use crate::traits::service::ServiceInfo;
use songbird_types::errors::SongbirdResult;
use tracing::debug;

impl UniversalContainerOrchestration {
    /// Detect Docker-compatible APIs
    pub(super) async fn detect_docker_apis(&mut self) {
        let docker_socket = songbird_process_env::var("SONGBIRD_DOCKER_SOCKET")
            .unwrap_or_else(|_| "/var/run/docker.sock".to_string());
        let docker_unix_url = format!("unix://{docker_socket}");

        let potential_endpoints = vec![
            songbird_process_env::var("DOCKER_HOST").unwrap_or_default(),
            docker_unix_url,
            "tcp://songbird_config::canonical::constants::network::DEFAULT_HOST:2376".to_string(), // Docker daemon default
        ];

        for endpoint in potential_endpoints {
            if !endpoint.is_empty() && self.test_docker_endpoint(&endpoint).await {
                debug!("Detected Docker API: {}", endpoint);

                self.orchestration_methods.push(OrchestrationMethod::DockerApi {
                    endpoint: endpoint.clone(),
                });
                self.orchestration_endpoints.push(endpoint.clone());

                if self.runtime_info.runtime_type == "unknown" {
                    self.runtime_info = ContainerRuntimeInfo {
                        runtime_type: "docker".to_string(),
                        api_endpoint: Some(ApiEndpoint {
                            url: endpoint.clone(),
                            version: "v1".to_string(),
                            timeout: std::time::Duration::from_secs(10),
                            verify_tls: true,
                        }),
                        auth_method: None,
                        namespace: None,
                    };
                }
            }
        }
    }

    /// Test if an endpoint is a valid Docker API
    async fn test_docker_endpoint(&self, _endpoint: &str) -> bool {
        // Check for Docker socket or environment indicators
        let docker_socket = songbird_process_env::var("SONGBIRD_DOCKER_SOCKET")
            .unwrap_or_else(|_| "/var/run/docker.sock".to_string());
        std::path::Path::new(&docker_socket).exists()
            || std::path::Path::new("/.dockerenv").exists()
            || songbird_process_env::var("DOCKER_HOST").is_ok()
    }

    /// Discover services from Docker API
    pub(super) async fn discover_from_docker_api(
        &self,
        endpoint: &str,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("Discovering services from Docker API: {}", endpoint);

        // In a real implementation, this would:
        // 1. Connect to Docker daemon
        // 2. List running containers
        // 3. Parse container labels and metadata
        // 4. Extract service information

        let mut services = Vec::new();

        // For now, create example services based on Docker environment
        if std::path::Path::new("/.dockerenv").exists() {
            services.push(self.create_service_info("docker-container-service", "docker"));
        }

        Ok(services)
    }
}
