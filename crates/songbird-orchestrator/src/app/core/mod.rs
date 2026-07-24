// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core orchestrator types and state management
//!
//! This module contains the main `SongbirdOrchestrator` struct and its
//! fundamental operations.

mod commands;
mod lifecycle;
mod peers;

use anyhow::Result;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_network_federation::{FederationConfig, FederationCoordinator};
use songbird_observability::ObservabilityManager;
use songbird_types::config::CanonicalSongbirdConfig;
use std::sync::Arc;
#[cfg(unix)]
use tracing::error;
use tracing::{debug, info, warn};

use super::connection_manager::ConnectionManager;

use crate::trust::TrustEscalationManager;

/// Main orchestrator application
///
/// Coordinates all Songbird subsystems including federation, discovery,
/// observability, and service registry.
#[expect(dead_code, reason = "reserved for future use: phased subsystem wiring")]
pub struct SongbirdOrchestrator {
    pub(super) _config: CanonicalSongbirdConfig,
    pub(super) _service_registry: Arc<FederatedServiceRegistry>,
    pub(super) service_registry: Arc<crate::service_registry::ServiceRegistry>,
    // gaming_manager: Arc<GamingManager>, // Temporarily disabled
    // federation_manager: Arc<CanonicalFederation>, // Temporarily disabled
    pub(super) federation_coordinator: Option<Arc<FederationCoordinator>>,
    pub(super) federation_config: Option<FederationConfig>,
    pub(super) federation_state: Arc<FederationState>,
    pub(super) federated_service_registry: Arc<FederatedServiceRegistry>,
    pub(super) observability_manager: Arc<ObservabilityManager>,
    // security_integration: Arc<UniversalSecurityIntegration>, // Temporarily disabled
    pub(super) trust_manager: Arc<TrustEscalationManager>,
    pub(super) connection_manager: Arc<ConnectionManager>,

    // ✅ MODERN RUST PATTERN (v3.10.3 - Jan 6, 2026): Store listener WITHOUT Arc
    // This enables builder pattern usage (add BirdSong, stats) before Arc wrapping.
    // Arc wrapping happens in start() AFTER all configuration is complete.
    // This follows "build then Arc" pattern, not "Arc then try to mutate" anti-pattern.
    pub(super) discovery_listener_pending: Option<AnonymousDiscoveryListener>,
    pub(super) discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,

    pub(super) discovery_status_manager: Arc<songbird_discovery::DiscoveryStatusManager>,
    pub(super) ipc_server_handle: Option<tokio::task::JoinHandle<()>>,
    pub(super) broker_registry: Option<crate::ipc::universal_broker::SharedServiceRegistry>,
    pub(super) broker_mesh_handler: Option<Arc<songbird_universal_ipc::handlers::MeshHandler>>,
    pub(super) shutdown_signal: tokio::sync::broadcast::Receiver<()>,
    pub(super) shutdown_sender: tokio::sync::broadcast::Sender<()>,
}

