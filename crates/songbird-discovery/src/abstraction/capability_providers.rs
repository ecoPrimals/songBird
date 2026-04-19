// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Capability-Based Provider System (Vendor Agnostic)
//!
//! This module provides vendor-agnostic access to discovery capabilities.
//! Instead of requesting "kubernetes" or "consul", you request capabilities
//! like "`container_orchestration`" or "`service_registry`".
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for zero-cost abstraction

use std::collections::HashMap;

use crate::abstraction::adapters::{DiscoveryProviderImpl, ProviderFactory, ProviderFactoryImpl};
use crate::abstraction::providers::ProviderConfig;
use songbird_types::{SongbirdError, SongbirdResult};

/// Capability types for vendor-agnostic discovery
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapabilityType {
    /// Container orchestration (kubernetes, docker-swarm, nomad, etc.)
    ContainerOrchestration,
    /// Service registry (consul, etcd, zookeeper, etc.)
    ServiceRegistry,
    /// Key-value store (etcd, consul kv, redis, etc.)
    KeyValueStore,
    /// Load balancer (nginx, haproxy, envoy, etc.)
    LoadBalancer,
    /// Service mesh (istio, linkerd, consul-connect, etc.)
    ServiceMesh,
    /// DNS-based discovery (coredns, bind, etc.)
    DnsDiscovery,
    /// Custom capability
    Custom(String),
}

impl CapabilityType {
    /// Parse a capability type from its string name.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "container_orchestration" | "containers" | "orchestration" => {
                Self::ContainerOrchestration
            }
            "service_registry" | "registry" | "service_discovery" => Self::ServiceRegistry,
            "key_value_store" | "kv_store" | "kvstore" => Self::KeyValueStore,
            "load_balancer" | "loadbalancer" | "lb" => Self::LoadBalancer,
            "service_mesh" | "mesh" => Self::ServiceMesh,
            "dns" | "dns_discovery" => Self::DnsDiscovery,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Get string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::ContainerOrchestration => "container_orchestration",
            Self::ServiceRegistry => "service_registry",
            Self::KeyValueStore => "key_value_store",
            Self::LoadBalancer => "load_balancer",
            Self::ServiceMesh => "service_mesh",
            Self::DnsDiscovery => "dns_discovery",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Vendor implementation details (optional metadata)
#[derive(Debug, Clone)]
pub struct VendorImplementation {
    /// Vendor name (kubernetes, consul, etcd, etc.)
    pub vendor_name: String,
    /// Vendor version (optional)
    pub version: Option<String>,
    /// Additional vendor-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Capability-based provider factory
///
/// This factory creates providers based on CAPABILITIES, not vendor names.
/// The actual vendor implementation is discovered or configured.
pub struct CapabilityProviderFactory {
    /// Registered capability mappings
    capability_mappings: HashMap<CapabilityType, Vec<ProviderFactoryImpl>>,
}

impl CapabilityProviderFactory {
    /// Create new capability provider factory
    #[must_use]
    pub fn new() -> Self {
        Self {
            capability_mappings: HashMap::new(),
        }
    }

    /// Register a provider factory for a capability
    pub fn register_capability(
        &mut self,
        capability: CapabilityType,
        factory: ProviderFactoryImpl,
    ) {
        self.capability_mappings.entry(capability).or_default().push(factory);
    }

    /// Create provider for capability (vendor-agnostic)
    pub async fn create_for_capability(
        &self,
        capability: CapabilityType,
        config: ProviderConfig,
    ) -> SongbirdResult<DiscoveryProviderImpl> {
        let factories = self.capability_mappings.get(&capability).ok_or_else(|| {
            SongbirdError::configuration(format!(
                "No providers registered for capability: {capability:?}"
            ))
        })?;

        let mut last_error = None;
        for factory in factories {
            match factory.create_provider(config.clone()).await {
                Ok(provider) => return Ok(provider),
                Err(e) => last_error = Some(e),
            }
        }

        Err(last_error
            .unwrap_or_else(|| SongbirdError::configuration("All provider factories failed")))
    }

    /// Get available capabilities
    #[must_use]
    pub fn available_capabilities(&self) -> Vec<CapabilityType> {
        self.capability_mappings.keys().cloned().collect()
    }
}

impl Default for CapabilityProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper: Create default capability factory with standard mappings
///
/// Maps capabilities to vendor implementations based on environment or discovery
#[must_use]
pub fn create_default_capability_factory() -> CapabilityProviderFactory {
    #[allow(
        unused_mut,
        reason = "mut needed only when kubernetes/consul features register factories"
    )]
    let mut factory = CapabilityProviderFactory::new();

    #[cfg(feature = "kubernetes")]
    {
        use crate::abstraction::adapters::kubernetes_adapter::KubernetesProviderFactory;
        factory.register_capability(
            CapabilityType::ContainerOrchestration,
            ProviderFactoryImpl::Kubernetes(KubernetesProviderFactory),
        );
    }

    #[cfg(feature = "consul")]
    {
        use crate::abstraction::adapters::consul_adapter::ConsulProviderFactory;
        factory.register_capability(
            CapabilityType::ServiceRegistry,
            ProviderFactoryImpl::Consul(ConsulProviderFactory),
        );
    }

    factory
}

/// Request a capability provider (vendor-agnostic entry point)
pub async fn request_capability_provider(
    capability: CapabilityType,
    config: ProviderConfig,
) -> SongbirdResult<DiscoveryProviderImpl> {
    let factory = create_default_capability_factory();
    factory.create_for_capability(capability, config).await
}

/// Discover which vendor provides a capability
pub async fn discover_capability_vendor(
    capability: CapabilityType,
) -> SongbirdResult<VendorImplementation> {
    let vendor_name = match capability {
        CapabilityType::ContainerOrchestration => {
            if songbird_process_env::var("KUBERNETES_SERVICE_HOST").is_ok() {
                "kubernetes".to_string()
            } else if songbird_process_env::var("DOCKER_HOST").is_ok() {
                "docker".to_string()
            } else {
                "kubernetes".to_string()
            }
        }
        CapabilityType::ServiceRegistry => {
            if songbird_process_env::var("CONSUL_HTTP_ADDR").is_ok() {
                "consul".to_string()
            } else if songbird_process_env::var("ETCD_ENDPOINTS").is_ok() {
                "etcd".to_string()
            } else {
                "consul".to_string()
            }
        }
        _ => "unknown".to_string(),
    };

    Ok(VendorImplementation {
        vendor_name,
        version: None,
        metadata: HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_type_from_str() {
        assert_eq!(
            CapabilityType::parse("container_orchestration"),
            CapabilityType::ContainerOrchestration
        );
        assert_eq!(CapabilityType::parse("service_registry"), CapabilityType::ServiceRegistry);
        assert_eq!(
            CapabilityType::parse("custom_capability"),
            CapabilityType::Custom("custom_capability".to_string())
        );
    }

    #[test]
    fn test_capability_type_as_str() {
        assert_eq!(CapabilityType::ContainerOrchestration.as_str(), "container_orchestration");
        assert_eq!(CapabilityType::ServiceRegistry.as_str(), "service_registry");
    }

    #[test]
    fn test_capability_factory_creation() {
        let factory = CapabilityProviderFactory::new();
        assert_eq!(factory.available_capabilities().len(), 0);
    }
}
