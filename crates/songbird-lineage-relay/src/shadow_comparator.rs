// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shadow Dual-Path Comparator
//!
//! Executes a NAT traversal operation on two paths (e.g. TURN relay vs
//! `cloudflared` emergency tunnel) in parallel and records structured metrics
//! for each. Used during shadow runs to evaluate which relay strategy offers
//! better latency, throughput, and reliability before promoting to production.

use crate::error::Result;
use crate::multi_tier_coordinator::{CloudflaredTunnel, ConnectionTier};
use serde::Serialize;
use songbird_turn_client::TurnSessionConfig;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

/// Metrics collected from a single path probe.
#[derive(Debug, Clone, Serialize)]
pub struct PathMetrics {
    /// Which tier was probed (e.g. "turn-relay", "emergency-tunnel").
    pub tier: String,
    /// Connection setup time in milliseconds.
    pub setup_duration_ms: u64,
    /// Relay address (TURN allocated addr or tunnel URL).
    pub relay_addr: Option<String>,
    /// Whether the probe succeeded.
    pub success: bool,
    /// Error description if the probe failed.
    pub error: Option<String>,
}

/// Combined report from a shadow dual-path comparison.
#[derive(Debug, Clone, Serialize)]
pub struct ShadowComparisonReport {
    /// Target peer address.
    pub peer_addr: String,
    /// UTC timestamp of the comparison run.
    pub timestamp: String,
    /// Metrics from the TURN relay path.
    pub turn_metrics: PathMetrics,
    /// Metrics from the cloudflared tunnel path.
    pub tunnel_metrics: PathMetrics,
    /// Recommended tier based on results.
    pub recommended_tier: String,
}

/// Run a shadow comparison: probe both TURN and cloudflared paths in parallel.
///
/// Neither path carries production traffic — this is a measurement-only run
/// that evaluates connection setup time and availability.
///
/// # Errors
///
/// Returns an error only if both paths fail to yield any metrics. Individual
/// path failures are recorded in the returned report.
pub async fn compare_paths(peer_addr: SocketAddr) -> Result<ShadowComparisonReport> {
    info!("shadow comparator: starting dual-path probe to {peer_addr}");

    let (turn_result, tunnel_result) =
        tokio::join!(probe_turn_path(peer_addr), probe_tunnel_path(),);

    let turn_metrics = turn_result.unwrap_or_else(|e| PathMetrics {
        tier: ConnectionTier::TurnRelay.to_string(),
        setup_duration_ms: 0,
        relay_addr: None,
        success: false,
        error: Some(e.to_string()),
    });

    let tunnel_metrics = tunnel_result.unwrap_or_else(|e| PathMetrics {
        tier: ConnectionTier::EmergencyTunnel.to_string(),
        setup_duration_ms: 0,
        relay_addr: None,
        success: false,
        error: Some(e.to_string()),
    });

    let recommended_tier = recommend_tier(&turn_metrics, &tunnel_metrics);

    let report = ShadowComparisonReport {
        peer_addr: peer_addr.to_string(),
        timestamp: songbird_types::defaults::time::rfc3339_now(),
        turn_metrics,
        tunnel_metrics,
        recommended_tier,
    };

    info!(
        tier = %report.recommended_tier,
        "shadow comparator: recommendation"
    );

    Ok(report)
}

/// Probe the TURN relay path (Tier 4).
async fn probe_turn_path(peer_addr: SocketAddr) -> Result<PathMetrics> {
    let config = TurnSessionConfig::from_env(peer_addr).map_err(|e| {
        crate::error::LineageRelayError::ConfigError(format!("TURN env config: {e}"))
    })?;

    let start = Instant::now();

    let client = songbird_stun::TurnClient::new(config.server_addr, config.credentials.clone())
        .with_timeout(config.control_timeout);

    let socket = UdpSocket::bind(songbird_types::constants::EPHEMERAL_BIND_ADDR)
        .await
        .map_err(|e| crate::error::LineageRelayError::NetworkError(format!("bind: {e}")))?;

    let allocation = client.allocate(&socket).await.map_err(|e| {
        crate::error::LineageRelayError::NetworkError(format!("TURN allocate: {e}"))
    })?;

    let setup_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    debug!("shadow: TURN allocation in {setup_ms}ms → {}", allocation.relay_addr);

    Ok(PathMetrics {
        tier: ConnectionTier::TurnRelay.to_string(),
        setup_duration_ms: setup_ms,
        relay_addr: Some(allocation.relay_addr.to_string()),
        success: true,
        error: None,
    })
}