// Implementation of SongbirdOrchestrator
// Moved from mod.rs in Phase 2 refactoring (Dec 25, 2025)
impl SongbirdOrchestrator {
    /// Create new orchestrator instance
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn new(config: CanonicalSongbirdConfig) -> Result<Self> {
        let (shutdown_sender, shutdown_signal) = tokio::sync::broadcast::channel(1);

        // Print secure-by-default configuration
        info!("🔒 Songbird Orchestrator - Secure by Default");

        // ✅ MODERN RUST PATTERN (v3.10.3 - Jan 6, 2026): Smart refactoring
        // Component initialization extracted to separate module for clarity and maintainability.
        // This follows single responsibility principle - initialization.rs handles component creation,
        // core.rs handles orchestration logic.
        info!(
            "   TLS: {} ({})",
            match config.security.tls.cert_policy {
                songbird_types::config::consolidated_canonical::security::TlsCertPolicy::ProvidedOnly => "✅ Enabled (provided certs)",
                songbird_types::config::consolidated_canonical::security::TlsCertPolicy::AutoGenerate => "✅ Enabled (auto-generate)",
                songbird_types::config::consolidated_canonical::security::TlsCertPolicy::AutoGenerateWithSans => "✅ Enabled (auto + SANs)",
            },
            match config.security.security_level {
                songbird_types::config::consolidated_canonical::security::SecurityLevel::Minimal => "minimal",
                songbird_types::config::consolidated_canonical::security::SecurityLevel::Standard => "standard",
                songbird_types::config::consolidated_canonical::security::SecurityLevel::Paranoid => "paranoid (2FA)",
            }
        );
        info!(
            "   Discovery: {} ({})",
            if config.discovery.mode.is_enabled() {
                "✅ Enabled"
            } else {
                "❌ Disabled"
            },
            match config.discovery.mode {
                songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::Disabled => "disabled",
                songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::Anonymous => "anonymous secure",
                songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::CapabilityAware => "capability-aware",
                songbird_types::config::consolidated_canonical::discovery::DiscoveryMode::FullDisclosure => "full disclosure",
            }
        );
        info!(
            "   Federation: {} (trust: {})",
            if config.federation.cluster_name.is_some() {
                "✅ Enabled"
            } else {
                "❌ Disabled"
            },
            match config.federation.trust_escalation_policy {
                songbird_types::config::consolidated_canonical::federation::TrustEscalationPolicy::Disabled => "static",
                songbird_types::config::consolidated_canonical::federation::TrustEscalationPolicy::CapabilityOnly => "capability escalation",
                songbird_types::config::consolidated_canonical::federation::TrustEscalationPolicy::Progressive => "progressive escalation",
            }
        );
        info!("   Trust Model: Zero-trust with progressive escalation");
        info!("   Initial Trust: {} → Escalate on demand", config.federation.initial_trust_level);
        info!("   🌐 Songbird handles complexity, security automatic!");

        // ✅ SMART REFACTORING (v3.10.3 - Jan 6, 2026): Initialize all components
        // Extracted to initialization module for clarity, maintainability, and testability.
        // This reduces core.rs by ~220 lines while improving separation of concerns.
        let components = super::initialization::initialize_components(&config)?;

        // Destructure initialized components for use
        let service_registry = components.service_registry;
        let universal_service_registry = components.universal_service_registry;
        let observability_manager = components.observability_manager;
        let trust_manager = components.trust_manager;
        let connection_manager = components.connection_manager;
        let federation_state = components.federation_state;
        let federated_service_registry = components.federated_service_registry;
        let node_identity = components.node_identity;
        let discovery_listener_pending = components.discovery_listener_pending;

        // ✅ SMART REFACTORING (v3.10.3 - Jan 6, 2026): Setup federation coordinator
        // Extracted to federation_setup module for clarity and testability.
        // This reduces core.rs by ~65 lines while improving separation of concerns.
        //
        // ✅ MODERN ASYNC PATTERN (v5.22.0 - Jan 25, 2026): Dependency injection
        // Production reads from environment via FederationOptions::from_env()
        // Tests pass explicit config - zero global state coupling!
        let federation_setup = super::federation_setup::setup_federation(
            &node_identity,
            Arc::clone(&federation_state),
            super::federation_setup::FederationOptions::from_env(),
        )
        .await?;
        let federation_coordinator = federation_setup.coordinator;
        let federation_config = federation_setup.config;

        // Security provider setup: capability-based runtime discovery.
        // Non-fatal — songbird degrades to unsigned/non-TLS operation when no
        // security provider is available (SB-STARTUP-01 fix).
        let _security_integration = match super::security_setup::setup_security().await {
            Ok(integration) => Some(integration),
            Err(e) => {
                tracing::warn!(
                    "⚠️  Security provider not available — operating in degraded mode: {e}"
                );
                tracing::warn!(
                    "   Set SONGBIRD_SECURITY_PROVIDER or SECURITY_ENDPOINT to enable TLS/signing"
                );
                None
            }
        };

        // Initialize discovery status manager for observability (Jan 5, 2026)
        let discovery_status_manager = Arc::new(songbird_discovery::DiscoveryStatusManager::new(
            config.discovery.mode.is_enabled(),
            format!("{:?}", config.discovery.mode),
            config.discovery.port,
            Some(songbird_types::defaults::network::ecosystem_discovery_multicast_addr()),
        ));

        Ok(Self {
            _config: config,
            _service_registry: service_registry,
            service_registry: universal_service_registry,
            // gaming_manager, // Temporarily disabled
            // federation_manager, // Temporarily disabled
            federation_coordinator,
            federation_config,
            federation_state,
            federated_service_registry,
            observability_manager,
            // security_integration, // Temporarily disabled
            trust_manager,
            connection_manager,
            discovery_listener_pending, // ✅ Holds non-Arc listener for configuration
            discovery_listener: None,   // ✅ Will be set in start() after full config
            discovery_status_manager,
            ipc_server_handle: None,   // Will be set in start()
            broker_registry: None,     // Set in stage_2 when broker starts
            broker_mesh_handler: None, // Set in stage_2 when broker starts
            shutdown_signal,
            shutdown_sender,
        })
    }

