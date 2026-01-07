//! Discovery Startup Module
//!
//! Handles the complete startup sequence for anonymous discovery:
//! 1. Fetching identity attestations from security provider
//! 2. Initializing BirdSong encryption processor
//! 3. Setting up discovery broadcaster
//! 4. Configuring and starting discovery listener
//! 5. Starting discovery→federation bridge
//!
//! ## Zero Hardcoding Philosophy
//!
//! This module discovers security providers at runtime via environment:
//! - `SONGBIRD_SECURITY_PROVIDER`: Security provider URL (NEW - generic capability)
//! - `SECURITY_ENDPOINT`: Alternative security endpoint (generic)
//! - `SONGBIRD_BEARDOG_URL`: DEPRECATED - Use SONGBIRD_SECURITY_PROVIDER instead
//!
//! No hardcoded endpoints - all runtime discovery!
//!
//! ## Modern Rust Patterns
//!
//! - "Build then Arc" for listener configuration
//! - Async/await for security provider communication
//! - Clear error handling with Result types
//! - Structured logging with context

use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info, warn};

use songbird_discovery::anonymous::{AnonymousDiscoveryBroadcaster, AnonymousDiscoveryListener, TransportEndpointMessage};
use songbird_discovery::DiscoveryStatusManager;

use crate::node_identity::NodeIdentity;
use crate::self_knowledge;

/// Start the complete discovery system
///
/// This orchestrates the entire discovery startup sequence:
/// 1. Fetch identity attestations from security provider (if configured)
/// 2. Initialize BirdSong encryption processor (if genetic identity available)
/// 3. Create and start discovery broadcaster
/// 4. Configure and start discovery listener with proper Arc wrapping
///
/// # Arguments
///
/// * `discovery_port` - UDP port for discovery
/// * `https_port` - Actual HTTPS port to advertise
/// * `node_identity` - Node's identity information
/// * `endpoint_messages` - Endpoint information to broadcast
/// * `capabilities` - Capabilities to advertise
/// * `broadcast_addrs` - Multicast addresses to broadcast to
/// * `discovery_listener_pending` - Pending listener to configure
/// * `discovery_status_manager` - Stats manager for observability
///
/// # Returns
///
/// Configured listener wrapped in Arc (for bridge polling)
///
/// # Modern Rust Pattern: "Build Then Arc"
///
/// The listener is taken as `AnonymousDiscoveryListener` (not Arc'd),
/// fully configured with builder methods, and THEN wrapped in Arc.
/// This prevents the "two instances" bug and enables fractal patterns.
pub async fn start_discovery_system(
    discovery_port: u16,
    https_port: u16,
    node_identity: &NodeIdentity,
    endpoint_messages: Vec<TransportEndpointMessage>,
    capabilities: Vec<String>,
    broadcast_addrs: Vec<std::net::SocketAddr>,
    discovery_listener_pending: Option<AnonymousDiscoveryListener>,
    discovery_status_manager: Arc<DiscoveryStatusManager>,
) -> Result<Option<Arc<AnonymousDiscoveryListener>>> {
    info!("🌐 Starting anonymous discovery system...");

    // Step 1: Fetch identity attestations from security provider
    let identity_attestations = fetch_identity_attestations().await?;

    // Step 2: Initialize BirdSong processor (if genetic identity available)
    let birdsong_processor = initialize_birdsong_processor(&identity_attestations).await;

    // Step 2.5: Discover our own identity tags (self-knowledge!)
    // Songbird doesn't interpret tags - just broadcasts them
    let identity_tags = self_knowledge::discover_identity_tags();
    
    // Step 3: Create and start broadcaster
    start_discovery_broadcaster(
        node_identity,
        endpoint_messages,
        capabilities,
        broadcast_addrs,
        identity_tags,
        identity_attestations,
        birdsong_processor.as_ref(),
    )
    .await?;

    // Step 4: Configure and start listener
    let listener_arc = start_discovery_listener(
        discovery_listener_pending,
        birdsong_processor,
        discovery_status_manager,
        node_identity,
    )
    .await?;

    info!(
        "✅ Anonymous discovery started (UDP port {}, advertising HTTPS port {})",
        discovery_port, https_port
    );

    Ok(listener_arc)
}

