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
    /// Ed25519 signature over `signed_payload` (base64, via `BearDog` delegation).
    /// `None` when crypto provider is unavailable (standalone mode).
    pub signature: Option<String>,
    /// Canonical JSON payload that was signed.
    pub signed_payload: Option<String>,
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
    /// Ed25519 signature (base64) if signed by crypto provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
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
            signature: self.signature.clone(),
        }
    }
}

/// A registry change event (for `ipc.watch` consumers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEvent {
    /// Monotonic revision at which this event occurred
    pub revision: u64,
    /// Event kind
    pub kind: RegistryEventKind,
    /// Primal that triggered the event
    pub primal: String,
    /// Capabilities affected
    pub capabilities: Vec<String>,
    /// Native endpoint (for newly registered services)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

/// Kind of registry mutation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryEventKind {
    Registered,
    Unregistered,
}

/// In-memory service registry
///
/// Tracks all registered services and their endpoints.
/// Thread-safe via `RwLock` for concurrent access.
/// Maintains a monotonic revision + event log for `ipc.watch`.
pub struct ServiceRegistry {
    /// Services by name
    services: RwLock<HashMap<String, ServiceEntry>>,
    /// Monotonic revision counter (increments on every mutation)
    revision: std::sync::atomic::AtomicU64,
    /// Event log for `ipc.watch` (bounded ring buffer of recent changes)
    events: RwLock<Vec<RegistryEvent>>,
}

/// Maximum number of events retained in the event log
const EVENT_LOG_CAPACITY: usize = 256;

