// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::types::EnrollPhase;
use tracing::{info, warn};

/// IP pool range for dynamic gate allocation.
const POOL_START: u8 = 20;
const POOL_END: u8 = 254;

pub(super) fn mesh_subnet() -> String {
    songbird_process_env::var("SONGBIRD_MESH_SUBNET").unwrap_or_else(|_| "10.13.37".into())
}

/// Allocate a mesh IP from the dynamic pool.
///
/// Checks which IPs in the .20–.254 range are already allocated by querying
/// `wg show wg0 allowed-ips` on the local hub.
pub(super) async fn allocate_mesh_ip(gate_name: &str) -> (EnrollPhase, Option<String>) {
    let used_ips = match tokio::process::Command::new("wg")
        .args(["show", "wg0", "allowed-ips"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_used_ips(&stdout)
        }
        Ok(_) | Err(_) => {
            return (
                EnrollPhase {
                    name: "ip.allocate".into(),
                    ok: false,
                    detail: "cannot query wg0 allowed-ips — is WireGuard running?".into(),
                },
                None,
            );
        }
    };

    for octet in POOL_START..=POOL_END {
        let candidate = format!("{}.{octet}", mesh_subnet());
        if !used_ips.contains(&candidate) {
            info!(gate = %gate_name, ip = %candidate, "ip.allocate: assigned from pool");
            return (
                EnrollPhase {
                    name: "ip.allocate".into(),
                    ok: true,
                    detail: format!(
                        "{candidate} allocated for {gate_name} (pool .{POOL_START}–.{POOL_END})"
                    ),
                },
                Some(candidate),
            );
        }
    }

    warn!("ip.allocate: pool exhausted (.{POOL_START}–.{POOL_END})");
    (
        EnrollPhase {
            name: "ip.allocate".into(),
            ok: false,
            detail: format!("pool exhausted — all .{POOL_START}–.{POOL_END} allocated"),
        },
        None,
    )
}

/// Parse IPs from `wg show wg0 allowed-ips` output.
pub(super) fn parse_used_ips(wg_output: &str) -> Vec<String> {
    wg_output
        .lines()
        .flat_map(|line| {
            line.split_whitespace()
                .skip(1)
                .filter_map(|cidr| cidr.split('/').next().map(String::from))
        })
        .collect()
}
