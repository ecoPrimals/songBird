// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # Service Registry for Songbird Orchestrator
//!
//! **Universal Port Authority Implementation**
//!
//! This module implements the service registry that allows primals to register
//! dynamically with Songbird. It manages:
//!
//! - Service registration and deregistration
//! - Port allocation and management
//! - Heartbeat tracking and TTL cleanup
//! - Service discovery and querying
//!
//! ## Architecture
//!
//! The service registry is the core of the "Universal Port Authority" principle:
//! - Songbird assigns ALL ports
//! - Primals never bind ports themselves
//! - Zero configuration required
//! - Infinite scalability
//!
//! ## Usage
//!
//! ```no_run
//! use songbird_orchestrator::service_registry::ServiceRegistry;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let registry = ServiceRegistry::new();
//!
//! // Primals register via HTTP API (handled by router)
//! // Registry manages everything automatically
//! # Ok(())
//! # }
//! ```

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use songbird_types::defaults::ports::{DEFAULT_PORT_RANGE_END, DEFAULT_PORT_RANGE_START};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

// ============================================================================
// CORE TYPES
// ============================================================================

/// Service registry manages registered primals and port allocation
#[derive(Clone)]
pub struct ServiceRegistry {
    /// Registered services
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,

    /// Port allocation state
    port_allocator: Arc<RwLock<PortAllocator>>,

    /// Registry configuration
    config: RegistryConfig,
}

/// Configuration for service registry
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Starting port for dynamic allocation
    pub port_range_start: u16,

    /// Ending port for dynamic allocation
    pub port_range_end: u16,

    /// Default heartbeat interval (seconds)
    pub default_heartbeat_interval: u64,

    /// Service TTL after missed heartbeats (seconds)
    pub service_ttl_sec: u64,

    /// Maximum missed heartbeats before cleanup
    pub max_missed_heartbeats: u32,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            port_range_start: DEFAULT_PORT_RANGE_START,
            port_range_end: DEFAULT_PORT_RANGE_END,
            default_heartbeat_interval: 30,
            service_ttl_sec: 300, // 5 minutes
            max_missed_heartbeats: 5,
        }
    }
}

/// Provide a default `tokio::time::Instant` for serde deserialization skip.
fn default_instant() -> tokio::time::Instant {
    tokio::time::Instant::now()
}

/// A registered service
///
/// **VIRTUAL-TIME** (Apr 2026): `last_heartbeat_instant` uses `tokio::time::Instant`
/// for elapsed-time checks in cleanup, enabling deterministic testing with
/// `start_paused = true`. The `SystemTime` fields are retained for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    /// Unique service ID
    pub service_id: String,

    /// Service name (e.g., "Toadstool", "security provider")
    pub service_name: String,

    /// Service version
    pub service_version: String,

    /// Assigned endpoint
    pub assigned_endpoint: ServiceEndpoint,

    /// Optional fallback endpoint
    pub fallback_endpoint: Option<ServiceEndpoint>,

    /// Registration token
    pub token: String,

    /// Capabilities
    pub capabilities: Vec<ServiceCapability>,

    /// Protocols
    pub protocols: Vec<String>,

    /// Registration timestamp
    pub registered_at: SystemTime,

    /// Last heartbeat timestamp (wall-clock, for serialization / display)
    pub last_heartbeat: SystemTime,

    /// Last heartbeat (monotonic, for elapsed-time cleanup checks).
    /// Advances with `tokio::time::advance()` under `start_paused = true`.
    #[serde(skip, default = "default_instant")]
    pub last_heartbeat_instant: tokio::time::Instant,

    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,

    /// Current status
    pub status: ServiceStatus,

    /// Trust level
    pub trust_level: String,

    /// Missed heartbeat count
    pub missed_heartbeats: u32,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Service endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub full_url: String,
}

impl ServiceEndpoint {
    #[must_use]
    pub fn new(protocol: &str, host: &str, port: u16) -> Self {
        Self {
            protocol: protocol.to_string(),
            host: host.to_string(),
            port,
            full_url: format!("{protocol}://{host}:{port}"),
        }
    }
}

/// Service capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability {
    pub name: String,
    #[serde(rename = "type")]
    pub capability_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Active,
    Inactive,
    Degraded,
}

/// Port allocator
struct PortAllocator {
    /// Allocated ports (port -> `service_id`)
    allocated: HashMap<u16, String>,

    /// Next port to try
    next_port: u16,

    /// Port range
    range_start: u16,
    range_end: u16,
}

impl PortAllocator {
    fn new(range_start: u16, range_end: u16) -> Self {
        Self {
            allocated: HashMap::new(),
            next_port: range_start,
            range_start,
            range_end,
        }
    }

