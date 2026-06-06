// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Container Orchestration Adapter
//!
//! Provides vendor-agnostic container orchestration discovery that can work with:
//! - Any Kubernetes-compatible system (K8s, K3s, `OpenShift`, etc.)
//! - Any Docker-compatible system (Docker, Podman, containerd, etc.)
//! - Any container runtime environment
//! - Any orchestration API that provides service information
//!
//! ## Native Async Traits
//! This module uses native async trait methods (Rust 1.75+) for zero-cost abstractions.

#![allow(
    async_fn_in_trait,
    clippy::unused_async,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::unused_self,
    reason = "async discovery traits: native async traits and adapter ergonomics"
)]

mod adapter;
mod discovery;
mod docker;
mod environment;
mod kubernetes;
mod trait_impl;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    ApiEndpoint, AuthenticationMethod, ContainerInfo, NamespaceConfig,
    UniversalContainerOrchestration,
};

#[cfg(test)]
mod environment_tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use std::collections::HashMap;

    use crate::traits::ServiceQuery;

    use super::types::{
        ContainerRuntimeInfo, OrchestrationMethod, UniversalContainerOrchestration,
    };

    fn blank_orchestration() -> UniversalContainerOrchestration {
        UniversalContainerOrchestration {
            orchestration_endpoints: Vec::new(),
            runtime_info: ContainerRuntimeInfo {
                runtime_type: "unknown".to_string(),
                api_endpoint: None,
                auth_method: None,
                namespace: None,
            },
            orchestration_methods: Vec::new(),
            discovered_containers: HashMap::new(),
        }
    }

    fn has_container_environment_method(orch: &UniversalContainerOrchestration) -> bool {
        orch.orchestration_methods
            .iter()
            .any(|m| matches!(m, OrchestrationMethod::ContainerEnvironment))
    }

    #[test]
    fn detect_container_environment_kubernetes_service_host() {
        songbird_process_env::set_var("KUBERNETES_SERVICE_HOST", "10.96.0.1");
        let mut orch = blank_orchestration();
        orch.detect_container_environment();
        assert!(has_container_environment_method(&orch));
        assert_eq!(orch.runtime_info.runtime_type, "kubernetes");
        songbird_process_env::remove_var("KUBERNETES_SERVICE_HOST");
    }

    #[test]
    fn detect_container_environment_docker_host() {
        songbird_process_env::remove_var("KUBERNETES_SERVICE_HOST");
        songbird_process_env::set_var("DOCKER_HOST", "unix:///var/run/docker.sock");
        let mut orch = blank_orchestration();
        orch.detect_container_environment();
        assert!(has_container_environment_method(&orch));
        assert_eq!(orch.runtime_info.runtime_type, "docker");
        songbird_process_env::remove_var("DOCKER_HOST");
    }

    #[test]
    fn detect_container_environment_generic_container_var() {
        songbird_process_env::remove_var("KUBERNETES_SERVICE_HOST");
        songbird_process_env::remove_var("DOCKER_HOST");
        songbird_process_env::set_var("CONTAINER", "podman");
        let mut orch = blank_orchestration();
        orch.detect_container_environment();
        assert!(has_container_environment_method(&orch));
        assert_eq!(orch.runtime_info.runtime_type, "generic");
        songbird_process_env::remove_var("CONTAINER");
    }

    #[test]
    fn detect_container_environment_no_env_vars_avoids_kubernetes_and_docker() {
        songbird_process_env::remove_var("KUBERNETES_SERVICE_HOST");
        songbird_process_env::remove_var("DOCKER_HOST");
        songbird_process_env::remove_var("CONTAINER");
        let mut orch = blank_orchestration();
        orch.detect_container_environment();
        assert_ne!(orch.runtime_info.runtime_type, "kubernetes");
        assert_ne!(orch.runtime_info.runtime_type, "docker");
    }

    #[test]
    fn detect_container_environment_does_not_duplicate_method() {
        songbird_process_env::set_var("KUBERNETES_SERVICE_HOST", "10.96.0.1");
        songbird_process_env::set_var("DOCKER_HOST", "unix:///var/run/docker.sock");
        let mut orch = blank_orchestration();
        orch.detect_container_environment();
        orch.detect_container_environment();
        let count = orch
            .orchestration_methods
            .iter()
            .filter(|m| matches!(m, OrchestrationMethod::ContainerEnvironment))
            .count();
        assert_eq!(count, 1);
        songbird_process_env::remove_var("KUBERNETES_SERVICE_HOST");
        songbird_process_env::remove_var("DOCKER_HOST");
    }

    #[tokio::test]
    async fn discover_from_container_environment_finds_service_host() {
        let var = "REDIS_SERVICE_HOST";
        songbird_process_env::set_var(var, "10.0.0.5");
        let orch = blank_orchestration();
        let services =
            orch.discover_from_container_environment(&ServiceQuery::new()).await.unwrap();
        assert!(services.iter().any(|s| s.name == "redis"));
        assert!(services.iter().any(|s| s.service_type == "container-env"));
        songbird_process_env::remove_var(var);
    }

    #[tokio::test]
    async fn discover_from_container_environment_finds_port_suffix() {
        let var = "POSTGRES_PORT";
        songbird_process_env::set_var(var, "5432");
        let orch = blank_orchestration();
        let services =
            orch.discover_from_container_environment(&ServiceQuery::new()).await.unwrap();
        assert!(services.iter().any(|s| s.name == "postgres"));
        songbird_process_env::remove_var(var);
    }
}
