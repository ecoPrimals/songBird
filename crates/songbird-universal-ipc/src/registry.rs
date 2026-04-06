// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Service registry for tracking registered primals

use crate::endpoint::{NativeEndpoint, VirtualEndpoint};
use crate::error::{IpcError, IpcResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Service entry in the registry
#[derive(Debug, Clone)]
pub struct ServiceEntry {
    /// Virtual endpoint (platform-agnostic)
    pub virtual_endpoint: VirtualEndpoint,
    /// Native endpoint (platform-specific)
    pub native_endpoint: NativeEndpoint,
    /// Capabilities this service provides
    pub capabilities: Vec<String>,
    /// When service was registered
    pub registered_at: Instant,
    /// Last seen timestamp (for health checking)
    pub last_seen: Instant,
}

/// Metadata for persistent storage (optional `storage provider` integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Service name
    pub name: String,
    /// Virtual endpoint path
    pub virtual_path: String,
    /// Native endpoint (for debugging)
    pub native_endpoint_display: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Platform
    pub platform: String,
    /// Registered timestamp (Unix epoch)
    pub registered_at_secs: u64,
}

impl ServiceEntry {
    /// Convert to metadata for persistent storage
    #[must_use]
    pub fn to_metadata(&self) -> ServiceMetadata {
        ServiceMetadata {
            name: self.virtual_endpoint.primal_name().unwrap_or("unknown").to_string(),
            virtual_path: self.virtual_endpoint.path.clone(),
            native_endpoint_display: self.native_endpoint.display(),
            capabilities: self.capabilities.clone(),
            platform: std::env::consts::OS.to_string(),
            registered_at_secs: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// In-memory service registry
///
/// Tracks all registered services and their endpoints.
/// Thread-safe via `RwLock` for concurrent access.
pub struct ServiceRegistry {
    /// Services by name
    services: RwLock<HashMap<String, ServiceEntry>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Register a service
    ///
    /// # Arguments
    /// * `name` - Service name (e.g., "beardog")
    /// * `native_endpoint` - Platform-specific endpoint
    /// * `capabilities` - List of capabilities this service provides
    ///
    /// # Returns
    /// Virtual endpoint for this service
    pub async fn register(
        &self,
        name: &str,
        native_endpoint: NativeEndpoint,
        capabilities: Vec<String>,
    ) -> IpcResult<VirtualEndpoint> {
        let mut services = self.services.write().await;

        // Check if already registered
        if services.contains_key(name) {
            return Err(IpcError::ServiceAlreadyRegistered(name.to_string()));
        }

        let virtual_endpoint = VirtualEndpoint::new(name);
        let entry = ServiceEntry {
            virtual_endpoint: virtual_endpoint.clone(),
            native_endpoint,
            capabilities: capabilities.clone(),
            registered_at: Instant::now(),
            last_seen: Instant::now(),
        };

        services.insert(name.to_string(), entry);
        drop(services);

        info!("Registered service '{}' with {} capabilities", name, capabilities.len());
        debug!("Service capabilities: {:?}", capabilities);

        Ok(virtual_endpoint)
    }

    /// Unregister a service
    ///
    /// # Arguments
    /// * `name` - Service name to unregister
    pub async fn unregister(&self, name: &str) -> IpcResult<()> {
        let mut services = self.services.write().await;

        if services.remove(name).is_some() {
            info!("Unregistered service '{}'", name);
            Ok(())
        } else {
            Err(IpcError::ServiceNotFound(name.to_string()))
        }
    }

    /// Resolve virtual path to native endpoint
    ///
    /// # Arguments
    /// * `virtual_path` - Virtual path (e.g., "/primal/beardog")
    ///
    /// # Returns
    /// Native endpoint for the service
    pub async fn resolve(&self, virtual_path: &str) -> IpcResult<NativeEndpoint> {
        let services = self.services.read().await;

        // Extract service name from virtual path
        let name = virtual_path
            .strip_prefix("/primal/")
            .ok_or_else(|| IpcError::InvalidVirtualPath(virtual_path.to_string()))?;

        services
            .get(name)
            .map(|entry| entry.native_endpoint.clone())
            .ok_or_else(|| IpcError::ServiceNotFound(name.to_string()))
    }

    /// Find services by capability
    ///
    /// # Arguments
    /// * `capability` - Capability to search for (e.g., "crypto", "storage")
    ///
    /// # Returns
    /// List of virtual paths for services with this capability
    pub async fn find_by_capability(&self, capability: &str) -> Vec<String> {
        let services = self.services.read().await;

        services
            .values()
            .filter(|entry| entry.capabilities.contains(&capability.to_string()))
            .map(|entry| entry.virtual_endpoint.path.clone())
            .collect()
    }

    /// List all registered services
    ///
    /// # Returns
    /// List of all service names
    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// Get service entry (for introspection)
    ///
    /// # Arguments
    /// * `name` - Service name
    ///
    /// # Returns
    /// Service entry if found
    pub async fn get_service(&self, name: &str) -> Option<ServiceEntry> {
        let services = self.services.read().await;
        services.get(name).cloned()
    }

    /// Update last seen timestamp (for health checking)
    ///
    /// # Arguments
    /// * `name` - Service name
    pub async fn update_last_seen(&self, name: &str) -> IpcResult<()> {
        let mut services = self.services.write().await;

        if let Some(entry) = services.get_mut(name) {
            entry.last_seen = Instant::now();
            Ok(())
        } else {
            Err(IpcError::ServiceNotFound(name.to_string()))
        }
    }

    /// Get all service metadata (for persistence)
    pub async fn get_all_metadata(&self) -> Vec<ServiceMetadata> {
        let services = self.services.read().await;
        services.values().map(ServiceEntry::to_metadata).collect()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::endpoint::NativeEndpoint;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_registry_register() {
        let registry = ServiceRegistry::new();

        #[cfg(unix)]
        let endpoint = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/test.sock"));

        #[cfg(not(unix))]
        let endpoint = NativeEndpoint::TcpLocal(8080);

        let virtual_endpoint =
            registry.register("test-primal", endpoint, vec!["test".to_string()]).await.unwrap();

        assert_eq!(virtual_endpoint.path, "/primal/test-primal");
    }

    #[tokio::test]
    async fn test_registry_resolve() {
        let registry = ServiceRegistry::new();

        #[cfg(unix)]
        let endpoint = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/test.sock"));

        #[cfg(not(unix))]
        let endpoint = NativeEndpoint::TcpLocal(8080);

        registry.register("test-primal", endpoint, vec![]).await.unwrap();

        let resolved = registry.resolve("/primal/test-primal").await.unwrap();

        // Verify we got a valid endpoint back
        assert!(!resolved.display().is_empty());

        // Verify it's one of the supported types
        match resolved {
            NativeEndpoint::UnixSocket(_)
            | NativeEndpoint::AbstractSocket(_)
            | NativeEndpoint::NamedPipe(_)
            | NativeEndpoint::XPC(_)
            | NativeEndpoint::InProcess(_)
            | NativeEndpoint::SharedMemory(_)
            | NativeEndpoint::TcpLocal(_) => {
                // Valid endpoint type
            }
        }
    }

    #[tokio::test]
    async fn test_registry_find_by_capability() {
        let registry = ServiceRegistry::new();

        #[cfg(unix)]
        let endpoint1 = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/primal1.sock"));
        #[cfg(unix)]
        let endpoint2 = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/primal2.sock"));

        #[cfg(not(unix))]
        let endpoint1 = NativeEndpoint::TcpLocal(8080);
        #[cfg(not(unix))]
        let endpoint2 = NativeEndpoint::TcpLocal(8081);

        registry.register("primal1", endpoint1, vec!["crypto".to_string()]).await.unwrap();

        registry
            .register("primal2", endpoint2, vec!["crypto".to_string(), "storage".to_string()])
            .await
            .unwrap();

        let crypto_services = registry.find_by_capability("crypto").await;
        assert_eq!(crypto_services.len(), 2);

        let storage_services = registry.find_by_capability("storage").await;
        assert_eq!(storage_services.len(), 1);
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let registry = ServiceRegistry::new();

        #[cfg(unix)]
        let endpoint = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/test.sock"));

        #[cfg(not(unix))]
        let endpoint = NativeEndpoint::TcpLocal(8080);

        registry.register("test-primal", endpoint, vec![]).await.unwrap();

        assert!(registry.get_service("test-primal").await.is_some());

        registry.unregister("test-primal").await.unwrap();

        assert!(registry.get_service("test-primal").await.is_none());
    }

    #[tokio::test]
    async fn test_register_duplicate_returns_error() {
        let registry = ServiceRegistry::new();
        #[cfg(unix)]
        let endpoint = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/dup.sock"));
        #[cfg(not(unix))]
        let endpoint = NativeEndpoint::TcpLocal(8080);

        registry.register("dup", endpoint.clone(), vec![]).await.expect("first register");
        let err = registry.register("dup", endpoint, vec![]).await.expect_err("duplicate");
        assert!(matches!(err, crate::error::IpcError::ServiceAlreadyRegistered(_)));
    }

    #[tokio::test]
    async fn test_resolve_invalid_virtual_path() {
        let registry = ServiceRegistry::new();
        let err = registry.resolve("/wrong/security-provider").await.expect_err("bad path");
        assert!(matches!(err, crate::error::IpcError::InvalidVirtualPath(_)));
    }

    #[tokio::test]
    async fn test_resolve_unknown_service() {
        let registry = ServiceRegistry::new();
        let err = registry.resolve("/primal/missing").await.expect_err("missing");
        assert!(matches!(err, crate::error::IpcError::ServiceNotFound(_)));
    }

    #[tokio::test]
    async fn test_unregister_missing_service() {
        let registry = ServiceRegistry::new();
        let err = registry.unregister("nope").await.expect_err("not found");
        assert!(matches!(err, crate::error::IpcError::ServiceNotFound(_)));
    }

    #[tokio::test]
    async fn test_update_last_seen_errors_when_missing() {
        let registry = ServiceRegistry::new();
        let err = registry.update_last_seen("ghost").await.expect_err("no service");
        assert!(matches!(err, crate::error::IpcError::ServiceNotFound(_)));
    }

    #[tokio::test]
    async fn test_list_and_metadata_track_services() {
        let registry = ServiceRegistry::new();
        #[cfg(unix)]
        let ep = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/meta.sock"));
        #[cfg(not(unix))]
        let ep = NativeEndpoint::TcpLocal(9090);

        registry.register("alpha", ep, vec!["a".into()]).await.expect("register");
        let names = registry.list_services().await;
        assert!(names.contains(&"alpha".to_string()));

        let meta = registry.get_all_metadata().await;
        assert_eq!(meta.len(), 1);
        assert_eq!(meta[0].name, "alpha");
        assert!(meta[0].virtual_path.contains("alpha"));
    }

    #[tokio::test]
    async fn test_service_entry_to_metadata_shape() {
        #[cfg(unix)]
        let ep = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/md.sock"));
        #[cfg(not(unix))]
        let ep = NativeEndpoint::TcpLocal(7070);

        let entry = ServiceEntry {
            virtual_endpoint: crate::endpoint::VirtualEndpoint::new("z"),
            native_endpoint: ep,
            capabilities: vec!["x".into()],
            registered_at: std::time::Instant::now(),
            last_seen: std::time::Instant::now(),
        };
        let m = entry.to_metadata();
        assert_eq!(m.name, "z");
        assert_eq!(m.virtual_path, "/primal/z");
        assert_eq!(m.capabilities, vec!["x".to_string()]);
    }

    #[tokio::test]
    async fn update_last_seen_updates_existing_service() {
        let registry = ServiceRegistry::new();
        #[cfg(unix)]
        let ep = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/seen.sock"));
        #[cfg(not(unix))]
        let ep = NativeEndpoint::TcpLocal(6060);

        registry.register("seen", ep, vec![]).await.expect("register");
        registry.update_last_seen("seen").await.expect("last_seen");
        let svc = registry.get_service("seen").await.expect("service");
        assert_eq!(svc.capabilities.len(), 0);
    }

    #[tokio::test]
    async fn find_by_capability_empty_when_no_match() {
        let registry = ServiceRegistry::new();
        #[cfg(unix)]
        let ep = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/nomatch.sock"));
        #[cfg(not(unix))]
        let ep = NativeEndpoint::TcpLocal(5050);

        registry.register("only", ep, vec!["a".into()]).await.expect("register");
        let paths = registry.find_by_capability("missing-cap").await;
        assert!(paths.is_empty());
    }

    #[test]
    fn service_metadata_serde_roundtrip() {
        let m = ServiceMetadata {
            name: "n".into(),
            virtual_path: "/primal/n".into(),
            native_endpoint_display: "unix:///x".into(),
            capabilities: vec!["c".into()],
            platform: "linux".into(),
            registered_at_secs: 42,
        };
        let v = serde_json::to_string(&m).expect("serialize metadata");
        let back: ServiceMetadata = serde_json::from_str(&v).expect("deserialize metadata");
        assert_eq!(back.name, "n");
        assert_eq!(back.registered_at_secs, 42);
    }
}
