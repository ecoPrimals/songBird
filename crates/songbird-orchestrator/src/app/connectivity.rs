// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Post-startup connectivity verification and auto-remediation
//!
//! Tests whether the HTTPS server is reachable from external IPs and provides
//! diagnostics and auto-remediation for common issues (firewall, VLANs, TLS).

use anyhow::Result;
use songbird_types::SafeEnv;
use tracing::{info, warn};

use super::network::get_local_ip_for_connectivity_test;
use crate::network::{ConnectivityRemediator, ConnectivityTester};

/// Parse `ip:port` for connectivity checks (pure; used by [`verify_external_connectivity`]).
///
/// # Errors
///
/// Returns an error if `ip` and `port` do not form a valid [`std::net::SocketAddr`] when combined.
#[allow(dead_code, reason = "used by upcoming connectivity test expansion")]
pub(crate) fn parse_connectivity_socket_addr(ip: &str, port: u16) -> Result<std::net::SocketAddr> {
    format!("{ip}:{port}")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse socket address: {e}"))
}

/// Verify external connectivity after startup.
///
/// Tests HTTPS reachability, provides diagnostics, and attempts auto-remediation.
/// Connectivity failure is non-fatal — the orchestrator continues regardless.
pub(crate) async fn verify_external_connectivity() -> Result<()> {
    info!("🔍 Verifying external connectivity...");

    let port =
        SafeEnv::get_port("SONGBIRD_PORT", songbird_config::defaults::ports::orchestrator_port());

    let local_ip = match get_local_ip_for_connectivity_test().await {
        Ok(ip) => ip,
        Err(e) => {
            warn!("⚠️  Could not determine local IP for connectivity test: {}", e);
            warn!("   Skipping external connectivity verification");
            return Ok(());
        }
    };

    let target: std::net::SocketAddr = format!("{local_ip}:{port}")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse socket address: {e}"))?;

    let tester = ConnectivityTester::new();
    let result = tester.test_comprehensive(target).await?;

    if result.https_reachable {
        info!("✅ External connectivity verified: https://{}", target);
        if let Some(rtt) = result.rtt_ms {
            info!("   Round-trip time: {}ms", rtt);
        }
        return Ok(());
    }

    warn!("⚠️  External connectivity test failed for https://{}", target);
    warn!("   This may prevent federation with other towers");

    let diagnostics = tester.diagnose_connectivity_issues(target).await;
    for diagnostic in &diagnostics {
        warn!("   {}", diagnostic);
    }

    warn!("🔧 Attempting auto-remediation...");
    match ConnectivityRemediator::attempt_remediation(target).await {
        Ok(actions) => {
            for action in actions {
                warn!("   {}", action);
            }

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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn parse_connectivity_socket_addr_accepts_loopback() {
        let addr = parse_connectivity_socket_addr("127.0.0.1", 8443).unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:8443");
    }

    #[test]
    fn parse_connectivity_socket_addr_rejects_invalid_ip_token() {
        let err = parse_connectivity_socket_addr("not-an-ip", 1).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Failed to parse socket address"), "unexpected message: {msg}");
    }
}
