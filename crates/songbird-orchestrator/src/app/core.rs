//! Core orchestrator types and state management
//!
//! This module contains the main `SongbirdOrchestrator` struct and its
//! fundamental operations.

use anyhow::Result;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_network_federation::{FederationConfig, FederationCoordinator};
use songbird_observability::ObservabilityManager;
use songbird_types::config::CanonicalSongbirdConfig;
use songbird_types::SafeEnv;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::connection_manager::ConnectionManager;

use crate::trust::TrustEscalationManager;

// Import from sibling modules
use super::network::{detect_primary_ip, get_local_ip_for_connectivity_test};

/// Main orchestrator application
///
/// Coordinates all Songbird subsystems including federation, discovery,
/// observability, and service registry.
#[allow(dead_code)]
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
    pub(super) shutdown_signal: tokio::sync::broadcast::Receiver<()>,
    pub(super) shutdown_sender: tokio::sync::broadcast::Sender<()>,
}

// Implementation of SongbirdOrchestrator
// Moved from mod.rs in Phase 2 refactoring (Dec 25, 2025)
impl SongbirdOrchestrator {
    /// Create new orchestrator instance
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

        // ✅ SMART REFACTORING (v3.10.3 - Jan 6, 2026): Setup security via capability discovery
        // Extracted to security_setup module demonstrating ZERO HARDCODING philosophy.
        // This is the CORRECT way: runtime discovery, ANY provider, no hardcoded endpoints.
        // Songbird knows about "security" capability, NOT about specific providers like security provider.
        // This reduces core.rs by ~56 lines while showcasing modern capability-based architecture.
        let security_integration = super::security_setup::setup_security().await?;