/// Probe the cloudflared tunnel path (Tier 5).
async fn probe_tunnel_path() -> Result<PathMetrics> {
    let start = Instant::now();

    match CloudflaredTunnel::spawn(7845).await {
        Ok(mut tunnel) => {
            let setup_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            let endpoint = tunnel.endpoint().to_string();
            debug!("shadow: cloudflared tunnel in {setup_ms}ms → {endpoint}");
            tunnel.shutdown().await;

            Ok(PathMetrics {
                tier: ConnectionTier::EmergencyTunnel.to_string(),
                setup_duration_ms: setup_ms,
                relay_addr: Some(endpoint),
                success: true,
                error: None,
            })
        }
        Err(e) => {
            let setup_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            warn!("shadow: cloudflared probe failed in {setup_ms}ms: {e}");

            Ok(PathMetrics {
                tier: ConnectionTier::EmergencyTunnel.to_string(),
                setup_duration_ms: setup_ms,
                relay_addr: None,
                success: false,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Recommend the better tier based on metrics.
fn recommend_tier(turn: &PathMetrics, tunnel: &PathMetrics) -> String {
    match (turn.success, tunnel.success) {
        (true, true) => {
            if turn.setup_duration_ms <= tunnel.setup_duration_ms {
                ConnectionTier::TurnRelay.to_string()
            } else {
                ConnectionTier::EmergencyTunnel.to_string()
            }
        }
        (true, false) => ConnectionTier::TurnRelay.to_string(),
        (false, true) => ConnectionTier::EmergencyTunnel.to_string(),
        (false, false) => String::from("none"),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn recommend_tier_prefers_faster_when_both_succeed() {
        let turn = PathMetrics {
            tier: String::from("turn-relay"),
            setup_duration_ms: 50,
            relay_addr: Some(String::from("1.2.3.4:5")),
            success: true,
            error: None,
        };
        let tunnel = PathMetrics {
            tier: String::from("emergency-tunnel"),
            setup_duration_ms: 200,
            relay_addr: Some(String::from("https://foo.trycloudflare.com")),
            success: true,
            error: None,
        };
        // TURN is faster (50ms < 200ms) → prefer turn-relay
        assert_eq!(recommend_tier(&turn, &tunnel), "turn-relay");
        // When TURN is slower (200ms > 50ms) → prefer emergency-tunnel
        assert_eq!(recommend_tier(&tunnel, &turn), "emergency-tunnel");
    }

    #[test]
    fn recommend_tier_picks_available_path() {
        let ok = PathMetrics {
            tier: String::from("turn-relay"),
            setup_duration_ms: 100,
            relay_addr: None,
            success: true,
            error: None,
        };
        let fail = PathMetrics {
            tier: String::from("emergency-tunnel"),
            setup_duration_ms: 0,
            relay_addr: None,
            success: false,
            error: Some(String::from("not found")),
        };
        assert_eq!(recommend_tier(&ok, &fail), "turn-relay");
        assert_eq!(recommend_tier(&fail, &ok), "emergency-tunnel");
    }

    #[test]
    fn recommend_tier_none_when_both_fail() {
        let fail = PathMetrics {
            tier: String::from("x"),
            setup_duration_ms: 0,
            relay_addr: None,
            success: false,
            error: Some(String::from("err")),
        };
        assert_eq!(recommend_tier(&fail, &fail), "none");
    }
}
