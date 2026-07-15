// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Startup Orchestration Module
//!
//! **Purpose**: Cleanly orchestrate the 7-stage startup sequence of Songbird
//!
//! **Deep Debt Evolution** (Feb 6, 2026):
//! - Extracted from 275-line `start()` method in core.rs
//! - Follows Single Responsibility Principle
//! - Each stage is focused and testable
//! - Maintains sequential dependencies
//!
//! **Startup Stages**:
//! 1. Provision Security - JWT secrets, identity query
//! 2. Start Core Servers - HTTP, IPC, tarpc
//! 3. Register Self - Federation self-registration
//! 4. Start Discovery - Anonymous peer discovery
//! 5. Start Federation - Coordinator and trust cleanup
//! 6. Start Background Tasks - Health monitoring, cleanup
//! 7. Verify Connectivity - Post-startup verification
//!
//! **Benefits**:
//! - Clear startup flow (7 stages vs 275-line monolith)
//! - Each stage is 20-40 lines (focused!)
//! - Easy to test each stage independently
//! - Maintains sequential dependencies
//! - core.rs reduced by ~250 lines

use anyhow::Result;
use songbird_discovery::anonymous::TransportEndpointMessage;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::core::SongbirdOrchestrator;
use super::network::detect_primary_ip;
use crate::node_identity::NodeIdentity;

/// Order of stages executed by [`StartupOrchestrator::start`].
///
/// Stage 2b runs after Stage 2 once the HTTPS port is known; it is best-effort and never fails
/// startup. Keep this list aligned with the implementation of `start`.
pub const STARTUP_PIPELINE_STAGE_ORDER: &[&str] = &[
    "stage_1_provision_security",
    "stage_2_start_servers",
    "stage_2b_igd_auto_configure",
    "stage_2c_socket_auto_discovery",
    "stage_3_register_self",
    "stage_4_start_discovery",
    "stage_5_start_federation",
    "stage_6_background_tasks",
    "stage_7_verify_connectivity",
];

/// Capabilities advertised when re-registering this node in federation (Stage 3).
pub(super) const STAGE_3_FEDERATION_SELF_CAPABILITIES: &[&str] =
    &["orchestrator", "secure_http", "http.request", "tls.1.3"];

/// Capabilities included in anonymous discovery beacons (Stage 4).
pub(super) const STAGE_4_DISCOVERY_CAPABILITIES: &[&str] = &[
    "orchestration",
    "federation",
    "secure_http",
    "http.request",
    "http.get",
    "http.post",
    "tls.1.3",
];

/// Interval for trust escalation cleanup spawned in Stage 5.
pub(super) const TRUST_CLEANUP_INTERVAL_SECS: u64 = 300;

/// Interval for periodic socket auto-discovery re-scan (Stage 6).
///
/// Primals may start after Songbird — a periodic re-scan ensures the
/// `ipc.resolve` registry is self-healing without launcher assistance.
#[cfg(any(unix, test))]
pub(super) const SOCKET_RESCAN_INTERVAL_SECS: u64 = 30;

/// Build the TCP bind address used for the HTTP server in Stage 2.
///
/// # Errors
///
/// Returns an error if `host:port` is not a valid [`SocketAddr`] string (e.g. invalid IP).
pub(super) fn http_bind_socket_addr(host: &str, port: u16) -> Result<SocketAddr> {
    format!("{host}:{port}").parse().map_err(|e| anyhow::anyhow!("Invalid bind address: {e}"))
}

/// Whether `SONGBIRD_IGD_ENABLED` enables IGD auto-configure (Stage 2b).
#[must_use]
pub(super) fn igd_enabled_from_env_value(value: &str) -> bool {
    value == "1" || value.eq_ignore_ascii_case("true")
}

/// Startup orchestrator for clean, focused startup sequence
pub struct StartupOrchestrator<'a> {
    orchestrator: &'a mut SongbirdOrchestrator,
}

impl<'a> StartupOrchestrator<'a> {
    /// Create new startup orchestrator
    pub const fn new(orchestrator: &'a mut SongbirdOrchestrator) -> Self {
        Self {
            orchestrator,
        }
    }