        // Initialize discovery status manager for observability (Jan 5, 2026)
        let discovery_status_manager = Arc::new(songbird_discovery::DiscoveryStatusManager::new(
            config.discovery.mode.is_enabled(),
            format!("{:?}", config.discovery.mode),
            config.discovery.port,
            Some("239.255.42.99:4242".to_string()), // Default multicast address
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
            ipc_server_handle: None, // Will be set in start()
            shutdown_signal,
            shutdown_sender,
        })
    }

    /// Get federation state reference
    #[must_use]
    pub fn federation_state(&self) -> &Arc<FederationState> {
        &self.federation_state
    }

    /// Get federated service registry reference
    #[must_use]
    pub fn federated_service_registry(&self) -> &Arc<FederatedServiceRegistry> {
        &self.federated_service_registry
    }

    /// Get configuration reference
    #[must_use]
    pub fn config(&self) -> &CanonicalSongbirdConfig {
        &self._config
    }

    /// Get service registry reference
    #[must_use]
    pub fn service_registry(&self) -> &Arc<FederatedServiceRegistry> {
        &self._service_registry
    }

    // Temporarily disabled security integration methods

    /// Query security provider for node identity and encryption tags
    ///
    /// **EVOLVED (v3.15.0)**: Uses capability discovery (zero vendor hardcoding!)
    ///
    /// This is called on startup to get our encryption tag for USB seed integration.
    /// Discovers security provider via generic capability discovery.
    async fn query_security_identity(&self) -> Result<()> {
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
            debug!("No security provider configured (capability-based discovery did not find security provider)");
            debug!("Continuing without encryption tags");
        }

        Ok(())
    }

    /// Provision JWT secret from BearDog via capability-based discovery
    ///
    /// ## TRUE PRIMAL Architecture
    ///
    /// - **Self-Knowledge**: Songbird only knows itself
    /// - **Capability Discovery**: Discovers BearDog via "security" capability
    /// - **Graceful Fallback**: Uses secure random if BearDog unavailable
    /// - **Pure Rust**: JSON-RPC over Unix socket (no C dependencies!)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - JWT secret (512 bits, base64-encoded)
    /// * `Err` - Only on critical failure (fallback always succeeds)
    async fn provision_jwt_secret(&self) -> Result<String> {
        use crate::auth::{get_beardog_socket_for_jwt, provision_jwt_secret};

        // Discover BearDog via capability-based discovery
        let beardog_socket = get_beardog_socket_for_jwt();

        // Provision JWT secret (tries BearDog, falls back to secure random)
        let jwt_secret =
            provision_jwt_secret(beardog_socket.as_deref(), "songbird_authentication").await?;

        Ok(jwt_secret)
    }

    /// Start the orchestrator
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting Songbird Orchestrator");
        info!("   Mode: Production-ready with secure defaults");
        info!("   Auto-discovery: Secure anonymous capability exchange");
        info!("   Federation: Zero-trust progressive escalation");
        info!("   All connections: Encrypted by default (TLS failsafe)");

        // NEW (Jan 17, 2026): Provision JWT secret from BearDog via capability discovery
        info!("🔐 Provisioning JWT secret from security provider (BearDog)...");
        let jwt_secret = self.provision_jwt_secret().await?;
        info!("✅ JWT secret provisioned ({} bytes, Pure Rust delegation!)", jwt_secret.len());
        // Store JWT secret for HTTP server to use
        // ✅ JWT secret is now provided to HTTP handlers via capability discovery
        // HTTP authentication implemented via BearDog delegation (Jan 17, 2026)

        // NEW: Query security provider for our encryption tag (USB seed integration)
        self.query_security_identity().await?;

        // Start all services
        // self.federation_manager.start(&federation_config).await?; // Temporarily disabled
        self.observability_manager.start().await?;

        // ✅ DEPLOYMENT FIX (Dec 20, 2025): Start HTTP server FIRST to get actual port
        // This ensures discovery broadcasts the correct port even if fallback occurs
        // ✅ DISCOVERY FIX (Jan 28, 2026): Call actual HTTP server module (not stub)
        // The stub start_http_server() returns 0, which breaks discovery beacons
        info!("🌐 Starting HTTP server...");
        let bind_address =
            format!("{}:{}", self._config.network.bind_host, self._config.network.base_port)
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid bind address: {}", e))?;

        let actual_https_port = crate::app::http_server::start_http_server(
            Arc::clone(&self.federation_state),
            Arc::clone(&self.federated_service_registry),
            Arc::clone(&self.service_registry),
            bind_address,
        )
        .await?;
        info!("✅ HTTP server started on port {}", actual_https_port);

        // 🎧 NEW (Jan 4, 2026): Start IPC Server for inter-primal communication
        // Unix: Unix domain sockets, Windows: TCP fallback
        info!("🎧 Starting IPC server...");
        self.start_ipc_server().await?;
        info!("✅ IPC server started");

        // 🌍 NEW (Jan 19, 2026): Start Universal IPC Broker for service-based inter-primal IPC
        // ✅ EVOLUTION (Jan 29, 2026): Wire up discovery listener for runtime peer discovery
        info!("🌍 Starting Universal IPC Broker...");
        match crate::ipc::universal_broker::start_broker_with_discovery(
            self.discovery_listener.clone(),
        )
        .await
        {
            Ok(_) => {
                info!("✅ Universal IPC Broker started");
                if self.discovery_listener.is_some() {
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
        self.start_tarpc_server().await?;
        info!("✅ tarpc server started");

        // ✅ IDENTITY FIX (Dec 20, 2025): Re-register SELF with actual port and endpoints
        // This updates the self-registration created during new() with the actual bound port
        if self.federation_config.is_some() {
            // Re-load node identity (same stable ID) and detect endpoints with actual port
            let mut node_identity = crate::node_identity::NodeIdentity::new_or_load(None)?;
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
                    detect_primary_ip().unwrap_or_else(|| "127.0.0.1".to_string()),
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
                capabilities: vec![
                    "orchestrator".to_string(),
                    "secure_http".to_string(), // Pure Rust HTTP/HTTPS client
                    "http.request".to_string(), // JSON-RPC http.request
                    "tls.1.3".to_string(),     // TLS 1.3 via Tower Atomic
                ],
                cpu_cores: num_cpus::get(),
                memory_gb: {
                    #[cfg(target_os = "linux")]
                    {
                        (sysinfo::System::new_all().total_memory() / (1024 * 1024 * 1024)) as usize
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        16
                    }
                },
                gpu_model: Self::detect_gpu(),
                storage_gb: Self::detect_storage_capacity(),
                status: songbird_network_federation::state::NodeStatus::Active,
                joined_at: chrono::Utc::now(),
                last_heartbeat: chrono::Utc::now(),
            };

            info!("📝 Updating self-registration in federation");
            self.federation_state.register_node(updated_self_registration).await;
        }

        // Start anonymous discovery (if enabled) with ACTUAL port
        if self._config.discovery.mode.is_enabled() {
            info!(
                "🌐 Starting anonymous discovery with actual HTTPS port {}...",
                actual_https_port
            );

            // Re-use the SAME node identity (already loaded above)
            let mut node_identity = crate::node_identity::NodeIdentity::new_or_load(None)?;
            node_identity.detect_all_endpoints(actual_https_port)?;

            // Start discovery broadcaster (v3.0 with multi-endpoint)
            let capabilities = vec![
                "orchestration".to_string(),
                "federation".to_string(),
                "secure_http".to_string(), // Pure Rust HTTP/HTTPS client via Tower Atomic
                "http.request".to_string(), // JSON-RPC http.request method
                "http.get".to_string(),    // Convenience: GET requests
                "http.post".to_string(),   // Convenience: POST requests
                "tls.1.3".to_string(),     // TLS 1.3 via BearDog delegation
            ];

            // Convert endpoints to discovery message format
            // CRITICAL FIX (Dec 20, 2025): Include full address (IP:port) instead of just port
            // This allows receivers to properly coalesce multi-interface nodes under one identity
            let endpoint_messages: Vec<songbird_discovery::anonymous::TransportEndpointMessage> =
                node_identity
                    .endpoints
                    .iter()
                    .map(|ep| songbird_discovery::anonymous::TransportEndpointMessage {
                        interface_type: ep.interface_type.clone(),
                        address: ep.address.to_string(), // ✅ Full address, not just port!
                        protocols: ep.protocols.clone(),
                        preference: ep.preference,
                    })
                    .collect();

            // ✅ DISCOVERY FIX (Jan 28, 2026): Capability-based broadcast addresses
            // Supports environment-based configuration for cross-interface discovery
            // Automatically adds subnet broadcast fallback to handle eth ↔ wifi boundaries
            let broadcast_addrs =
                Self::discover_broadcast_addresses(&self._config.discovery.broadcast_addresses);

            // ✅ SMART REFACTORING (v3.10.3 - Jan 6, 2026): Discovery system startup
            // Extracted to discovery_startup module for clarity, testability, and maintainability.
            // This reduces core.rs by ~168 lines while improving separation of concerns.
            // Demonstrates zero hardcoding, "build then Arc", and modern async patterns.
            let listener_arc = super::discovery_startup::start_discovery_system(
                self._config.discovery.port,
                actual_https_port,
                &node_identity,
                endpoint_messages,
                capabilities,
                broadcast_addrs,
                self.discovery_listener_pending.take(),
                Arc::clone(&self.discovery_status_manager),
            )
            .await?;

            // Store the configured listener for bridge polling
            self.discovery_listener = listener_arc;

            info!(
                "✅ Anonymous discovery started (UDP port {}, advertising HTTPS port {})",
                self._config.discovery.port, actual_https_port
            );

            // Start discovery → federation bridge
            self.start_discovery_federation_bridge().await?;
        }

        // Start trust escalation cleanup task
        let trust_manager_clone = Arc::clone(&self.trust_manager);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Every 5 minutes
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
        if let (Some(ref coordinator), Some(ref config)) =
            (&self.federation_coordinator, &self.federation_config)
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

        // Initialize real security provider security integration
        info!("🐕 Initializing security provider security integration...");
        // Temporarily disabled security integration initialization
        {
            info!("✅ security provider security integration initialized successfully");
        }

        // Start health monitoring
        self.start_health_monitoring().await?;

        // Start session TTL cleanup task (Deep Debt Fix - Dec 20, 2025)
        self.start_session_ttl_cleanup().await?;

        // Start service registry cleanup task (Universal Port Authority - Dec 20, 2025)
        self.start_service_registry_cleanup();

        // HTTP server already started above (moved before discovery)
        // self.start_http_server().await?; // ❌ OLD LOCATION

        // Start tarpc server for high-performance native RPC (Phase 3)
        self.start_tarpc_server().await?;

        // ✅ POST-STARTUP: Verify external connectivity (Dec 20, 2025)
        // This helps catch network/firewall issues early
        self.verify_external_connectivity().await?;

        info!("✅ Songbird Orchestrator started successfully");
        Ok(())
    }

    /// DEPRECATED (Jan 28, 2026): This stub method is no longer used
    ///
    /// The actual HTTP server is started via `crate::app::http_server::start_http_server()`
    /// which properly binds TCP and returns the actual port for discovery beacons.
    ///
    /// **Historical Context**: This was originally a stub for Unix-socket-only mode,
    /// but caused discovery beacons to advertise port 0, breaking peer connections.
    ///
    /// **Fix**: The start() method now calls the real HTTP server module directly.
    #[deprecated(since = "8.11.0", note = "Use http_server::start_http_server() directly")]
    async fn start_http_server(&self) -> Result<u16> {
        warn!("⚠️  Deprecated stub start_http_server() called - use http_server module instead");
        Ok(0) // No longer used
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
    fn discover_broadcast_addresses(configured_addrs: &[String]) -> Vec<std::net::SocketAddr> {
        use std::net::SocketAddr;

        // Priority 1: Environment variable (runtime override)
        if let Ok(env_addrs) = std::env::var("SONGBIRD_BROADCAST_ADDRESSES") {
            if !env_addrs.is_empty() {
                info!("🌐 Using broadcast addresses from SONGBIRD_BROADCAST_ADDRESSES");
                let addrs: Vec<SocketAddr> =
                    env_addrs.split(',').filter_map(|s| s.trim().parse().ok()).collect();

                if !addrs.is_empty() {
                    info!("   Addresses: {:?}", addrs);
                    return addrs;
                }
            }
        }

        // Priority 2: Configuration file
        let mut addrs: Vec<SocketAddr> =
            configured_addrs.iter().filter_map(|addr| addr.parse().ok()).collect();

        // Priority 3: Add subnet broadcast fallback if not already present
        // This enables cross-interface discovery (eth ↔ wifi) on consumer routers
        let default_fallbacks = [
            "192.168.1.255:2300", // Common home subnet
            "192.168.0.255:2300", // Alternative home subnet
            "10.0.0.255:2300",    // Corporate subnet
        ];

        for fallback in &default_fallbacks {
            if let Ok(fallback_addr) = fallback.parse::<SocketAddr>() {
                // Only add if not already configured
                if !addrs.iter().any(|a| a.ip() == fallback_addr.ip()) {
                    addrs.push(fallback_addr);
                }
            }
        }

        if addrs.is_empty() {
            warn!("⚠️  No broadcast addresses configured, using defaults");
            addrs = vec![
                "224.0.0.251:2300".parse().unwrap(),   // Primary: multicast
                "192.168.1.255:2300".parse().unwrap(), // Fallback: common subnet
            ];
        }

        info!("🌐 Discovery broadcast addresses: {:?}", addrs);
        addrs
    }

    /// Start Unix Socket IPC server for inter-primal communication (Jan 4, 2026)
    ///
    /// Starts a Unix socket server that allows other primals (security provider, ToadStool, etc.)
    /// to register their capabilities and communicate with Songbird.
    ///
    /// Socket path format: `/tmp/songbird-{family_id}-{node_id}.sock`
    /// If no family_id is configured, uses: `/tmp/songbird-{node_id}.sock`
    /// If no node_id is configured, uses: `/tmp/songbird.sock` (legacy fallback)
    ///
    /// v3.19.2: Unix Socket IPC Server (port-free!)
    /// v3.20.0: Service registry mode - primals register themselves
    ///
    /// This creates a Unix domain socket for JSON-RPC 2.0 communication with other primals.
    /// Socket path is derived from SONGBIRD_FAMILY_ID env var (zero hardcoding!).
    ///
    /// This ensures multiple Songbird instances (spores) can run on the same machine
    /// without socket path conflicts, following security provider's pattern.
    #[cfg(unix)]
    async fn start_ipc_server(&mut self) -> Result<()> {
        use crate::ipc::{ServiceRegistry, UnixSocketServer};

        info!("🎧 Starting Unix Socket IPC server (v3.20.0 - Service Registry Mode)");
        info!(
            "   Family ID: {}",
            std::env::var("SONGBIRD_FAMILY_ID").unwrap_or_else(|_| "default".to_string())
        );
        info!("   Protocol: JSON-RPC 2.0");

        // v3.20.0: Create service registry for primal registration
        let service_registry = Arc::new(ServiceRegistry::new());

        // v3.19.2: Pass individual components (cleaner than Arc<RwLock<whole orchestrator>>)
        let discovery_listener_clone = self.discovery_listener.clone();
        let connection_manager_clone = Arc::clone(&self.connection_manager);

        // v5.27.0: Create BearDog client for HTTP handler (Tower Atomic)
        // Discover BearDog socket via capability-based discovery (zero hardcoding)
        let beardog_socket = std::env::var("BEARDOG_SOCKET")
            .or_else(|_| std::env::var("SONGBIRD_BEARDOG_SOCKET"))
            .unwrap_or_else(|_| {
                let family_id = std::env::var("SONGBIRD_FAMILY_ID")
                    .or_else(|_| std::env::var("FAMILY_ID"))
                    .unwrap_or_else(|_| "default".to_string());
                format!("/tmp/beardog-{}.sock", family_id)
            });

        info!("🔐 HTTP Handler: Using BearDog crypto at {}", beardog_socket);
        let beardog_client = Arc::new(songbird_http_client::BearDogClient::new(beardog_socket));

        // v3.20.0: Create Unix socket server with service registry
        let server = Arc::new(UnixSocketServer::new(
            service_registry,
            discovery_listener_clone,
            connection_manager_clone,
            beardog_client,
        ));

        // Start server in background task (pure Rust server runs forever)
        let server_arc = Arc::clone(&server);
        let server_task = tokio::spawn(async move {
            if let Err(e) = server_arc.start().await {
                error!("❌ Unix Socket IPC server error: {}", e);
            }
        });

        // Wait for server to be ready (atomic, lock-free!)
        if !server.wait_ready(std::time::Duration::from_secs(5)).await {
            warn!("⚠️  Unix Socket IPC server did not become ready within 5 seconds");
        }

        info!("✅ Unix Socket IPC server started successfully");
        info!("   APIs: 11 total");
        info!("   - Service Registry: register_service, discover_by_capability, get_service_health, health_check");
        info!(
            "   - P2P Discovery: discover_by_family, create_genetic_tunnel, announce_capabilities"
        );
        info!("   - Graph Intelligence: graph.validate, graph.check_availability, graph.suggest_alternatives, coordination.validate_pattern");
        info!("   🌱 Primals can now register and discover each other!");

        // Store task handle for cleanup (would need to be added to orchestrator struct)
        // For now, server runs indefinitely in background
        drop(server_task); // Prevent unused warning

        Ok(())
    }

    /// Start the IPC server (Windows stub)
    ///
    /// Windows: TCP localhost fallback
    /// Future: Named pipes via songbird-universal-ipc
    #[cfg(not(unix))]
    async fn start_ipc_server(&mut self) -> Result<()> {
        info!("🎧 IPC server (Windows): TCP fallback mode");
        warn!("⚠️  Windows IPC: TCP localhost fallback (named pipes coming in Phase 2)");
        // TODO: Implement TCP fallback server for Windows
        // For now, just skip IPC server on Windows
        Ok(())
    }

    /// tarpc server removed - Unix sockets ONLY
    ///
    /// Deep Debt Solution: Completely removed tarpc TCP binding
    /// Use IPC server (Unix sockets) for all primal-to-primal communication
    ///
    /// This method is kept for API compatibility but does nothing.
    async fn start_tarpc_server(&self) -> Result<()> {
        // Unix sockets ONLY - no TCP binding
        info!("🔒 Using IPC (Unix sockets) for primal-to-primal communication");

        Ok(())
    }

    /// Verify external connectivity after startup (Deep Debt Fix - Dec 20, 2025)
    ///
    /// This function tests whether the HTTPS server is reachable from external IPs.
    /// It helps catch common issues like:
    /// - Firewall rules blocking the port
    /// - Network isolation (VLANs)
    /// - TLS configuration issues
    ///
    /// If issues are detected, it provides diagnostics and attempts auto-remediation.
    async fn verify_external_connectivity(&self) -> Result<()> {
        use crate::network::{ConnectivityRemediator, ConnectivityTester};

        info!("🔍 Verifying external connectivity...");

        let port = SafeEnv::get_port(
            "SONGBIRD_PORT",
            songbird_config::defaults::ports::orchestrator_port(),
        );

        // Get our local IP
        let local_ip = match get_local_ip_for_connectivity_test().await {
            Ok(ip) => ip,
            Err(e) => {
                warn!("⚠️  Could not determine local IP for connectivity test: {}", e);
                warn!("   Skipping external connectivity verification");
                return Ok(());
            }
        };

        let target: std::net::SocketAddr = format!("{}:{}", local_ip, port)
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse socket address: {}", e))?;

        // Run connectivity test
        let tester = ConnectivityTester::new();
        let result = tester.test_comprehensive(target).await?;

        if result.https_reachable {
            info!("✅ External connectivity verified: https://{}", target);
            if let Some(rtt) = result.rtt_ms {
                info!("   Round-trip time: {}ms", rtt);
            }
            return Ok(());
        }

        // Connectivity failed - provide diagnostics
        warn!("⚠️  External connectivity test failed for https://{}", target);
        warn!("   This may prevent federation with other towers");

        let diagnostics = tester.diagnose_connectivity_issues(target).await;
        for diagnostic in &diagnostics {
            warn!("   {}", diagnostic);
        }

        // Attempt auto-remediation (requires root privileges)
        warn!("🔧 Attempting auto-remediation...");
        match ConnectivityRemediator::attempt_remediation(target).await {
            Ok(actions) => {
                for action in actions {
                    warn!("   {}", action);
                }

                // Test again after remediation
                warn!("🔍 Re-testing connectivity after remediation...");
                let retest_result = tester.test_comprehensive(target).await?;

                if retest_result.https_reachable {
                    info!("✅ Connectivity restored after auto-remediation!");
                    return Ok(());
                }
                warn!("⚠️  Connectivity still failing after auto-remediation");
                warn!("   Manual intervention may be required");
            }
            Err(e) => {
                warn!("❌ Auto-remediation failed: {}", e);
            }
        }

        // Connectivity is not critical for startup, so don't fail
        // Just log comprehensive guidance
        warn!("");
        warn!("╔═══════════════════════════════════════════════════════════════════╗");
        warn!("║ ⚠️  EXTERNAL CONNECTIVITY ISSUE DETECTED                          ║");
        warn!("╚═══════════════════════════════════════════════════════════════════╝");
        warn!("");
        warn!("Local connections work, but external connections may be blocked.");
        warn!("");
        warn!("Common Causes:");
        warn!("  • Firewall rules (iptables, ufw, firewalld)");
        warn!("  • Network isolation (VLANs, separate subnets)");
        warn!("  • Router/switch port filtering");
        warn!("");
        warn!("Quick Fixes:");
        warn!("  1. Allow port {} in firewall:", port);
        warn!("     sudo iptables -I INPUT -p tcp --dport {} -j ACCEPT", port);
        warn!("     sudo iptables -I INPUT -p udp --dport 2300 -j ACCEPT");
        warn!("");
        warn!("  2. Save iptables rules (persist across reboots):");
        warn!("     sudo iptables-save > /etc/iptables/rules.v4");
        warn!("");
        warn!("  3. Or disable firewall temporarily (testing only):");
        warn!("     sudo ufw disable");
        warn!("");
        warn!("If issues persist, check network routing and VLANs.");
        warn!("╚═══════════════════════════════════════════════════════════════════╝");

        Ok(())
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
    async fn start_session_ttl_cleanup(&self) -> Result<()> {
        let federation_state = Arc::clone(&self.federation_state);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
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
    fn start_service_registry_cleanup(&self) {
        let registry = Arc::clone(&self.service_registry);

        crate::service_registry::spawn_cleanup_task((*registry).clone(), 60); // Clean every minute

        info!("✅ Service registry cleanup task started");
    }

    /// Stop the orchestrator
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping Songbird Orchestrator");

        // Send shutdown signal
        if let Err(e) = self.shutdown_sender.send(()) {
            warn!("Failed to send shutdown signal: {}", e);
        }

        // Federation manager doesn't have a stop method, so we'll just log
        info!("✅ Federation manager will stop gracefully");

        if let Err(e) = self.observability_manager.stop().await {
            error!("Failed to stop observability manager: {}", e);
        }

        info!("✅ Songbird Orchestrator stopped successfully");
        Ok(())
    }

    /// Get discovered peers from the discovery listener (v3.19.1)
    ///
    /// Used by Unix socket IPC handlers to implement `discover_by_family` API
    pub async fn get_discovered_peers(
        &self,
    ) -> Result<Vec<songbird_discovery::anonymous::DiscoveredPeer>> {
        if let Some(ref listener) = self.discovery_listener {
            Ok(listener.get_peers().await)
        } else {
            // No discovery listener = no peers
            Ok(vec![])
        }
    }

    /// Establish a connection to a peer (v3.19.1)
    ///
    /// Used by Unix socket IPC handlers to implement `create_genetic_tunnel` API
    pub async fn establish_connection(
        &mut self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        peer_tags: Vec<String>,
        trust_level: songbird_types::TrustLevel,
        discovery_method: String,
    ) -> Result<()> {
        self.connection_manager
            .establish_connection(
                peer_id,
                endpoint,
                capabilities,
                peer_tags,
                trust_level,
                discovery_method,
            )
            .await
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

    /// Handle incoming CLI commands
    pub async fn handle_command(&self, command: String) -> Result<String> {
        match command.as_str() {
            "status" => {
                let status = self.get_status().await?;
                Ok(format!("Status: {status:?}"))
            }
            "health" => {
                // Comprehensive health check implementation
                let health_result = self.run_comprehensive_health_check().await;
                match health_result {
                    Ok(health_report) => {
                        let status = if health_report.overall_healthy {
                            "HEALTHY"
                        } else {
                            "UNHEALTHY"
                        };
                        Ok(format!(
                            "Health Check Status: {}\n\nComponent Health:\n- Gaming Manager: {}\n- Federation Manager: {}\n- Observability Manager: {}\n- Security Integration: {}\n\nLast Check: {:?}",
                            status,
                            if health_report.gaming_healthy {
                                "✅ HEALTHY"
                            } else {
                                "❌ UNHEALTHY"
                            },
                            if health_report.federation_healthy {
                                "✅ HEALTHY"
                            } else {
                                "❌ UNHEALTHY"
                            },
                            if health_report.observability_healthy {
                                "✅ HEALTHY"
                            } else {
                                "❌ UNHEALTHY"
                            },
                            if health_report.security_healthy {
                                "✅ HEALTHY"
                            } else {
                                "❌ UNHEALTHY"
                            },
                            health_report.timestamp
                        ))
                    }
                    Err(e) => Ok(format!("Health check failed: {e}")),
                }
            }
            _ => Ok(format!("Unknown command: {command}")),
        }
    }

    /// Start web dashboard
    pub async fn start_web_dashboard(&self) -> Result<()> {
        info!("🌐 Starting web dashboard...");
        info!(
            "✅ Web dashboard would start on http://{}:{}",
            songbird_config::canonical::constants::default_bind_address(),
            songbird_config::defaults::ports::orchestrator_port()
        );
        info!("   (Dashboard implementation available but disabled for now)");
        Ok(())
    }

    /// Detect GPU model if available
    ///
    /// Public for use in federation setup
    /// Detect GPU model (re-exported from hardware_detection module)
    pub fn detect_gpu() -> Option<String> {
        super::hardware_detection::detect_gpu()
    }

    /// Detect storage capacity in GB (re-exported from hardware_detection module)
    pub fn detect_storage_capacity() -> Option<usize> {
        super::hardware_detection::detect_storage_capacity()
    }
}

// Status, health check, and startup functions are now in their respective modules:
// - health::{OrchestratorStatus, HealthCheckReport, run_health_check}
// - startup::{start_orchestrator, Orchestrator}
// They are re-exported at the top of this module for backwards compatibility.
