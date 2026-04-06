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