    /// Execute complete startup sequence (7 stages)
    ///
    /// This is the main entry point for startup orchestration.
    /// It executes all 7 stages sequentially, respecting dependencies.
    ///
    /// **Stages**:
    /// 1. Provision Security (JWT + identity)
    /// 2. Start Core Servers (HTTP, IPC, tarpc)
    /// 3. Register Self (federation)
    /// 4. Start Discovery (anonymous peer discovery)
    /// 5. Start Federation (coordinator, trust cleanup)
    /// 6. Start Background Tasks (health, cleanup)
    /// 7. Verify Connectivity (post-startup)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start(mut self) -> Result<()> {
        crate::env_config::validate_btsp_insecure_guard()?;

        info!("🚀 Starting Songbird Orchestrator");
        info!("   Mode: Production-ready with secure defaults");
        info!("   Auto-discovery: Secure anonymous capability exchange");
        info!("   Federation: Zero-trust progressive escalation");
        info!("   All connections: Encrypted by default (TLS failsafe)");

        // Stage 1: Provision security credentials
        self.stage_1_provision_security().await?;

        // Stage 2: Start core servers (returns actual HTTP port)
        let actual_https_port = self.stage_2_start_servers().await?;

        // Stage 2b: IGD auto-configure (optional, non-blocking)
        // Attempts to forward the Songbird port on the router via UPnP/NAT-PMP
        self.stage_2b_igd_auto_configure().await;

        // Stage 2c: Socket auto-discovery (LD-08) — scan biomeos dirs and seed registry
        self.stage_2c_socket_auto_discovery().await;

        // Stage 3: Register self in federation (needs actual port)
        self.stage_3_register_self(actual_https_port).await?;

        // Stage 4: Start discovery system (needs actual port)
        self.stage_4_start_discovery(actual_https_port).await?;

        // Stage 5: Start federation coordinator and trust cleanup
        self.stage_5_start_federation().await?;

        // Stage 6: Start background tasks (health, cleanup)
        self.stage_6_background_tasks().await?;

        // Stage 7: Verify external connectivity
        self.stage_7_verify_connectivity().await?;

