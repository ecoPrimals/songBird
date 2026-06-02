// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Container environment heuristics (env vars, filesystem, PID 1) and discovery from those signals.

use super::types::{OrchestrationMethod, UniversalContainerOrchestration};
use crate::traits::ServiceQuery;
use crate::traits::service::ServiceInfo;
use songbird_types::errors::SongbirdResult;
use std::collections::HashMap;
use tracing::debug;

impl UniversalContainerOrchestration {
    /// Detect container environment
    pub(super) fn detect_container_environment(&mut self) {
        // Check for various container environment indicators
        let container_indicators = vec![
            ("KUBERNETES_SERVICE_HOST", "kubernetes"),
            ("DOCKER_HOST", "docker"),
            ("CONTAINER", "generic"),
        ];

        for (env_var, runtime) in container_indicators {
            if songbird_process_env::var(env_var).is_ok() {
                debug!("Detected container environment: {} ({})", runtime, env_var);
                if self
                    .orchestration_methods
                    .iter()
                    .all(|m| !matches!(m, OrchestrationMethod::ContainerEnvironment))
                {
                    self.orchestration_methods.push(OrchestrationMethod::ContainerEnvironment);
                }

                if self.runtime_info.runtime_type == "unknown" {
                    self.runtime_info.runtime_type = runtime.to_string();
                }
                break;
            }
        }

        // Check for container filesystem indicators
        let container_files = vec!["/.dockerenv", "/run/.containerenv", "/proc/1/cgroup"];

        for file_path in container_files {
            if std::path::Path::new(file_path).exists() {
                debug!("Detected container environment via filesystem: {}", file_path);
                if self
                    .orchestration_methods
                    .iter()
                    .all(|m| !matches!(m, OrchestrationMethod::ContainerEnvironment))
                {
                    self.orchestration_methods.push(OrchestrationMethod::ContainerEnvironment);
                }

                if self.runtime_info.runtime_type == "unknown" {
                    self.runtime_info.runtime_type = "container".to_string();
                }
                break;
            }
        }
    }

    /// Detect process-based containers
    pub(super) fn detect_process_based_containers(&mut self) {
        // Check if running as PID 1 (common in containers)
        if std::process::id() == 1 {
            debug!("Detected container environment: running as PID 1");
            self.orchestration_methods.push(OrchestrationMethod::ProcessBased);

            if self.runtime_info.runtime_type == "unknown" {
                self.runtime_info.runtime_type = "container".to_string();
            }
        }
    }

    /// Discover services from container environment
    pub(super) async fn discover_from_container_environment(
        &self,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut services = Vec::new();
        let env_vars = songbird_process_env::vars().collect::<HashMap<_, _>>();

        // Look for service-related environment variables common in containers
        for (key, _value) in env_vars {
            if key.ends_with("_SERVICE_HOST") || key.ends_with("_PORT") {
                let service_name =
                    key.replace("_SERVICE_HOST", "").replace("_PORT", "").to_lowercase();
                if !service_name.is_empty() {
                    services.push(self.create_service_info(&service_name, "container-env"));
                }
            }
        }

        Ok(services)
    }

    /// Discover services from process-based detection
    pub(super) async fn discover_from_process_based(
        &self,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        // In a real implementation, this would scan running processes
        // and identify services based on common patterns
        debug!("Process-based service discovery not yet implemented");
        Ok(Vec::new())
    }
}
