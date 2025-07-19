//! Service discovery backend implementations

pub mod consul;
pub mod kubernetes;
pub mod static_discovery;

// Re-export backend implementations
pub use consul::ConsulServiceDiscovery;
pub use kubernetes::KubernetesServiceDiscovery;
pub use static_discovery::StaticServiceDiscovery;