        info!("✅ Songbird Orchestrator started successfully");
        Ok(())
    }

    /// Stage 1: Provision security credentials (JWT secret + identity query)
    ///
    /// **Purpose**: Acquire security credentials from `security provider` before starting servers
    ///
    /// **Actions**:
    /// - Provision JWT secret from `security provider` (via capability discovery)
    /// - Query security identity (USB seed integration)
    /// - Start observability manager
    ///
    /// **Why First**: Servers need JWT secrets for authentication
    async fn stage_1_provision_security(&self) -> Result<()> {
        // NEW (Jan 17, 2026): Provision JWT secret from security provider via capability discovery
        info!("🔐 Provisioning JWT secret from security provider...");
        let jwt_secret = self.orchestrator.provision_jwt_secret().await?;
        info!("✅ JWT secret provisioned ({} bytes, Pure Rust delegation!)", jwt_secret.len());
        // ✅ JWT secret is now provided to HTTP handlers via capability discovery
        // HTTP authentication implemented via security provider delegation (Jan 17, 2026)

        // NEW: Query security provider for our encryption tag (USB seed integration)
        self.orchestrator.query_security_identity().await?;

        // Start observability manager
        self.orchestrator.observability_manager.start().await?;

        Ok(())
    }

    /// Stage 2: Start core servers (HTTP, IPC, Universal IPC, tarpc)
    ///
    /// **Purpose**: Start all network servers before discovery/federation
    ///
    /// **Actions**:
    /// - Start HTTP server (returns actual bound port)
    /// - Start IPC server (Unix sockets or Windows TCP fallback)
    /// - Start Universal IPC Broker (service-based inter-primal IPC)
    /// - Start tarpc server (high-performance primal-to-primal RPC)
    ///
    /// **Why Second**: Discovery needs actual HTTP port to advertise
    ///
    /// **Returns**: Actual HTTPS port (may differ from configured if fallback occurs)
    async fn stage_2_start_servers(&mut self) -> Result<u16> {
        // ✅ Wave 75: Create shared IpcServiceHandler BEFORE starting any server.
        // Both HTTP and UDS broker share this instance for state unification.
        let shared_registry = Arc::new(tokio::sync::RwLock::new(
            songbird_universal_ipc::registry::ServiceRegistry::new(),
        ));
        let shared_ipc_handler =
            Arc::new(songbird_universal_ipc::service::IpcServiceHandler::with_federation_state(
                Arc::clone(&shared_registry),
                Arc::clone(&self.orchestrator.federation_state),
            ));
        info!("✅ Shared IPC handler created (HTTP/UDS unified state)");

        // Log proxy routes registered from SONGBIRD_PROXY_ROUTES env
        let proxy_caps = shared_ipc_handler.capability_router().list_capabilities();
        if !proxy_caps.is_empty() {
            info!("🔀 http.proxy routes registered: {:?}", proxy_caps);
        }

        // Start HTTP server with shared handler (needs to start first for port binding)
        info!("🌐 Starting HTTP server...");
        let bind_address = http_bind_socket_addr(
            &self.orchestrator._config.network.bind_host,
            self.orchestrator._config.network.base_port,
        )?;

        let actual_https_port = crate::app::http_server::start_http_server(
            Arc::clone(&self.orchestrator.federation_state),
            Arc::clone(&self.orchestrator.federated_service_registry),
            Arc::clone(&self.orchestrator.service_registry),
            bind_address,
            Some(Arc::clone(&shared_ipc_handler)),
        )
        .await?;
        info!("✅ HTTP server started on port {}", actual_https_port);

        // 🎧 Start IPC Server for inter-primal communication
        info!("🎧 Starting IPC server...");
        self.orchestrator.start_ipc_server().await?;
        info!("✅ IPC server started");

        // 🌍 Start Universal IPC Broker with the SAME shared handler (state unification)
        info!("🌍 Starting Universal IPC Broker...");
        match crate::ipc::universal_broker::start_broker_with_shared_handler(
            Arc::clone(&shared_ipc_handler),
            Arc::clone(&shared_registry),
        )
        .await
        {
            Ok(handle) => {
                info!("✅ Universal IPC Broker started (unified with HTTP)");
                self.orchestrator.broker_registry = Some(handle.registry);
                self.orchestrator.broker_mesh_handler = Some(Arc::clone(&handle.mesh_handler));

                // Auto-seed mesh from SONGBIRD_PEERS (Wave 73: no manual mesh.init required)
                crate::mesh_seed::spawn_mesh_seed(handle.mesh_handler);
            }
            Err(e) => {
                warn!("⚠️  Universal IPC Broker failed to start: {}", e);
                warn!("   Continuing without Universal IPC Broker");
                warn!("   Core functionality (Tower Atomic, HTTP, Unix sockets) still available");
            }
        }

        // 🚀 Start tarpc Server for high-performance primal-to-primal RPC
        info!("🚀 Starting tarpc server...");
        self.orchestrator.start_tarpc_server().await?;
        info!("✅ tarpc server started");

        Ok(actual_https_port)
    }

    /// Stage 3: Register self in federation (with actual port and endpoints)
    ///
    /// **Purpose**: Register this node in federation state with actual bound port
    ///
    /// **Actions**:
    /// - Load node identity (stable ID)
    /// - Detect all endpoints with actual port
    /// - Create self-registration with capabilities
    ///
    /// # Stage 2b: IGD Auto-Configure (optional, non-blocking)
    ///
    /// **Purpose**: Attempt to forward the Songbird port on the router
    ///
    /// **Actions**:
    /// - Check if `SONGBIRD_IGD_ENABLED` is set (opt-in)
    /// - Discover router via `UPnP` IGD or NAT-PMP
    /// - Request port forwarding for the Songbird port
    /// - Log result (success or manual instructions)
    ///
    /// **Why after Stage 2**: Port is now bound, forwarding makes it reachable
    /// **Non-blocking**: Failure here does NOT prevent startup
    async fn stage_2b_igd_auto_configure(&self) {
        // Opt-in via environment variable (default: disabled)
        let enabled = songbird_process_env::var("SONGBIRD_IGD_ENABLED")
            .map(|v| igd_enabled_from_env_value(&v))
            .unwrap_or(false);

        if !enabled {
            info!("IGD auto-configure: Disabled (set SONGBIRD_IGD_ENABLED=true to enable)");
            return;
        }

        info!("IGD auto-configure: Discovering router...");

        let (gateway, diagnostics) = songbird_igd::Gateway::discover_with_diagnostics().await;

        if !gateway.is_available() {
            warn!(
                "IGD auto-configure: No UPnP/NAT-PMP support detected on gateway {}",
                diagnostics.gateway_ip
            );
            if !diagnostics.manual_instructions.is_empty() {
                info!("Manual port forwarding instructions:");
                for step in &diagnostics.manual_instructions {
                    info!("  {}", step);
                }
            }
            if !diagnostics.alternative_tiers.is_empty() {
                info!("Alternative connectivity:");
                for tier in &diagnostics.alternative_tiers {
                    info!("  {}", tier);
                }
            }
            return;
        }

        let port = self.orchestrator._config.network.base_port;
        info!("IGD auto-configure: Mapping port {} on gateway {}...", port, gateway.ip);

        match gateway.map_port(port, port, "TCP", 86400).await {
            Ok(mapping) => {
                info!("IGD auto-configure: Port {} forwarded successfully", port);
                if let Some(ext_ip) = mapping.external_ip {
                    info!("  External endpoint: {}:{}", ext_ip, mapping.external_port);
                }
                info!("  TTL: {} seconds (auto-renew recommended)", mapping.lease_duration);
            }
            Err(e) => {
                warn!("IGD auto-configure: Port mapping failed: {}", e);
                warn!("  Songbird will continue with other connectivity tiers");
            }
        }
    }

    /// Stage 2c: Socket auto-discovery (LD-08)
    ///
    /// Scans biomeos socket directories for `*.sock` files, probes each with
    /// `identity.get` + `capabilities.list` (Wire Standard L3), and registers
    /// discovered primals into the broker's service registry. This ensures
    /// `ipc.resolve` and `capability.resolve` have data to resolve against
    /// without requiring every primal to call `ipc.register` at startup.
    ///
    /// **Why after Stage 2**: Broker socket must be bound first, and we need
    /// the registry handle from the broker.
    ///
    /// **Non-blocking**: Failure does not prevent startup.
    #[cfg(unix)]
    async fn stage_2c_socket_auto_discovery(&self) {
        let Some(ref registry) = self.orchestrator.broker_registry else {
            info!("Socket auto-discovery: skipped (no broker registry available)");
            return;
        };

        info!("🔍 Starting socket auto-discovery (LD-08)...");
        let reg = registry.read().await;
        let count =
            crate::primal_discovery::socket_auto_discovery::discover_and_register_biomeos_primals(
                &reg,
            )
            .await;
        drop(reg);

        if count > 0 {
            info!("✅ Socket auto-discovery: {} primal(s) seeded into ipc.resolve registry", count);
        } else {
            info!(
                "Socket auto-discovery: no peer sockets found (primals will register via ipc.register at runtime)"
            );
        }
    }

    #[cfg(not(unix))]
    async fn stage_2c_socket_auto_discovery(&self) {
        info!("Socket auto-discovery: skipped (Unix sockets not available on this platform)");
    }

    /// Stage 3: Register self in federation
    ///
    /// - Register in federation state
    ///
    /// **Why Third**: Needs actual HTTP port from Stage 2
    async fn stage_3_register_self(&self, actual_https_port: u16) -> Result<()> {
        // ✅ IDENTITY FIX (Dec 20, 2025): Re-register SELF with actual port and endpoints
        // This updates the self-registration created during new() with the actual bound port
        if self.orchestrator.federation_config.is_some() {
            // Re-load node identity (same stable ID) and detect endpoints with actual port
            let mut node_identity = NodeIdentity::new_or_load(None)?;
            node_identity.detect_all_endpoints(actual_https_port)?;

            info!("🆔 Re-registering self with actual port {}:", actual_https_port);
            info!("   ID: {}", node_identity.node_id);
            info!("   Name: {}", node_identity.node_name);
            info!("   Endpoints: {}", node_identity.endpoints.len());

            let updated_self_registration = songbird_network_federation::state::NodeRegistration {
                node_id: node_identity.node_id.to_string(),
                node_name: node_identity.node_name.clone(),
                node_address: format!(
                    "http://{}:{}",
                    detect_primary_ip()
                        .unwrap_or_else(|| songbird_types::constants::LOCALHOST.to_string()),
                    actual_https_port
                ),
                endpoints: Some(
                    node_identity
                        .endpoints
                        .iter()
                        .map(|ep| songbird_network_federation::state::TransportEndpointInfo {
                            interface_type: ep.interface_type.clone(),
                            address: format!("{}:{}", ep.address.ip(), actual_https_port),
                            protocols: ep.protocols.clone(),
                            preference: ep.preference,
                            status: songbird_network_federation::state::EndpointStatus::Active,
                            last_check: chrono::Utc::now(),
                        })
                        .collect(),
                ),
                capabilities: STAGE_3_FEDERATION_SELF_CAPABILITIES
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
                cpu_cores: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
                memory_gb: songbird_types::sys_metrics::total_memory_gb().max(16),
                gpu_model: SongbirdOrchestrator::detect_gpu(),
                storage_gb: SongbirdOrchestrator::detect_storage_capacity(),
                status: songbird_network_federation::state::NodeStatus::Active,
                joined_at: chrono::Utc::now(),
                last_heartbeat: chrono::Utc::now(),
            };

            info!("📝 Updating self-registration in federation");
            self.orchestrator.federation_state.register_node(updated_self_registration).await;
        }

        Ok(())
    }

    /// Stage 4: Start anonymous discovery system (with actual port)
    ///
    /// **Purpose**: Start anonymous peer discovery to find family members
    ///
    /// **Actions**:
    /// - Load node identity
    /// - Detect endpoints with actual port
    /// - Build capability list
    /// - Start discovery broadcaster (UDP multicast + subnet broadcast)
    /// - Start discovery → federation bridge
    ///
    /// **Why Fourth**: Needs actual HTTP port to advertise to peers
    async fn stage_4_start_discovery(&mut self, actual_https_port: u16) -> Result<()> {
        // Start anonymous discovery (if enabled) with ACTUAL port
        if self.orchestrator._config.discovery.mode.is_enabled() {
            info!(
                "🌐 Starting anonymous discovery with actual HTTPS port {}...",
                actual_https_port
            );

            // Re-use the SAME node identity (already loaded in Stage 3)
            let mut node_identity = NodeIdentity::new_or_load(None)?;
            node_identity.detect_all_endpoints(actual_https_port)?;

            // Start discovery broadcaster (v3.0 with multi-endpoint)
            let capabilities =
                STAGE_4_DISCOVERY_CAPABILITIES.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();

            // Convert endpoints to discovery message format
            // CRITICAL FIX (Dec 20, 2025): Include full address (IP:port) instead of just port
            // This allows receivers to properly coalesce multi-interface nodes under one identity
            let endpoint_messages: Vec<TransportEndpointMessage> = node_identity
                .endpoints
                .iter()
                .map(|ep| TransportEndpointMessage {
                    interface_type: ep.interface_type.clone(),
                    address: ep.address.to_string(), // ✅ Full address, not just port!
                    protocols: ep.protocols.clone(),
                    preference: ep.preference,
                })
                .collect();

            // ✅ DISCOVERY FIX (Jan 28, 2026): Capability-based broadcast addresses
            // Supports environment-based configuration for cross-interface discovery
            // Automatically adds subnet broadcast fallback to handle eth ↔ wifi boundaries
            let broadcast_addrs = SongbirdOrchestrator::discover_broadcast_addresses(
                &self.orchestrator._config.discovery.broadcast_addresses,
            );

            // ✅ SMART REFACTORING (v3.10.3 - Jan 6, 2026): Discovery system startup
            // Extracted to discovery_startup module for clarity, testability, and maintainability.
            // This reduces core.rs by ~168 lines while improving separation of concerns.
            // Demonstrates zero hardcoding, "build then Arc", and modern async patterns.
            let listener_arc = super::discovery_startup::start_discovery_system(
                self.orchestrator._config.discovery.port,
                actual_https_port,
                &node_identity,
                endpoint_messages,
                capabilities,
                broadcast_addrs,
                self.orchestrator.discovery_listener_pending.take(),
                Arc::clone(&self.orchestrator.discovery_status_manager),
            )
            .await?;

            // Store the configured listener for bridge polling
            self.orchestrator.discovery_listener = listener_arc;

            info!(
                "✅ Anonymous discovery started (UDP port {}, advertising HTTPS port {})",
                self.orchestrator._config.discovery.port, actual_https_port
            );

            // Start discovery → federation bridge
            self.orchestrator.start_discovery_federation_bridge().await?;
        }

        Ok(())
    }

    /// Stage 5: Start federation coordinator and trust cleanup
    ///
    /// **Purpose**: Start federation coordination and trust management
    ///
    /// **Actions**:
    /// - Start trust escalation cleanup task (every 5 minutes)
    /// - Start federation coordinator (if enabled)
    ///
    /// Security credentials and capability discovery run in Stage 1; construction-time
    /// [`super::security_setup::SecurityIntegration`] resolution happens in [`SongbirdOrchestrator::new`].
    ///
    /// **Why Fifth**: Needs discovery system running to coordinate federation
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn stage_5_start_federation(&self) -> Result<()> {
        // Start trust escalation cleanup task
        let trust_manager_clone = Arc::clone(&self.orchestrator.trust_manager);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                TRUST_CLEANUP_INTERVAL_SECS,
            ));
            loop {
                interval.tick().await;
                let removed = trust_manager_clone.cleanup_expired().await;
                if removed > 0 {
                    info!("🧹 Trust cleanup: removed {} expired relationships", removed);
                }
            }
        });
        info!("✅ Trust escalation cleanup task started");

        // Start federation coordinator (if enabled)
        if let (Some(coordinator), Some(config)) =
            (&self.orchestrator.federation_coordinator, &self.orchestrator.federation_config)
        {
            info!("🌐 Starting federation coordinator...");
            let coordinator_clone = Arc::clone(coordinator);
            let config_clone = config.clone();
            tokio::spawn(async move {
                if let Err(e) = coordinator_clone.coordinate(&config_clone).await {
                    error!("❌ Federation coordination error: {}", e);
                } else {
                    info!("✅ Federation coordinator started successfully");
                }
            });
        }

        Ok(())
    }

    /// Stage 6: Start background tasks (health monitoring, cleanup tasks)
    ///
    /// **Purpose**: Start all background maintenance tasks
    ///
    /// **Actions**:
    /// - Start health monitoring (heartbeats, health checks)
    /// - Start session TTL cleanup (removes stale sessions)
    /// - Start service registry cleanup (removes stale services)
    ///
    /// **Why Sixth**: All core systems running, now start maintenance
    async fn stage_6_background_tasks(&self) -> Result<()> {
        // Start health monitoring
        self.orchestrator.start_health_monitoring().await?;

        // Start session TTL cleanup task (Deep Debt Fix - Dec 20, 2025)
        self.orchestrator.start_session_ttl_cleanup().await?;

        // Start service registry cleanup task (Universal Port Authority - Dec 20, 2025)
        self.orchestrator.start_service_registry_cleanup();

        // Start periodic socket re-scan (self-healing auto-discovery)
        self.start_periodic_socket_rescan();

        Ok(())
    }

    /// Spawn a background task that periodically re-scans biomeos socket dirs
    /// and registers any newly-appeared primals. This makes the `ipc.resolve`
    /// registry self-healing: primals that start after Songbird are picked up
    /// within `SOCKET_RESCAN_INTERVAL_SECS` without launcher assistance.
    #[cfg(unix)]
    fn start_periodic_socket_rescan(&self) {
        let Some(ref registry) = self.orchestrator.broker_registry else {
            debug!("Periodic socket re-scan: skipped (no broker registry)");
            return;
        };

        let registry = std::sync::Arc::clone(registry);
        let interval = std::time::Duration::from_secs(SOCKET_RESCAN_INTERVAL_SECS);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let reg = registry.read().await;
                let count =
                    crate::primal_discovery::socket_auto_discovery::discover_and_register_biomeos_primals(&reg).await;
                if count > 0 {
                    info!("🔄 Periodic re-scan: registered {count} new primal(s)");
                }
            }
        });

        info!("🔄 Periodic socket re-scan: enabled (every {}s)", SOCKET_RESCAN_INTERVAL_SECS);
    }

    #[cfg(not(unix))]
    fn start_periodic_socket_rescan(&self) {
        debug!("Periodic socket re-scan: Unix sockets not supported on this platform");
    }

    /// Stage 7: Verify external connectivity (post-startup diagnostics)
    ///
    /// **Purpose**: Verify that HTTP server is reachable from external IPs
    ///
    /// **Actions**:
    /// - Test external connectivity
    /// - Provide diagnostics if issues detected
    /// - Attempt auto-remediation (firewall rules)
    ///
    /// **Why Last**: All systems running, now verify they're reachable
    async fn stage_7_verify_connectivity(&self) -> Result<()> {
        // Connectivity verification is best-effort — never fatal to startup.
        // Missing crypto provider, network issues, etc. should warn and continue.
        match self.orchestrator.verify_external_connectivity().await {
            Ok(()) => {}
            Err(e) => {
                warn!("⚠️  External connectivity verification failed (non-fatal): {e:#}");
                warn!("   Songbird will continue in cleartext/degraded mode");
            }
        }

        Ok(())
    }
}
