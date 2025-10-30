//! # Legacy Backend Adapters
//!
//! This module provides adapters that wrap the existing hardcoded Consul and Kubernetes
//! backends to work with the new agnostic provider system. This enables gradual migration
//! without breaking existing functionality.

pub mod consul_adapter;
pub mod kubernetes_adapter;
pub mod static_adapter;

#[cfg(test)]
#[path = "../adapters_tests.rs"]
mod adapters_tests;

// Re-export adapters
pub use consul_adapter::{ConsulProviderAdapter, ConsulProviderFactory};
pub use kubernetes_adapter::{KubernetesProviderAdapter, KubernetesProviderFactory};
pub use static_adapter::{StaticProviderAdapter, StaticProviderFactory};
