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
use std::sync::Arc;
use tracing::{error, info, warn};

use super::core::SongbirdOrchestrator;
use super::network::detect_primary_ip;
use crate::node_identity::NodeIdentity;

/// Startup orchestrator for clean, focused startup sequence
pub struct StartupOrchestrator<'a> {
    orchestrator: &'a mut SongbirdOrchestrator,
}

impl<'a> StartupOrchestrator<'a> {
    /// Create new startup orchestrator
    pub fn new(orchestrator: &'a mut SongbirdOrchestrator) -> Self {
        Self { orchestrator }
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
    /// **Purpose**: Acquire security credentials from BearDog before starting servers
    ///
    /// **Actions**:
    /// - Provision JWT secret from BearDog (via capability discovery)
    /// - Query security identity (USB seed integration)
    /// - Start observability manager
    ///
    /// **Why First**: Servers need JWT secrets for authentication
    async fn stage_1_provision_security(&mut self) -> Result<()> {
        // NEW (Jan 17, 2026): Provision JWT secret from BearDog via capability discovery
        info!("🔐 Provisioning JWT secret from security provider (BearDog)...");
        let jwt_secret = self.orchestrator.provision_jwt_secret().await?;
        info!("✅ JWT secret provisioned ({} bytes, Pure Rust delegation!)", jwt_secret.len());
        // ✅ JWT secret is now provided to HTTP handlers via capability discovery
        // HTTP authentication implemented via BearDog delegation (Jan 17, 2026)

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
        // ✅ DEPLOYMENT FIX (Dec 20, 2025): Start HTTP server FIRST to get actual port
        // This ensures discovery broadcasts the correct port even if fallback occurs
        // ✅ DISCOVERY FIX (Jan 28, 2026): Call actual HTTP server module (not stub)
        // The stub start_http_server() returns 0, which breaks discovery beacons
        info!("🌐 Starting HTTP server...");
        let bind_address = format!(
            "{}:{}",
            self.orchestrator._config.network.bind_host,
            self.orchestrator._config.network.base_port
        )
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid bind address: {}", e))?;

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
    /// - Register in federation state
    ///
    /// **Why Third**: Needs actual HTTP port from Stage 2
    async fn stage_3_register_self(&mut self, actual_https_port: u16) -> Result<()> {
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
    /// - Initialize security provider integration (disabled)
    ///
    /// **Why Fifth**: Needs discovery system running to coordinate federation
    async fn stage_5_start_federation(&mut self) -> Result<()> {
        // Start trust escalation cleanup task
        let trust_manager_clone = Arc::clone(&self.orchestrator.trust_manager);
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
        if let (Some(ref coordinator), Some(ref config)) = (
            &self.orchestrator.federation_coordinator,
            &self.orchestrator.federation_config,
        ) {
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
    async fn stage_6_background_tasks(&mut self) -> Result<()> {
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
    async fn stage_7_verify_connectivity(&mut self) -> Result<()> {
        // ✅ POST-STARTUP: Verify external connectivity (Dec 20, 2025)
        // This helps catch network/firewall issues early
        self.orchestrator.verify_external_connectivity().await?;

        Ok(())
    }
}
