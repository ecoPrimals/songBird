// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::enrollment_crypto::{is_valid_mesh_ip, is_valid_wg_pubkey};
use super::types::EnrollPhase;
use tracing::info;

/// Register a `WireGuard` peer on the local hub.
pub(super) async fn register_wg_peer(
    gate_name: &str,
    wg_pubkey: &str,
    mesh_ip: &str,
) -> EnrollPhase {
    if !is_valid_wg_pubkey(wg_pubkey) {
        return EnrollPhase {
            name: "wg.peer".into(),
            ok: false,
            detail: format!(
                "invalid WireGuard public key format: {}",
                &wg_pubkey[..wg_pubkey.len().min(8)]
            ),
        };
    }
    if !is_valid_mesh_ip(mesh_ip) {
        return EnrollPhase {
            name: "wg.peer".into(),
            ok: false,
            detail: format!("invalid mesh IP format: {mesh_ip}"),
        };
    }

    let wg_interface = songbird_process_env::var("WG_INTERFACE").unwrap_or_else(|_| "wg0".into());

    match tokio::process::Command::new("wg")
        .args(["set", &wg_interface, "peer", wg_pubkey, "allowed-ips", &format!("{mesh_ip}/32")])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let save_result = tokio::process::Command::new("wg-quick")
                .args(["save", &wg_interface])
                .output()
                .await;
            let save_ok = save_result.as_ref().is_ok_and(|o| o.status.success());
            info!(gate = %gate_name, ip = %mesh_ip, save_ok, "wg.peer: registered on hub");
            EnrollPhase {
                name: "wg.peer".into(),
                ok: true,
                detail: format!(
                    "peer {gate_name} ({mesh_ip}) registered on hub {wg_interface}{}",
                    if save_ok {
                        ""
                    } else {
                        " (save failed — runtime-only)"
                    }
                ),
            }
        }
        Ok(output) => EnrollPhase {
            name: "wg.peer".into(),
            ok: false,
            detail: format!(
                "wg set failed (exit {}): {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        },
        Err(e) => EnrollPhase {
            name: "wg.peer".into(),
            ok: false,
            detail: format!("wg command failed: {e}"),
        },
    }
}
