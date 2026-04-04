// SPDX-License-Identifier: AGPL-3.0-only
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
use tracing::{error, info, warn};

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
    async fn stage_2_start_servers(&self) -> Result<u16> {
        // ✅ DEPLOYMENT FIX (Dec 20, 2025): Start HTTP server FIRST to get actual port
        // This ensures discovery broadcasts the correct port even if fallback occurs
        // ✅ DISCOVERY FIX (Jan 28, 2026): Call actual HTTP server module (not stub)
        // The stub start_http_server() returns 0, which breaks discovery beacons
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
        )
        .await?;
        info!("✅ HTTP server started on port {}", actual_https_port);

        // 🎧 NEW (Jan 4, 2026): Start IPC Server for inter-primal communication
        // Unix: Unix domain sockets, Windows: TCP fallback
        info!("🎧 Starting IPC server...");
        self.orchestrator.start_ipc_server().await?;
        info!("✅ IPC server started");

        // 🌍 NEW (Jan 19, 2026): Start Universal IPC Broker for service-based inter-primal IPC
        // ✅ EVOLUTION (Jan 29, 2026): Wire up discovery listener for runtime peer discovery
        info!("🌍 Starting Universal IPC Broker...");
        match crate::ipc::universal_broker::start_broker_with_discovery(
            self.orchestrator.discovery_listener.clone(),
        )
        .await
        {
            Ok(()) => {
                info!("✅ Universal IPC Broker started");
                if self.orchestrator.discovery_listener.is_some() {
                    info!("   🌉 Discovery bridge: ENABLED (real-time peer discovery)");
                }
            }
            Err(e) => {
                warn!("⚠️  Universal IPC Broker failed to start: {}", e);
                warn!("   Continuing without Universal IPC Broker");
                warn!("   Core functionality (Tower Atomic, HTTP, Unix sockets) still available");
            }
        }

        // 🚀 NEW (Jan 6, 2026): Start tarpc Server for high-performance primal-to-primal RPC
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
                    "https://{}:{}",
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

        Ok(())
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
        // ✅ POST-STARTUP: Verify external connectivity (Dec 20, 2025)
        // This helps catch network/firewall issues early
        self.orchestrator.verify_external_connectivity().await?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::app::core::SongbirdOrchestrator;
    use songbird_types::config::CanonicalSongbirdConfig;

    #[test]
    fn startup_pipeline_stage_order_is_sequential_and_includes_two_b() {
        assert_eq!(STARTUP_PIPELINE_STAGE_ORDER.len(), 8);
        assert_eq!(
            STARTUP_PIPELINE_STAGE_ORDER,
            &[
                "stage_1_provision_security",
                "stage_2_start_servers",
                "stage_2b_igd_auto_configure",
                "stage_3_register_self",
                "stage_4_start_discovery",
                "stage_5_start_federation",
                "stage_6_background_tasks",
                "stage_7_verify_connectivity",
            ]
        );
        let pos_2 = STARTUP_PIPELINE_STAGE_ORDER
            .iter()
            .position(|s| *s == "stage_2_start_servers")
            .unwrap();
        let pos_2b = STARTUP_PIPELINE_STAGE_ORDER
            .iter()
            .position(|s| *s == "stage_2b_igd_auto_configure")
            .unwrap();
        let pos_3 = STARTUP_PIPELINE_STAGE_ORDER
            .iter()
            .position(|s| *s == "stage_3_register_self")
            .unwrap();
        assert!(pos_2 < pos_2b && pos_2b < pos_3);
    }

    #[test]
    fn stage_3_federation_capabilities_match_expected_set() {
        assert_eq!(
            STAGE_3_FEDERATION_SELF_CAPABILITIES,
            &["orchestrator", "secure_http", "http.request", "tls.1.3"]
        );
    }

    #[test]
    fn stage_4_discovery_capabilities_match_expected_set() {
        assert_eq!(
            STAGE_4_DISCOVERY_CAPABILITIES,
            &[
                "orchestration",
                "federation",
                "secure_http",
                "http.request",
                "http.get",
                "http.post",
                "tls.1.3",
            ]
        );
    }

    #[test]
    fn trust_cleanup_interval_is_five_minutes() {
        assert_eq!(TRUST_CLEANUP_INTERVAL_SECS, 300);
    }

    #[test]
    fn http_bind_socket_addr_accepts_loopback_defaults() {
        let addr = http_bind_socket_addr("127.0.0.1", 8080).unwrap();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn http_bind_socket_addr_rejects_invalid_socket_form() {
        let err = http_bind_socket_addr("not-a-valid-socket-addr", 1).unwrap_err();
        assert!(err.to_string().contains("Invalid bind address"), "unexpected message: {err}");
    }

    #[test]
    fn igd_enabled_from_env_value_parses_opt_in_cases() {
        assert!(igd_enabled_from_env_value("1"));
        assert!(igd_enabled_from_env_value("true"));
        assert!(igd_enabled_from_env_value("TRUE"));
        assert!(!igd_enabled_from_env_value("0"));
        assert!(!igd_enabled_from_env_value("false"));
        assert!(!igd_enabled_from_env_value(""));
    }

    #[test]
    fn default_config_network_matches_stage_2_bind_inputs() {
        let cfg = CanonicalSongbirdConfig::default();
        assert_eq!(cfg.network.bind_host, "127.0.0.1");
        assert_eq!(cfg.network.base_port, 8080);
        let addr = http_bind_socket_addr(&cfg.network.bind_host, cfg.network.base_port).unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:8080");
    }

    /// When Stage 2 cannot build a bind address, startup must not proceed (surfaced as `Err`).
    #[test]
    fn http_bind_failure_propagates_as_error() {
        let res = http_bind_socket_addr("%%%invalid%%%", 80);
        assert!(res.is_err());
    }

    /// Compile-time check: `start` returns a `Send` future (required for Tokio multi-thread).
    #[allow(dead_code, reason = "compile-time Send bound assertion — never called at runtime")]
    fn _assert_start_returns_send_future(
        orch: &mut SongbirdOrchestrator,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        StartupOrchestrator::new(orch).start()
    }

    /// Compile-time check: `StartupOrchestrator::new` remains usable from `&mut SongbirdOrchestrator`.
    #[allow(dead_code, reason = "compile-time API usability assertion — never called at runtime")]
    fn _assert_new_accepts_mutable_orchestrator(orch: &mut SongbirdOrchestrator) {
        let _ = StartupOrchestrator::new(orch);
    }
}