    /// Get federation state reference
    #[must_use]
    pub const fn federation_state(&self) -> &Arc<FederationState> {
        &self.federation_state
    }

    /// Get federated service registry reference
    #[must_use]
    pub const fn federated_service_registry(&self) -> &Arc<FederatedServiceRegistry> {
        &self.federated_service_registry
    }

    /// Get configuration reference
    #[must_use]
    pub const fn config(&self) -> &CanonicalSongbirdConfig {
        &self._config
    }

    /// Get service registry reference
    #[must_use]
    pub const fn service_registry(&self) -> &Arc<FederatedServiceRegistry> {
        &self._service_registry
    }

    // Temporarily disabled security integration methods

    /// Query security provider for node identity and encryption tags
    ///
    /// **EVOLVED (v3.15.0)**: Uses capability discovery (zero vendor hardcoding!)
    ///
    /// This is called on startup to get our encryption tag for USB seed integration.
    /// Discovers security provider via generic capability discovery.
    pub(crate) async fn query_security_identity(&self) -> Result<()> {
        use crate::security_capability_client::SecurityCapabilityClient;

        // EVOLVED: Use capability discovery (not hardcoded vendor name!)
        let security_url = crate::app::security_setup::discover_security_endpoint(None).await;

        if let Ok(url) = security_url {
            info!("🔐 Security provider configured: {}", url);

            // Query for identity
            let security_client = SecurityCapabilityClient::from_endpoint(url).await;

            match security_client?.get_identity().await {
                Ok(identity) => {
                    info!("✅ Got encryption tag: {}", identity.encryption_tag);
                    if let Some(family_id) = &identity.family_id {
                        info!("👨‍👩‍👧‍👦 Family ID: {}", family_id);
                    }
                    info!("🔑 Capabilities: {:?}", identity.capabilities);

                    // ✅ v3.14.0: Tags now broadcast in discovery via discover_identity_tags()
                    // For now, it's logged and can be accessed via SecurityCapabilityClient
                }
                Err(e) => {
                    warn!("⚠️  Could not query security identity: {}", e);
                    warn!("   Continuing without encryption tags");
                }
            }
        } else {
            debug!(
                "No security provider configured (capability-based discovery did not find security provider)"
            );
            debug!("Continuing without encryption tags");
        }

        Ok(())
    }

    /// Provision JWT secret from `security provider` via capability-based discovery
    ///
    /// ## TRUE PRIMAL Architecture
    ///
    /// - **Self-Knowledge**: Songbird only knows itself
    /// - **Capability Discovery**: Discovers `security provider` via "security" capability
    /// - **Graceful Fallback**: Uses secure random if `security provider` unavailable
    /// - **Pure Rust**: JSON-RPC over Unix socket (no C dependencies!)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - JWT secret (512 bits, base64-encoded)
    /// * `Err` - Only on critical failure (fallback always succeeds)
    pub(crate) async fn provision_jwt_secret(&self) -> Result<String> {
        use crate::auth::{get_security_socket_for_jwt, provision_jwt_secret};

        let security_socket = get_security_socket_for_jwt();

        let jwt_secret =
            provision_jwt_secret(security_socket.as_deref(), "songbird_authentication").await?;

        Ok(jwt_secret)
    }

