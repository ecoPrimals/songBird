//! # Capability-Based Provider System (Vendor Agnostic)
//!
//! This module provides vendor-agnostic access to discovery capabilities.
//! Instead of requesting "kubernetes" or "consul", you request capabilities
//! like "container_orchestration" or "service_registry".
//!
//! # Philosophy
//!
//! Code should not know or care about specific vendors. It should request
//! capabilities and let the discovery system find appropriate providers.
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for zero-cost abstraction

#![allow(async_fn_in_trait)]
use std::collections::HashMap;

use crate::abstraction::providers::{DiscoveryProvider, ProviderConfig, ProviderFactory};
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
    /// Get capability type from string
    pub fn from_str(s: &str) -> Self {
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
    capability_mappings: HashMap<CapabilityType, Vec<Box<dyn ProviderFactory>>>,
}

impl CapabilityProviderFactory {
    /// Create new capability provider factory
    pub fn new() -> Self {
        Self {
            capability_mappings: HashMap::new(),
        }
    }

    /// Register a provider factory for a capability
    pub fn register_capability(
        &mut self,
        capability: CapabilityType,
        factory: Box<dyn ProviderFactory>,
    ) {
        self.capability_mappings
            .entry(capability)
            .or_insert_with(Vec::new)
            .push(factory);
    }

    /// Create provider for capability (vendor-agnostic)
    pub async fn create_for_capability(
        &self,
        capability: CapabilityType,
        config: ProviderConfig,
    ) -> SongbirdResult<Box<dyn DiscoveryProvider>> {
        let factories = self.capability_mappings.get(&capability).ok_or_else(|| {
            SongbirdError::configuration_error(&format!(
                "No providers registered for capability: {:?}",
                capability
            ))
        })?;

        // Try each factory until one succeeds
        let mut last_error = None;
        for factory in factories {
            match factory.create_provider(config.clone()).await {
                Ok(provider) => return Ok(provider),
                Err(e) => last_error = Some(e),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            SongbirdError::configuration_error("All provider factories failed")
        }))
    }

    /// Get available capabilities
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
pub fn create_default_capability_factory() -> CapabilityProviderFactory {
    let mut factory = CapabilityProviderFactory::new();

    // Container orchestration capability
    // Maps to: kubernetes (primary), docker-swarm, nomad, etc.
    #[cfg(feature = "kubernetes")]
    {
        use crate::abstraction::adapters::kubernetes_adapter::KubernetesProviderFactory;
        factory.register_capability(
            CapabilityType::ContainerOrchestration,
            Box::new(KubernetesProviderFactory),
        );
    }

    // Service registry capability
    // Maps to: consul (primary), etcd, zookeeper, etc.
    #[cfg(feature = "consul")]
    {
        use crate::abstraction::adapters::consul_adapter::ConsulProviderFactory;
        factory.register_capability(
            CapabilityType::ServiceRegistry,
            Box::new(ConsulProviderFactory),
        );
    }

    // More capability mappings can be added here as features
    
    factory
}

/// Request a capability provider (vendor-agnostic entry point)
///
/// This is the main entry point for capability-based discovery.
/// You request a capability type, and the system finds an appropriate provider.
///
/// # Example
///
/// ```rust,ignore
/// // Request container orchestration (don't care if it's k8s, docker, etc.)
/// let orchestration = request_capability_provider(
///     CapabilityType::ContainerOrchestration,
///     config
/// ).await?;
///
/// // Request service registry (don't care if it's consul, etcd, etc.)
/// let registry = request_capability_provider(
///     CapabilityType::ServiceRegistry,
///     config
/// ).await?;
/// ```
pub async fn request_capability_provider(
    capability: CapabilityType,
    config: ProviderConfig,
) -> SongbirdResult<Box<dyn DiscoveryProvider>> {
    let factory = create_default_capability_factory();
    factory.create_for_capability(capability, config).await
}

/// Discover which vendor provides a capability
///
/// This function discovers which vendor implementation is actually providing
/// a capability, without the caller needing to know in advance.
pub async fn discover_capability_vendor(
    capability: CapabilityType,
) -> SongbirdResult<VendorImplementation> {
    // Check environment variables for hints
    let vendor_name = match capability {
        CapabilityType::ContainerOrchestration => {
            // Check for kubernetes
            if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
                "kubernetes".to_string()
            }
            // Check for docker
            else if std::env::var("DOCKER_HOST").is_ok() {
                "docker".to_string()
            }
            // Default to kubernetes
            else {
                "kubernetes".to_string()
            }
        }
        CapabilityType::ServiceRegistry => {
            // Check for consul
            if std::env::var("CONSUL_HTTP_ADDR").is_ok() {
                "consul".to_string()
            }
            // Check for etcd
            else if std::env::var("ETCD_ENDPOINTS").is_ok() {
                "etcd".to_string()
            }
            // Default to consul
            else {
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
            CapabilityType::from_str("container_orchestration"),
            CapabilityType::ContainerOrchestration
        );
        assert_eq!(
            CapabilityType::from_str("service_registry"),
            CapabilityType::ServiceRegistry
        );
        assert_eq!(
            CapabilityType::from_str("custom_capability"),
            CapabilityType::Custom("custom_capability".to_string())
        );
    }

    #[test]
    fn test_capability_type_as_str() {
        assert_eq!(
            CapabilityType::ContainerOrchestration.as_str(),
            "container_orchestration"
        );
        assert_eq!(
            CapabilityType::ServiceRegistry.as_str(),
            "service_registry"
        );
    }

    #[test]
    fn test_capability_factory_creation() {
        let factory = CapabilityProviderFactory::new();
        assert_eq!(factory.available_capabilities().len(), 0);
    }
}

