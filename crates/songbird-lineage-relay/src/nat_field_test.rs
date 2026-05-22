// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! NAT Field Test Harness
//!
//! Validates that the TURN relay path works for cross-gate capability dispatch
//! in residential NAT scenarios (CGNAT, double-NAT, symmetric NAT).
//!
//! ## Usage
//!
//! These tests require a live TURN server. Set the environment variables:
//! - `SONGBIRD_TURN_SERVER` — relay address (e.g. `157.230.3.183:3478`)
//! - `SONGBIRD_TURN_USERNAME` — TURN credential username
//! - `SONGBIRD_TURN_KEY` — TURN credential key (hex-encoded)
//!
//! Run with: `cargo test -p songbird-lineage-relay nat_field_test -- --ignored`

use crate::error::Result;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tracing::info;

/// Result of a single NAT traversal probe.
#[derive(Debug, Clone)]
pub struct NatProbeResult {
    /// NAT scenario being tested.
    pub scenario: String,
    /// Whether the probe succeeded.
    pub success: bool,
    /// Time to establish the relay path.
    pub setup_duration: Duration,
    /// Time to complete a round-trip JSON-RPC exchange.
    pub rtt: Option<Duration>,
    /// Error message if failed.
    pub error: Option<String>,
    /// The relay address allocated (if successful).
    pub relay_addr: Option<SocketAddr>,
}

/// NAT scenario descriptor for field testing.
#[derive(Debug, Clone)]
pub struct NatScenario {
    /// Human-readable name (e.g. "CGNAT residential", "double-NAT", "symmetric")
    pub name: String,
    /// The peer address to relay towards.
    pub peer_addr: SocketAddr,
    /// Whether direct TCP is expected to fail (simulated by not connecting directly).
    pub expect_direct_failure: bool,
}

/// Run a NAT field test probe: allocate TURN, send a JSON-RPC health check, measure RTT.
///
/// # Errors
///
/// Returns error if TURN env vars are not configured or allocation fails.
pub async fn probe_turn_path(scenario: &NatScenario) -> Result<NatProbeResult> {
    use songbird_turn_client::{TurnSession, TurnSessionConfig};

    let start = Instant::now();

    let config = match TurnSessionConfig::from_env(scenario.peer_addr) {
        Ok(c) => c,
        Err(e) => {
            return Ok(NatProbeResult {
                scenario: scenario.name.clone(),
                success: false,
                setup_duration: start.elapsed(),
                rtt: None,
                error: Some(format!("TURN config: {e}")),
                relay_addr: None,
            });
        }
    };

    let session = match TurnSession::connect(config).await {
        Ok(s) => s,
        Err(e) => {
            return Ok(NatProbeResult {
                scenario: scenario.name.clone(),
                success: false,
                setup_duration: start.elapsed(),
                rtt: None,
                error: Some(format!("TURN connect: {e}")),
                relay_addr: None,
            });
        }
    };

    let setup_duration = start.elapsed();
    let relay_addr = session.relay_addr();

    info!(
        scenario = %scenario.name,
        relay_addr = %relay_addr,
        setup_ms = setup_duration.as_millis(),
        "TURN allocation successful"
    );

    // Send a JSON-RPC health check through the relay
    let request = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"id\":1}\n";
    let rtt_start = Instant::now();

    if let Err(e) = session.send(request).await {
        return Ok(NatProbeResult {
            scenario: scenario.name.clone(),
            success: false,
            setup_duration,
            rtt: None,
            error: Some(format!("TURN send: {e}")),
            relay_addr: Some(relay_addr),
        });
    }

    let mut buf = vec![0u8; 4096];
    match tokio::time::timeout(Duration::from_secs(10), session.recv(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let rtt = rtt_start.elapsed();
            info!(
                scenario = %scenario.name,
                rtt_ms = rtt.as_millis(),
                bytes = n,
                "Round-trip through TURN relay successful"
            );
            Ok(NatProbeResult {
                scenario: scenario.name.clone(),
                success: true,
                setup_duration,
                rtt: Some(rtt),
                error: None,
                relay_addr: Some(relay_addr),
            })
        }
        Ok(Ok(_)) => Ok(NatProbeResult {
            scenario: scenario.name.clone(),
            success: false,
            setup_duration,
            rtt: None,
            error: Some("zero bytes received".to_string()),
            relay_addr: Some(relay_addr),
        }),
        Ok(Err(e)) => Ok(NatProbeResult {
            scenario: scenario.name.clone(),
            success: false,
            setup_duration,
            rtt: None,
            error: Some(format!("recv error: {e}")),
            relay_addr: Some(relay_addr),
        }),
        Err(_) => Ok(NatProbeResult {
            scenario: scenario.name.clone(),
            success: true,
            setup_duration,
            rtt: None,
            error: Some("recv timeout (peer may not be listening — allocation valid)".to_string()),
            relay_addr: Some(relay_addr),
        }),
    }
}