    /// Allocate a port for a service
    fn allocate(&mut self, service_id: &str) -> Result<u16> {
        let start_port = self.next_port;

        loop {
            let port = self.next_port;

            // Move to next port for next allocation
            self.next_port += 1;
            if self.next_port > self.range_end {
                self.next_port = self.range_start;
            }

            // Check if this port is available
            if let std::collections::hash_map::Entry::Vacant(e) = self.allocated.entry(port) {
                e.insert(service_id.to_string());
                debug!("✅ Allocated port {} to service {}", port, service_id);
                return Ok(port);
            }

            // If we've wrapped around, no ports available
            if self.next_port == start_port {
                return Err(anyhow!(
                    "No available ports in range {}-{}",
                    self.range_start,
                    self.range_end
                ));
            }
        }
    }

    /// Release a port
    fn release(&mut self, port: u16) {
        if let Some(service_id) = self.allocated.remove(&port) {
            debug!("Released port {} from service {}", port, service_id);
        }
    }

    /// Check if a port is allocated
    fn is_allocated(&self, port: u16) -> bool {
        self.allocated.contains_key(&port)
    }
}

// ============================================================================
// REGISTRATION REQUEST/RESPONSE
// ============================================================================

/// Registration request from a primal
#[derive(Debug, Deserialize)]
pub struct RegistrationRequest {
    pub primal_name: String,
    pub primal_version: String,
    pub capabilities: Vec<ServiceCapability>,
    pub protocols: Vec<String>,
    pub preferred_protocol: String,
    pub health_check_path: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Registration response
#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    pub status: String,
    pub service_id: String,
    pub assigned_endpoint: ServiceEndpoint,
    pub fallback_endpoint: Option<ServiceEndpoint>,
    pub registration_token: String,
    pub heartbeat_interval_sec: u64,
    pub trust_level: String,
}

/// Heartbeat request
#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub service_id: String,
    pub token: String,
    pub status: String,
    pub current_load: Option<serde_json::Value>,
    pub capabilities_changed: bool,
}

/// Heartbeat response
#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub status: String,
    pub next_heartbeat_sec: u64,
    pub commands: Vec<String>,
}

/// Deregistration request
#[derive(Debug, Deserialize)]
pub struct DeregistrationRequest {
    pub service_id: String,
    pub token: String,
    pub reason: String,
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

impl ServiceRegistry {
    /// Create a new service registry with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(RegistryConfig::default())
    }