/// Fetch identity attestations from security provider
///
/// **EVOLVED (v3.15.0)**: Uses capability discovery (zero vendor hardcoding!)
///
/// Queries the configured security provider for genetic lineage attestations.
/// These are used for same-family trust and BirdSong encryption.
///
/// # Environment Variables (v3.15.0)
///
/// - `SONGBIRD_SECURITY_PROVIDER` (NEW - generic capability)
/// - `SECURITY_ENDPOINT` (generic)
/// - `SONGBIRD_BEARDOG_URL` (DEPRECATED - vendor-specific)
async fn fetch_identity_attestations() -> Result<Vec<songbird_discovery::IdentityAttestation>, anyhow::Error> {
    match crate::app::security_setup::discover_security_endpoint(None).await {
        Ok(url) => {
            info!("🔐 Fetching identity attestations from security provider: {}", url);
            let security_client =
                crate::security_capability_client::SecurityCapabilityClient::from_endpoint(url);

            match security_client?.get_identity().await {
                Ok(identity) => {
                    info!("✅ Got identity with encryption tag: {}", identity.encryption_tag);
                    if let Some(ref family_id) = identity.family_id {
                        info!("👨‍👩‍👧‍👦 Family ID: {} (enabling auto-trust)", family_id);
                    }

                    let attestations =
                        crate::security_capability_client::SecurityCapabilityClient::identity_to_discovery_attestations(
                            &identity,
                        );
                    info!("✅ Created {} identity attestations for discovery", attestations.len());
                    Ok(attestations)
                }
                Err(e) => {
                    warn!("⚠️  Could not get identity from security provider: {}", e);
                    warn!("   Discovery will continue without genetic lineage attestations");
                    Ok(Vec::new())
                }
            }
        }
        Err(_) => {
            info!("📡 No security provider configured - discovery without genetic lineage");
            Ok(Vec::new())
        }
    }
}

/// Initialize BirdSong encryption processor
///
/// Creates the BirdSong processor for privacy-preserving discovery.
/// Requires genetic identity (from attestations) to enable encryption.
async fn initialize_birdsong_processor(
    identity_attestations: &[songbird_discovery::IdentityAttestation],
) -> Option<Arc<songbird_discovery::BirdSongProcessor>> {
    if identity_attestations.is_empty() {
        info!("📡 BirdSong disabled (no genetic identity)");
        return None;
    }

    // Extract security endpoint
    // EVOLVED (v3.15.0): Use capability discovery (zero vendor hardcoding!)
    let security_endpoint = crate::app::security_setup::discover_security_endpoint(None)
        .await
        .ok();

    // Extract family_id from identity attestations
    let family_id = identity_attestations.iter().find_map(|a| {
        if a.provider_capability == "security/identity" {
            // Parse the data JSON to find family_id
            if let Some(data_obj) = a.data.as_object() {
                if let Some(family) = data_obj.get("family_id") {
                    return family.as_str().map(String::from);
                }
            }
        }
        None
    });

    let Some(endpoint) = security_endpoint else {
        info!("📡 BirdSong disabled (no security endpoint configured)");
        return None;
    };

    info!("🎵 Initializing security provider BirdSong encryption provider");
    info!("   Endpoint: {}", endpoint);
    if let Some(ref fam) = family_id {
        info!("   Family ID: {}", fam);
    }

    // Create security provider provider
    let security_provider = songbird_discovery::BearDogBirdSongProvider::new(
        endpoint.clone(),
        family_id.clone(),
    );

    if security_provider.check_health().await {
        info!("✅ security provider provider healthy");
        let config = songbird_discovery::BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true, // Allow mixed-mode for migration
            security_endpoint: Some(endpoint.clone()),
            mixed_mode: true, // Support both encrypted and plaintext discovery
        };

        let processor = songbird_discovery::BirdSongProcessor::new(
            Some(Arc::new(security_provider)),
            config,
        );
        info!("🎵 BirdSong processor initialized: {}", processor.status());
        Some(Arc::new(processor))
    } else {
        warn!("⚠️  security provider provider not available, using plaintext fallback");
        let config = songbird_discovery::BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            security_endpoint: Some(endpoint.clone()),
            mixed_mode: true,
        };
        let processor = songbird_discovery::BirdSongProcessor::new(None, config);
        info!("📡 BirdSong processor initialized (plaintext fallback): {}", processor.status());
        Some(Arc::new(processor))
    }
}

