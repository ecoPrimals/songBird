// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN discovery traversal strategies (sovereignty-first vs fastest-first).

#![forbid(unsafe_code)]

use crate::error::{LineageRelayError, Result};
use songbird_types::config::stun_relay::StunStrategy;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::MultiTierCoordinator;

/// Internal traversal strategy for public-address discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TraversalStrategy {
    /// Try user-provided STUN, then public STUN (if enabled), sequentially.
    SovereigntyFirst,
    /// Attempt all enabled STUN tiers in parallel; first success wins.
    FastestFirst,
    /// Genetic lineage relay only — STUN discovery is not used.
    LineageOnly,
}

impl From<StunStrategy> for TraversalStrategy {
    fn from(strategy: StunStrategy) -> Self {
        match strategy {
            StunStrategy::SovereigntyFirst => Self::SovereigntyFirst,
            StunStrategy::FastestFirst => Self::FastestFirst,
            StunStrategy::LineageOnly => Self::LineageOnly,
        }
    }
}

impl MultiTierCoordinator {
    /// Route discovery to the configured traversal strategy.
    pub(super) async fn discover_by_strategy(
        &self,
        strategy: TraversalStrategy,
    ) -> Result<SocketAddr> {
        match strategy {
            TraversalStrategy::SovereigntyFirst => self.sovereignty_first_discovery().await,
            TraversalStrategy::FastestFirst => self.fastest_first_discovery().await,
            TraversalStrategy::LineageOnly => Err(LineageRelayError::NoRelayAvailable(
                String::from("LineageOnly strategy does not use STUN discovery"),
            )),
        }
    }

    /// Sovereignty-first strategy: Try tiers sequentially
    ///
    /// 1. User-provided STUN (if configured)
    /// 2. Public STUN (if enabled)
    /// 3. Error if all fail
    ///
    /// Note: Genetic lineage relay doesn't need STUN discovery
    async fn sovereignty_first_discovery(&self) -> Result<SocketAddr> {
        info!("🛡️  Sovereignty-first STUN discovery");

        // Tier 2: User-provided STUN
        if !self.config.user_provided.is_empty() {
            info!("   Tier 2: Trying user-provided STUN servers...");

            for user_stun in &self.config.user_provided {
                if !user_stun.enabled {
                    continue;
                }

                match self.stun_client.discover_public_address(&user_stun.address).await {
                    Ok(addr) => {
                        info!("✅ User-provided STUN successful ({}): {}", user_stun.address, addr);
                        return Ok(addr);
                    }
                    Err(e) => {
                        warn!("⚠️  User-provided STUN failed ({}): {}", user_stun.address, e);
                    }
                }
            }
        }

        // Tier 3: Public STUN (opt-in only)
        if self.config.public_stun.enabled {
            info!("   Tier 3: Trying public STUN servers...");
            warn!("⚠️  Privacy Warning: Public STUN servers can observe your IP");

            for public_server in &self.config.public_stun.servers {
                match self.stun_client.discover_public_address(&public_server.address).await {
                    Ok(addr) => {
                        info!("✅ Public STUN successful ({}): {}", public_server.address, addr);
                        return Ok(addr);
                    }
                    Err(e) => {
                        warn!("⚠️  Public STUN failed ({}): {}", public_server.address, e);
                    }
                }
            }
        } else {
            debug!("   Tier 3: Public STUN disabled (sovereignty-first)");
        }

        Err(LineageRelayError::NoRelayAvailable(String::from("All STUN discovery tiers failed")))
    }

    /// Fastest-first strategy: Try all tiers in parallel
    ///
    /// Attempts all enabled tiers simultaneously and returns first success.
    async fn fastest_first_discovery(&self) -> Result<SocketAddr> {
        info!("⚡ Fastest-first STUN discovery (parallel)");

        let mut tasks = Vec::new();

        // Tier 2: User-provided STUN
        if !self.config.user_provided.is_empty() {
            for user_stun in self.config.user_provided.clone() {
                if !user_stun.enabled {
                    continue;
                }

                let client = Arc::clone(&self.stun_client);
                tasks.push(tokio::spawn(async move {
                    info!("   Tier 2: Trying user-provided STUN ({})...", user_stun.address);
                    client.discover_public_address(&user_stun.address).await
                }));
            }
        }

        // Tier 3: Public STUN (if enabled)
        if self.config.public_stun.enabled {
            for public_server in self.config.public_stun.servers.clone() {
                let client = Arc::clone(&self.stun_client);
                tasks.push(tokio::spawn(async move {
                    info!("   Tier 3: Trying public STUN ({})...", public_server.address);
                    warn!("⚠️  Privacy Warning: Public STUN servers can observe your IP");
                    client.discover_public_address(&public_server.address).await
                }));
            }
        }

        for task in tasks {
            if let Ok(Ok(addr)) = task.await {
                info!("✅ Fastest-first discovery successful: {}", addr);
                return Ok(addr);
            }
        }

        Err(LineageRelayError::NoRelayAvailable(String::from(
            "All parallel STUN discovery attempts failed",
        )))
    }
}