    /// Create a new service registry with custom configuration
    #[must_use]
    pub fn with_config(config: RegistryConfig) -> Self {
        let port_allocator = PortAllocator::new(config.port_range_start, config.port_range_end);

        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            port_allocator: Arc::new(RwLock::new(port_allocator)),
            config,
        }
    }

    /// Register a new service
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn register(&self, request: RegistrationRequest) -> Result<RegistrationResponse> {
        info!("📝 Registering service: {}", request.primal_name);

        // Generate service ID
        let service_id = Uuid::new_v4().to_string();

        // Generate registration token
        let token = Uuid::new_v4().to_string();

        // Allocate port
        let port = {
            let mut allocator = self.port_allocator.write().await;
            allocator.allocate(&service_id)?
        };

        // Create endpoints
        let assigned_endpoint = ServiceEndpoint::new(&request.preferred_protocol, "0.0.0.0", port);

        // Optional: Allocate fallback port for different protocol
        let fallback_endpoint = if request.protocols.len() > 1 {
            let fallback_protocol = request
                .protocols
                .iter()
                .find(|p| *p != &request.preferred_protocol)
                .map_or("https", std::string::String::as_str);

            let fallback_port = {
                let mut allocator = self.port_allocator.write().await;
                allocator.allocate(&service_id).ok()
            };

            fallback_port.map(|p| ServiceEndpoint::new(fallback_protocol, "0.0.0.0", p))
        } else {
            None
        };

        let now = SystemTime::now();
        let service = RegisteredService {
            service_id: service_id.clone(),
            service_name: request.primal_name.clone(),
            service_version: request.primal_version,
            assigned_endpoint: assigned_endpoint.clone(),
            fallback_endpoint: fallback_endpoint.clone(),
            token: token.clone(),
            capabilities: request.capabilities,
            protocols: request.protocols,
            registered_at: now,
            last_heartbeat: now,
            last_heartbeat_instant: tokio::time::Instant::now(),
            heartbeat_interval: self.config.default_heartbeat_interval,
            status: ServiceStatus::Active,
            trust_level: "anonymous".to_string(),
            missed_heartbeats: 0,
            metadata: request.metadata.unwrap_or_default(),
        };

        // Store service
        {
            let mut services = self.services.write().await;
            services.insert(service_id.clone(), service);
        }

        info!("✅ Registered {} as service {} on port {}", request.primal_name, service_id, port);

        Ok(RegistrationResponse {
            status: "registered".to_string(),
            service_id,
            assigned_endpoint,
            fallback_endpoint,
            registration_token: token,
            heartbeat_interval_sec: self.config.default_heartbeat_interval,
            trust_level: "anonymous".to_string(),
        })
    }

    /// Process a heartbeat
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn heartbeat(&self, request: HeartbeatRequest) -> Result<HeartbeatResponse> {
        debug!("💓 Heartbeat from service {}", request.service_id);

        let mut services = self.services.write().await;

        let service = services
            .get_mut(&request.service_id)
            .ok_or_else(|| anyhow!("Service not found: {}", request.service_id))?;

        // Validate token
        if service.token != request.token {
            return Err(anyhow!("Invalid token for service {}", request.service_id));
        }

        service.last_heartbeat = SystemTime::now();
        service.last_heartbeat_instant = tokio::time::Instant::now();
        service.missed_heartbeats = 0;

        // Update status if changed
        if request.status == "operational" {
            service.status = ServiceStatus::Active;
        }

        debug!("✅ Heartbeat acknowledged for {}", service.service_name);

        Ok(HeartbeatResponse {
            status: "acknowledged".to_string(),
            next_heartbeat_sec: service.heartbeat_interval,
            commands: vec![],
        })
    }

    /// Deregister a service
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn deregister(&self, request: DeregistrationRequest) -> Result<()> {
        info!("🛑 Deregistering service {}", request.service_id);

        let service = {
            let mut services = self.services.write().await;
            services.remove(&request.service_id)
        };

        if let Some(service) = service {
            // Validate token
            if service.token != request.token {
                return Err(anyhow!("Invalid token for service {}", request.service_id));
            }

            // Release ports
            {
                let mut allocator = self.port_allocator.write().await;
                allocator.release(service.assigned_endpoint.port);
                if let Some(fallback) = service.fallback_endpoint {
                    allocator.release(fallback.port);
                }
            }

            info!("✅ Deregistered service {}", service.service_name);
            Ok(())
        } else {
            Err(anyhow!("Service not found: {}", request.service_id))
        }
    }

    /// Get a service by ID
    pub async fn get_service(&self, service_id: &str) -> Option<RegisteredService> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    /// List all services
    pub async fn list_services(&self) -> Vec<RegisteredService> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// Query services by capability
    pub async fn query_by_capability(&self, capability: &str) -> Vec<RegisteredService> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|s| s.capabilities.iter().any(|c| c.name == capability))
            .cloned()
            .collect()
    }

    /// Cleanup stale services (TTL expired)
    ///
    /// Uses `tokio::time::Instant` for elapsed-time checks, compatible with
    /// `start_paused = true` virtual time in tests.
    pub async fn cleanup_stale_services(&self) -> usize {
        let ttl = Duration::from_secs(self.config.service_ttl_sec);

        let mut services = self.services.write().await;
        let mut to_remove = Vec::new();

        for (id, service) in services.iter_mut() {
            let elapsed = service.last_heartbeat_instant.elapsed();

            if elapsed > ttl {
                warn!(
                    "Service {} ({}) has not sent heartbeat for {:?}, marking for removal",
                    service.service_name, id, elapsed
                );
                to_remove.push(id.clone());
            } else {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "heartbeat count won't exceed u32 in practice"
                )]
                let expected_heartbeats =
                    (elapsed.as_secs() / service.heartbeat_interval) as u32;
                if expected_heartbeats > 0 {
                    service.missed_heartbeats = expected_heartbeats.saturating_sub(1);

                    if service.missed_heartbeats >= self.config.max_missed_heartbeats {
                        service.status = ServiceStatus::Degraded;
                    }
                }
            }
        }

        // Remove stale services
        let count = to_remove.len();
        for id in to_remove {
            if let Some(service) = services.remove(&id) {
                // Release ports
                let mut allocator = self.port_allocator.write().await;
                allocator.release(service.assigned_endpoint.port);
                if let Some(fallback) = service.fallback_endpoint {
                    allocator.release(fallback.port);
                }
            }
        }

        if count > 0 {
            info!("🗑️  Cleaned up {} stale service(s)", count);
        }

        count
    }

    /// Get registry statistics
    pub async fn get_stats(&self) -> RegistryStats {
        let services = self.services.read().await;

        let total = services.len();
        let active = services.values().filter(|s| s.status == ServiceStatus::Active).count();
        let degraded = services.values().filter(|s| s.status == ServiceStatus::Degraded).count();
        let inactive = services.values().filter(|s| s.status == ServiceStatus::Inactive).count();

        let allocator = self.port_allocator.read().await;
        let allocated_ports = allocator.allocated.len();

        RegistryStats {
            total_services: total,
            active_services: active,
            degraded_services: degraded,
            inactive_services: inactive,
            allocated_ports,
            available_ports: (self.config.port_range_end - self.config.port_range_start) as usize
                - allocated_ports,
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Serialize)]
pub struct RegistryStats {
    pub total_services: usize,
    pub active_services: usize,
    pub degraded_services: usize,
    pub inactive_services: usize,
    pub allocated_ports: usize,
    pub available_ports: usize,
}

// ============================================================================
// BACKGROUND CLEANUP TASK
// ============================================================================

/// Spawn a background task to periodically cleanup stale services
#[must_use]
pub fn spawn_cleanup_task(
    registry: ServiceRegistry,
    interval_sec: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_sec));

        loop {
            interval.tick().await;

            match registry.cleanup_stale_services().await {
                0 => debug!("Cleanup: No stale services found"),
                count => info!("Cleanup: Removed {} stale service(s)", count),
            }
        }
    })
}

#[cfg(test)]
#[path = "service_registry_tests.rs"]
mod tests;