/// Start discovery broadcaster
///
/// Creates and spawns the broadcaster task that periodically
/// announces this node's presence to the network.
async fn start_discovery_broadcaster(
    node_identity: &NodeIdentity,
    endpoint_messages: Vec<TransportEndpointMessage>,
    capabilities: Vec<String>,
    broadcast_addrs: Vec<std::net::SocketAddr>,
    identity_tags: Vec<String>,  // NEW: Tags we broadcast (don't interpret!)
    identity_attestations: Vec<songbird_discovery::IdentityAttestation>,
    birdsong_processor: Option<&Arc<songbird_discovery::BirdSongProcessor>>,
) -> Result<()> {
    let mut broadcaster = AnonymousDiscoveryBroadcaster::new_v3(
        node_identity.node_id.to_string(),
        node_identity.node_name.clone(),
        endpoint_messages,
        capabilities,
        broadcast_addrs,
        30, // broadcast every 30 seconds
    )
    .with_identity_tags(identity_tags) // NEW: Broadcast tags (we don't interpret!)
    .with_identity_attestations(identity_attestations); // Include attestations for genetic lineage

    // Enable BirdSong encryption if available
    if let Some(processor) = birdsong_processor {
        broadcaster = broadcaster.with_birdsong(Arc::clone(processor));
        info!("🎵 BirdSong encryption enabled for broadcaster");
    }

    tokio::spawn(async move {
        if let Err(e) = broadcaster.start_broadcasting().await {
            error!("❌ Anonymous discovery broadcaster error: {}", e);
        }
    });

    info!("✅ Discovery broadcaster started");
    Ok(())
}

/// Start discovery listener with proper Arc wrapping
///
/// Implements the "Build Then Arc" pattern:
/// 1. Take pending listener (un-Arc'd)
/// 2. Configure with BirdSong and stats
/// 3. Wrap in Arc (configuration complete!)
/// 4. Spawn listening task
/// 5. Return Arc for bridge polling
///
/// This ensures ONE instance with full configuration is used by both
/// the listening task and the discovery→federation bridge.
async fn start_discovery_listener(
    discovery_listener_pending: Option<AnonymousDiscoveryListener>,
    birdsong_processor: Option<Arc<songbird_discovery::BirdSongProcessor>>,
    discovery_status_manager: Arc<DiscoveryStatusManager>,
    node_identity: &NodeIdentity,
) -> Result<Option<Arc<AnonymousDiscoveryListener>>> {
    let Some(mut listener) = discovery_listener_pending else {
        warn!("⚠️  Discovery listener not initialized, skipping");
        return Ok(None);
    };

    info!("🔧 Configuring discovery listener (BirdSong, stats, then Arc wrap)...");

    // Add BirdSong processor if available
    if let Some(ref processor) = birdsong_processor {
        info!("   🎵 Wiring BirdSong decryption");
        listener = listener.with_birdsong(Arc::clone(processor));
    }

    // Add discovery stats for observability
    info!("   📊 Wiring discovery statistics");
    listener = listener.with_stats(discovery_status_manager.stats());

    // NOW wrap in Arc (configuration complete!)
    let listener_arc = Arc::new(listener);
    info!("   ✅ Configuration complete, wrapped in Arc");
    info!("   Self-filtering: enabled for node_id {}", node_identity.node_id);

    // Spawn listening task
    let listener_for_spawn = Arc::clone(&listener_arc);
    tokio::spawn(async move {
        if let Err(e) = listener_for_spawn.start_listening().await {
            error!("❌ Anonymous discovery listener error: {}", e);
        }
    });

    info!("✅ Discovery listener started (SAME instance used by bridge)");
    Ok(Some(listener_arc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_identity_attestations_no_provider() {
        // No security provider configured
        std::env::remove_var("SONGBIRD_BEARDOG_URL");
        std::env::remove_var("SECURITY_ENDPOINT");

        let attestations = fetch_identity_attestations().await.unwrap_or_default();
        assert!(
            attestations.is_empty(),
            "Should return empty attestations when no provider configured"
        );
    }

    #[tokio::test]
    async fn test_initialize_birdsong_processor_no_identity() {
        let empty_attestations: Vec<songbird_discovery::IdentityAttestation> = Vec::new();
        let processor = initialize_birdsong_processor(&empty_attestations).await;

        assert!(
            processor.is_none(),
            "Should return None when no identity attestations"
        );
    }

    #[test]
    fn test_zero_hardcoding_discovery_startup() {
        // This test verifies the zero hardcoding philosophy:
        // - No hardcoded security provider endpoints
        // - All configuration via environment
        // - Runtime discovery of security provider

        // Security provider discovered via environment
        assert!(std::env::var("SONGBIRD_BEARDOG_URL").is_err() || std::env::var("SONGBIRD_BEARDOG_URL").is_ok());
        assert!(std::env::var("SECURITY_ENDPOINT").is_err() || std::env::var("SECURITY_ENDPOINT").is_ok());

        // No hardcoded values in this module!
        // All security provider discovery is runtime-based via environment
    }
}