    /// Discover broadcast addresses with capability-based fallback (NEW - Jan 28, 2026)
    ///
    /// **Zero Hardcoding Philosophy**: Discovers broadcast addresses at runtime from:
    /// 1. Environment variable: `SONGBIRD_BROADCAST_ADDRESSES` (comma-separated)
    /// 2. Configuration file: `discovery.broadcast_addresses`
    /// 3. Automatic fallback: Subnet broadcast for cross-interface discovery
    ///
    /// **Cross-Interface Discovery**: Automatically adds subnet broadcast addresses
    /// to handle eth ↔ wifi boundaries that multicast can't cross on consumer routers.
    ///
    /// # Environment Variable Format
    ///
    /// ```bash
    /// export SONGBIRD_BROADCAST_ADDRESSES="224.0.0.251:2300,192.168.1.255:2300"
    /// ```
    ///
    /// # Arguments
    ///
    /// * `configured_addrs` - Addresses from configuration file
    ///
    /// # Returns
    ///
    /// Vec of socket addresses including multicast + subnet broadcast fallback
    pub(crate) fn discover_broadcast_addresses(
        configured_addrs: &[String],
    ) -> Vec<std::net::SocketAddr> {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let port = songbird_types::defaults::network::broadcast_discovery_port();

        // Priority 1: Environment variable (runtime override)
        if let Ok(env_addrs) = songbird_process_env::var("SONGBIRD_BROADCAST_ADDRESSES")
            && !env_addrs.is_empty()
        {
            info!("🌐 Using broadcast addresses from SONGBIRD_BROADCAST_ADDRESSES");
            let addrs: Vec<SocketAddr> =
                env_addrs.split(',').filter_map(|s| s.trim().parse().ok()).collect();

            if !addrs.is_empty() {
                info!("   Addresses: {:?}", addrs);
                return addrs;
            }
        }

        // Priority 2: Configuration file
        let mut addrs: Vec<SocketAddr> =
            configured_addrs.iter().filter_map(|addr| addr.parse().ok()).collect();

        // Priority 3: Add subnet broadcast fallback if not already present
        // This enables cross-interface discovery (eth ↔ wifi) on consumer routers
        let default_fallbacks = [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255)), port), // Common home subnet
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 255)), port), // Alternative home subnet
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 255)), port),    // Corporate subnet
        ];

        for fallback_addr in &default_fallbacks {
            // Only add if not already configured
            if !addrs.iter().any(|a| a.ip() == fallback_addr.ip()) {
                addrs.push(*fallback_addr);
            }
        }

        if addrs.is_empty() {
            warn!("⚠️  No broadcast addresses configured, using defaults");
            addrs = vec![
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)), port),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255)), port),
            ];
        }

        info!("🌐 Discovery broadcast addresses: {:?}", addrs);
        addrs
    }

    /// Start Unix Socket IPC server for inter-primal communication (Jan 4, 2026)
    ///
    /// Starts a Unix socket server that allows other primals (security provider, compute provider / compute.schedule, etc.; formerly `ToadStool`)
    /// to register their capabilities and communicate with Songbird.
    ///
    /// Socket path resolved via [`crate::env_config::socket_path()`] (XDG-compliant):
    /// `SONGBIRD_SOCKET` > `BIOMEOS_SOCKET_DIR` > `$XDG_RUNTIME_DIR/biomeos/` > `$TMPDIR`.
    ///
    /// v3.19.2: Unix Socket IPC Server (port-free!)
    /// v3.20.0: Service registry mode - primals register themselves
    ///
    /// This creates a Unix domain socket for JSON-RPC 2.0 communication with other primals.
    /// Socket path is derived from `SONGBIRD_FAMILY_ID` env var (zero hardcoding!).
    ///
    /// This ensures multiple Songbird instances (spores) can run on the same machine
    /// without socket path conflicts, following security provider's pattern.
    #[cfg(unix)]
    pub(crate) async fn start_ipc_server(&self) -> Result<()> {
        use crate::ipc::{ServiceRegistry, UnixSocketServer};

        info!("🎧 Starting Unix Socket IPC server (v3.20.0 - Service Registry Mode)");
        info!(
            "   Family ID: {}",
            songbird_process_env::var("SONGBIRD_FAMILY_ID")
                .unwrap_or_else(|_| String::from("default"))
        );
        info!("   Protocol: JSON-RPC 2.0");

        // v3.20.0: Create service registry for primal registration
        let service_registry = Arc::new(ServiceRegistry::new());

        // v3.19.2: Pass individual components (cleaner than Arc<RwLock<whole orchestrator>>)
        let discovery_listener_clone = self.discovery_listener.clone();
        let connection_manager_clone = Arc::clone(&self.connection_manager);

        // v5.28.0: Discover crypto provider via capability-based discovery (zero identity hardcoding)
        // Priority: explicit env → capability env → family-scoped fallback
        let crypto_socket = crate::env_config::security_crypto_ipc_socket_from_env(|| {
            let family_id = songbird_process_env::var("SONGBIRD_FAMILY_ID")
                .or_else(|_| songbird_process_env::var("FAMILY_ID"))
                .unwrap_or_else(|_| String::from("default"));
            songbird_types::defaults::paths::family_scoped_crypto_socket_path(&family_id)
                .to_string_lossy()
                .into_owned()
        });

        info!("🔐 Security provider (Direct): {}", crypto_socket);
        let security_client =
            Arc::new(songbird_http_client::SecurityRpcClient::new_direct(crypto_socket));

        // v3.20.0: Create Unix socket server with service registry
        let server = Arc::new(UnixSocketServer::new(
            service_registry,
            discovery_listener_clone,
            connection_manager_clone,
            security_client,
        ));

        // Start server in background task (pure Rust server runs forever)
        let server_arc = Arc::clone(&server);
        let server_task = tokio::spawn(async move {
            if let Err(e) = server_arc.start().await {
                error!("❌ Unix Socket IPC server error: {}", e);
            }
        });

        // Wait for server to be ready (atomic, lock-free!)
        if !server.wait_ready(songbird_types::defaults::timeouts::DEFAULT_REQUEST_TIMEOUT).await {
            warn!("⚠️  Unix Socket IPC server did not become ready within 5 seconds");
        }

        info!("✅ Unix Socket IPC server started successfully");
        info!("   APIs: 11 total");
        info!(
            "   - Service Registry: register_service, discover_by_capability, get_service_health, health_check"
        );
        info!(
            "   - P2P Discovery: discover_by_family, create_genetic_tunnel, announce_capabilities"
        );
        info!(
            "   - Graph Intelligence: graph.validate, graph.check_availability, graph.suggest_alternatives, coordination.validate_pattern"
        );
        info!("   🌱 Primals can now register and discover each other!");

        // Store task handle for cleanup (would need to be added to orchestrator struct)
        // For now, server runs indefinitely in background
        drop(server_task); // Prevent unused warning

        Ok(())
    }

    /// Start the JSON-RPC IPC server (**known platform limitation on non-Unix**).
    ///
    /// Production IPC uses Unix domain sockets (`UnixSocketServer`). Native Windows builds
    /// do not start an equivalent transport (named pipes / TCP parity is not implemented).
    ///
    /// **Tracking**: Treat native Windows IPC as out of scope until an explicit transport
    /// design is scheduled; use [WSL2](https://learn.microsoft.com/windows/wsl/) for the same
    /// socket-based workflow as Linux.
    #[cfg(not(unix))]
    pub(crate) async fn start_ipc_server(&self) -> Result<()> {
        warn!(
            target_os = std::env::consts::OS,
            "IPC server is unavailable on this platform: Songbird's primal IPC requires Unix domain sockets"
        );
        warn!(
            "For Windows hosts, run Songbird under WSL2 or wait for tracked native IPC (named pipes / TCP) support"
        );
        Err(anyhow::anyhow!(
            "IPC server requires Unix domain sockets (Linux/macOS/BSD). On Windows use WSL2 for parity."
        ))
    }

    /// Start the tarpc binary RPC server for high-performance primal-to-primal communication.
    ///
    /// Binds on the port from `SONGBIRD_TARPC_PORT` (default 8091). Opt-out by setting
    /// `SONGBIRD_TARPC_ENABLED=false`. JSON-RPC over IPC remains the primary transport;
    /// tarpc provides a low-latency binary hot path for Rust-to-Rust calls.
    pub(crate) async fn start_tarpc_server(&self) -> Result<()> {
        let enabled = songbird_process_env::var("SONGBIRD_TARPC_ENABLED")
            .map(|v| songbird_types::error_helpers::parse_bool_relaxed(&v).unwrap_or(true))
            .unwrap_or(true);

        if !enabled {
            info!("tarpc server disabled via SONGBIRD_TARPC_ENABLED=false");
            return Ok(());
        }

        let bind_host = &self._config.network.bind_host;
        let port = songbird_config::defaults::ports::tarpc_port();

        crate::app::http_server::start_tarpc_server(
            Arc::clone(&self.federation_state),
            Arc::clone(&self.federated_service_registry),
            bind_host,
            port,
        )
        .await?;

        info!("tarpc binary RPC listening on {bind_host}:{port}");
        Ok(())
    }

    /// Verify external connectivity after startup.
    ///
    /// Delegates to [`connectivity::verify_external_connectivity`](super::connectivity)
    /// which tests HTTPS reachability, provides diagnostics, and attempts auto-remediation.
    pub(crate) async fn verify_external_connectivity(&self) -> Result<()> {
        super::connectivity::verify_external_connectivity().await
    }

    // Discovery bridge implementation moved to discovery_bridge.rs (v3.10.0 refactoring)

    /// Start session TTL cleanup task
    ///
    /// Deep Debt Fix (Dec 20, 2025):
    /// - Session IDs rotate hourly, creating "new" nodes for same tower
    /// - Without cleanup, federation accumulates stale entries (69 nodes for 4 towers!)
    /// - This task removes nodes that haven't sent heartbeat within TTL
    ///
    /// Lifecycle Evolution:
    /// - Runs every 5 minutes
    /// - TTL: 10 minutes (2x heartbeat interval)
    /// - Graceful cleanup with logging
    /// - Self-healing federation state
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub(crate) async fn start_session_ttl_cleanup(&self) -> Result<()> {
        let federation_state = Arc::clone(&self.federation_state);

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(songbird_types::defaults::timeouts::DEFAULT_CACHE_TTL);
            let ttl_secs = 600; // 10 minutes (2x heartbeat interval)

            info!("🧹 Session TTL cleanup task started (interval: 5min, TTL: 10min)");

            loop {
                interval.tick().await;

                let removed = federation_state.cleanup_stale_nodes(ttl_secs).await;

                if removed > 0 {
                    info!("🧹 TTL cleanup: Removed {} stale sessions", removed);
                }
            }
        });

        info!("✅ Session TTL cleanup task spawned");
        Ok(())
    }

    /// Start service registry cleanup task (Universal Port Authority)
    ///
    /// Cleans up stale registered services that have missed heartbeats.
    pub(crate) fn start_service_registry_cleanup(&self) {
        let registry = Arc::clone(&self.service_registry);

        drop(crate::service_registry::spawn_cleanup_task((*registry).clone(), 60)); // Clean every minute

        info!("✅ Service registry cleanup task started");
    }

    // ✅ MODERN RUST (v3.13.0 - Jan 7, 2026): Health check methods extracted to health.rs
    // Reduces core.rs from 1043→975 lines (-68 lines / 6.5% reduction)
    //
    // Extracted methods (all in health.rs):
    // - start_health_monitoring() - Spawns background health check loop
    // - get_status() - Returns current orchestrator status
    // - run_comprehensive_health_check() - Checks all subsystems
    // - check_gaming_manager_health() - Gaming subsystem check
    // - check_federation_manager_health() - Federation connectivity check
    // - check_observability_manager_health() - Metrics collection check
    // - check_security_integration_health() - Security subsystem check
    //
    // See: crates/songbird-orchestrator/src/app/health.rs for implementation
}

// Status, health check, and startup functions are now in their respective modules:
// - health::{OrchestratorStatus, HealthCheckReport, run_health_check}
// - startup::{start_orchestrator, Orchestrator}
// They are re-exported at the top of this module for backwards compatibility.

/// Shared `SONGBIRD_BROADCAST_ADDRESSES` mutex for all `discover_broadcast_addresses` tests.
///
/// Uses `std::sync::Mutex` on purpose: every caller is a **synchronous** `#[test]` that never
/// holds this guard across `.await`. A blocking mutex is fine for short test-only serialization;
/// `tokio::sync::Mutex` would not help here because there is no async boundary.
#[cfg(test)]
pub(crate) mod broadcast_test_lock {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub fn guard() -> std::sync::MutexGuard<'static, ()> {
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
#[path = "../core_broadcast_tests.rs"]
mod discover_broadcast_tests;
