// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Discovery Abstraction Layer
//!
//! This module provides a capability-based, agnostic discovery system that eliminates
//! hard-coded dependencies on specific external services like Consul or Kubernetes.
//!
//! ## Architecture Principles
//!
//! 1. **Capability-Based**: Providers declare what they can do, not what they are
//! 2. **Protocol Agnostic**: No assumptions about HTTP, gRPC, or other protocols
//! 3. **Runtime Registration**: Providers can be registered at runtime
//! 4. **Delegation Pattern**: Route requests to capable providers
//! 5. **Zero Hard-coding**: No string matching or specific service assumptions

pub mod adapters;
pub mod capabilities;
pub mod capability_providers;
pub mod delegation;
pub mod modernized_factory;
pub mod providers;
pub mod registry;

// Re-export main abstractions
pub use adapters::{DiscoveryProviderImpl, ProviderFactory, ProviderFactoryImpl};
pub use capabilities::{CapabilityMatcher, DiscoveryCapability};
pub use capability_providers::{
    CapabilityProviderFactory, CapabilityType, VendorImplementation,
    create_default_capability_factory, discover_capability_vendor, request_capability_provider,
};
pub use delegation::{DelegationStrategy, DiscoveryDelegator};
pub use providers::{
    DiscoveryProvider, LoadBalancingHints, ProviderConfig, ProviderMetadata, ServiceMetrics,
};
pub use registry::{ProviderRegistry, RegistryError};

// Re-export adapters for migration
pub use adapters::{
    ConsulProviderAdapter, ConsulProviderFactory, KubernetesProviderAdapter,
    KubernetesProviderFactory, StaticProviderAdapter, StaticProviderFactory,
};

// Re-export modernized factory
pub use modernized_factory::{DiscoveryConfigBuilder, ModernizedDiscoveryFactory};