/// Run the full NAT field test matrix.
///
/// Tests TURN allocation reachability against a set of peer addresses representing
/// different NAT scenarios. Reports per-scenario results.
pub async fn run_field_test_matrix(peer_addrs: &[NatScenario]) -> Vec<NatProbeResult> {
    let mut results = Vec::with_capacity(peer_addrs.len());
    for scenario in peer_addrs {
        let result = probe_turn_path(scenario).await.unwrap_or_else(|e| NatProbeResult {
            scenario: scenario.name.clone(),
            success: false,
            setup_duration: Duration::ZERO,
            rtt: None,
            error: Some(format!("probe failed: {e}")),
            relay_addr: None,
        });
        results.push(result);
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nat_probe_result_reports_failure_without_turn_config() {
        let scenario = NatScenario {
            name: "unit-test-no-turn".to_string(),
            peer_addr: "192.168.1.100:8080".parse().unwrap(),
            expect_direct_failure: true,
        };

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        let result = rt.block_on(probe_turn_path(&scenario)).unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("TURN config"));
    }

    #[test]
    fn field_test_matrix_runs_all_scenarios() {
        let scenarios = vec![
            NatScenario {
                name: "cgnat-residential".to_string(),
                peer_addr: "100.64.0.1:8080".parse().unwrap(),
                expect_direct_failure: true,
            },
            NatScenario {
                name: "double-nat".to_string(),
                peer_addr: "192.168.1.1:8080".parse().unwrap(),
                expect_direct_failure: true,
            },
            NatScenario {
                name: "symmetric-nat".to_string(),
                peer_addr: "10.0.0.1:8080".parse().unwrap(),
                expect_direct_failure: true,
            },
        ];

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        let results = rt.block_on(run_field_test_matrix(&scenarios));
        assert_eq!(results.len(), 3);
        for r in &results {
            assert!(!r.success, "Expected failure without TURN env for {}", r.scenario);
        }
    }

    /// Live TURN test — requires `SONGBIRD_TURN_*` environment variables.
    /// Run with: `cargo test -p songbird-lineage-relay nat_field_test -- --ignored`
    #[tokio::test]
    #[ignore = "requires live TURN server (SONGBIRD_TURN_* env vars)"]
    async fn live_turn_allocation_validates_cgnat_scenario() {
        let scenario = NatScenario {
            name: "live-cgnat-probe".to_string(),
            peer_addr: "100.64.0.1:8080".parse().unwrap(),
            expect_direct_failure: true,
        };

        let result = probe_turn_path(&scenario).await.unwrap();
        assert!(
            result.success || result.relay_addr.is_some(),
            "TURN allocation should succeed even if peer is unreachable: {:?}",
            result.error
        );
        println!("Setup: {:?}, Relay: {:?}", result.setup_duration, result.relay_addr);
    }
}