impl ServiceRegistry {
    /// Create a new service registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            revision: std::sync::atomic::AtomicU64::new(0),
            events: RwLock::new(Vec::with_capacity(EVENT_LOG_CAPACITY)),
        }
    }

    /// Current monotonic revision (increases on every register/unregister)
    pub fn revision(&self) -> u64 {
        self.revision.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Get events since a given revision (for `ipc.watch` consumers).
    ///
    /// Returns all events with `revision > since_revision`, optionally
    /// filtered to only those affecting `capabilities`.
    pub async fn events_since(
        &self,
        since_revision: u64,
        capability_filter: Option<&[String]>,
    ) -> (u64, Vec<RegistryEvent>) {
        let current = self.revision();
        let events = self.events.read().await;
        let filtered: Vec<RegistryEvent> = events
            .iter()
            .filter(|e| e.revision > since_revision)
            .filter(|e| {
                capability_filter
                    .map_or(true, |caps| e.capabilities.iter().any(|c| caps.contains(c)))
            })
            .cloned()
            .collect();
        (current, filtered)
    }

    /// Register a service
    ///
    /// # Arguments
    /// * `name` - Service name (e.g., "beardog")
    /// * `native_endpoint` - Platform-specific endpoint
    /// * `capabilities` - List of capabilities this service provides
    /// * `signature` - Ed25519 signature from crypto provider (`None` in standalone mode)
    /// * `signed_payload` - Canonical JSON that was signed (`None` in standalone mode)
    ///
    /// # Returns
    /// Virtual endpoint for this service
    pub async fn register(
        &self,
        name: &str,
        native_endpoint: NativeEndpoint,
        capabilities: Vec<String>,
        signature: Option<String>,
        signed_payload: Option<String>,
    ) -> IpcResult<VirtualEndpoint> {
        let mut services = self.services.write().await;

        // Check if already registered
        if services.contains_key(name) {
            return Err(IpcError::ServiceAlreadyRegistered(name.to_string()));
        }

        let virtual_endpoint = VirtualEndpoint::new(name);
        let endpoint_display = native_endpoint.display();
        let entry = ServiceEntry {
            virtual_endpoint: virtual_endpoint.clone(),
            native_endpoint,
            capabilities: capabilities.clone(),
            registered_at: Instant::now(),
            last_seen: Instant::now(),
            signature,
            signed_payload,
        };

        services.insert(name.to_string(), entry);
        drop(services);

        let rev = self.revision.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
        let event = RegistryEvent {
            revision: rev,
            kind: RegistryEventKind::Registered,
            primal: name.to_string(),
            capabilities: capabilities.clone(),
            endpoint: Some(endpoint_display),
        };
        {
            let mut log = self.events.write().await;
            let len = log.len();
            if len >= EVENT_LOG_CAPACITY {
                log.drain(..len / 2);
            }
            log.push(event);
        }

        info!(
            "Registered service '{}' with {} capabilities (rev={})",
            name,
            capabilities.len(),
            rev
        );
        debug!("Service capabilities: {:?}", capabilities);

        Ok(virtual_endpoint)
    }

    /// Unregister a service
    ///
    /// # Arguments
    /// * `name` - Service name to unregister
    pub async fn unregister(&self, name: &str) -> IpcResult<()> {
        let mut services = self.services.write().await;

        if let Some(removed) = services.remove(name) {
            drop(services);

            let rev = self.revision.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
            let event = RegistryEvent {
                revision: rev,
                kind: RegistryEventKind::Unregistered,
                primal: name.to_string(),
                capabilities: removed.capabilities,
                endpoint: None,
            };
            {
                let mut log = self.events.write().await;
                let len = log.len();
                if len >= EVENT_LOG_CAPACITY {
                    log.drain(..len / 2);
                }
                log.push(event);
            }

            info!("Unregistered service '{}' (rev={})", name, rev);
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

    /// Resolve a capability to the single best provider (most-recently-seen first).
    ///
    /// This is the IPC equivalent of DNS resolution: given a capability string,
    /// return the one endpoint a caller should connect to.
    pub async fn resolve_by_capability(&self, capability: &str) -> Option<(String, ServiceEntry)> {
        let services = self.services.read().await;
        services
            .iter()
            .filter(|(_, entry)| entry.capabilities.contains(&capability.to_string()))
            .max_by_key(|(_, entry)| entry.last_seen)
            .map(|(name, entry)| (name.clone(), entry.clone()))
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

        let virtual_endpoint = registry
            .register("test-primal", endpoint, vec!["test".to_string()], None, None)
            .await
            .unwrap();

        assert_eq!(virtual_endpoint.path, "/primal/test-primal");
    }

    #[tokio::test]
    async fn test_registry_resolve() {
        let registry = ServiceRegistry::new();

        #[cfg(unix)]
        let endpoint = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/test.sock"));

        #[cfg(not(unix))]
        let endpoint = NativeEndpoint::TcpLocal(8080);

        registry.register("test-primal", endpoint, vec![], None, None).await.unwrap();

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

        registry
            .register("primal1", endpoint1, vec!["crypto".to_string()], None, None)
            .await
            .unwrap();

        registry
            .register(
                "primal2",
                endpoint2,
                vec!["crypto".to_string(), "storage".to_string()],
                None,
                None,
            )
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

        registry.register("test-primal", endpoint, vec![], None, None).await.unwrap();

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

        registry
            .register("dup", endpoint.clone(), vec![], None, None)
            .await
            .expect("first register");
        let err =
            registry.register("dup", endpoint, vec![], None, None).await.expect_err("duplicate");
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

        registry.register("alpha", ep, vec!["a".into()], None, None).await.expect("register");
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
            signature: None,
            signed_payload: None,
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

        registry.register("seen", ep, vec![], None, None).await.expect("register");
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

        registry.register("only", ep, vec!["a".into()], None, None).await.expect("register");
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
            signature: None,
        };
        let v = serde_json::to_string(&m).expect("serialize metadata");
        let back: ServiceMetadata = serde_json::from_str(&v).expect("deserialize metadata");
        assert_eq!(back.name, "n");
        assert_eq!(back.registered_at_secs, 42);
    }

    #[tokio::test]
    async fn register_stores_and_returns_signature() {
        let registry = ServiceRegistry::new();
        #[cfg(unix)]
        let ep = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/signed.sock"));
        #[cfg(not(unix))]
        let ep = NativeEndpoint::TcpLocal(4040);

        let sig = Some("base64sig==".to_string());
        let payload =
            Some(r#"{"c":["crypto"],"e":"/tmp/signed.sock","p":"sec","t":"T0"}"#.to_string());

        registry
            .register("sec", ep, vec!["crypto".into()], sig.clone(), payload.clone())
            .await
            .expect("register");

        let entry = registry.get_service("sec").await.expect("service exists");
        assert_eq!(entry.signature, sig);
        assert_eq!(entry.signed_payload, payload);
    }

    #[tokio::test]
    async fn register_without_signature_stores_none() {
        let registry = ServiceRegistry::new();
        #[cfg(unix)]
        let ep = NativeEndpoint::UnixSocket(PathBuf::from("/tmp/unsigned.sock"));
        #[cfg(not(unix))]
        let ep = NativeEndpoint::TcpLocal(3030);

        registry
            .register("standalone", ep, vec!["net".into()], None, None)
            .await
            .expect("register");

        let entry = registry.get_service("standalone").await.expect("service exists");
        assert!(entry.signature.is_none());
        assert!(entry.signed_payload.is_none());
    }

    #[test]
    fn service_metadata_omits_none_signature() {
        let m = ServiceMetadata {
            name: "x".into(),
            virtual_path: "/primal/x".into(),
            native_endpoint_display: "unix:///x".into(),
            capabilities: vec![],
            platform: "linux".into(),
            registered_at_secs: 1,
            signature: None,
        };
        let json = serde_json::to_string(&m).expect("serialize");
        assert!(!json.contains("signature"), "None signature should be omitted");
    }
}
