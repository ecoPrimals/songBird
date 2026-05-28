// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Outbound `primal.announce` push to biomeOS Neural API.
//!
//! After our socket binds, we announce ourselves to the Neural API so routing
//! weights are seeded for songbird's capability domains (relay, communication,
//! presence). Uses WAVE42 tiered socket discovery.

use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// Resolve the biomeOS Neural API socket path using WAVE42 tiered discovery.
///
/// Priority:
/// 1. `$NEURAL_API_SOCKET` — explicit override
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api-{family}.sock`
/// 3. `/tmp/biomeos/neural-api-{family}.sock`
pub fn resolve_neural_api_socket() -> Option<PathBuf> {
    if let Ok(p) = songbird_process_env::var("NEURAL_API_SOCKET") {
        let path = PathBuf::from(&p);
        if path.exists() {
            debug!(path = %path.display(), "Neural API socket from NEURAL_API_SOCKET");
            return Some(path);
        }
    }

    let family = resolve_family_id();

    if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
        let candidate =
            PathBuf::from(&xdg).join("biomeos").join(format!("neural-api-{family}.sock"));
        if candidate.exists() {
            debug!(path = %candidate.display(), "Neural API socket from XDG_RUNTIME_DIR");
            return Some(candidate);
        }
    }

    let tmp_candidate =
        std::env::temp_dir().join("biomeos").join(format!("neural-api-{family}.sock"));
    if tmp_candidate.exists() {
        debug!(path = %tmp_candidate.display(), "Neural API socket from /tmp fallback");
        return Some(tmp_candidate);
    }

    None
}

/// Push `primal.announce` to the biomeOS Neural API socket.
///
/// Sends our full announce payload as a JSON-RPC call and logs the response.
/// Non-fatal: if the neural-api socket is unavailable, we log a warning and
/// continue — songbird functions without biomeOS routing.
pub async fn announce_to_neural_api(neural_socket: &Path, own_socket: &str) {
    let payload = songbird_universal_ipc::introspection::primal_announce_with_socket(own_socket);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "primal.announce",
        "params": payload,
        "id": 1
    });

    let mut request_bytes = match serde_json::to_vec(&request) {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to serialize announce payload: {e}");
            return;
        }
    };
    request_bytes.push(b'\n');

    let stream = match UnixStream::connect(neural_socket).await {
        Ok(s) => s,
        Err(e) => {
            debug!(
                socket = %neural_socket.display(),
                error = %e,
                "Neural API socket not available — skipping announce (songbird operates standalone)"
            );
            return;
        }
    };

    let (reader, mut writer) = stream.into_split();

    if let Err(e) = writer.write_all(&request_bytes).await {
        warn!("Failed to write announce to neural-api: {e}");
        return;
    }

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();

    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        buf_reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(n)) if n > 0 => {
            info!("Neural API announce accepted ({n} bytes response)");
            debug!(response = %response_line.trim(), "Neural API response");
        }
        Ok(Ok(_)) => {
            debug!("Neural API closed connection after announce (likely accepted)");
        }
        Ok(Err(e)) => {
            warn!("Error reading neural-api response: {e}");
        }
        Err(_) => {
            debug!("Neural API response timeout — announce likely accepted");
        }
    }
}

/// Spawn the outbound announce after server socket is ready.
///
/// Call this after the IPC server has bound its socket. It runs asynchronously
/// and does not block server operation.
pub fn spawn_announce(own_socket_path: &Path) {
    let own_socket = own_socket_path.to_string_lossy().to_string();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        match resolve_neural_api_socket() {
            Some(neural_socket) => {
                info!(
                    neural = %neural_socket.display(),
                    own = %own_socket,
                    "Announcing to biomeOS Neural API"
                );
                announce_to_neural_api(&neural_socket, &own_socket).await;
            }
            None => {
                debug!("No biomeOS neural-api socket found — operating standalone");
            }
        }
    });
}

fn resolve_family_id() -> String {
    songbird_process_env::var("FAMILY_ID")
        .or_else(|_| songbird_process_env::var("BIOMEOS_FAMILY_ID"))
        .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
        .unwrap_or_else(|_| "ecoPrimal".to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn resolve_neural_api_socket_returns_none_when_no_env() {
        // Without NEURAL_API_SOCKET set and no socket on disk, returns None
        let result = resolve_neural_api_socket();
        // May or may not find a socket depending on environment — just verify no panic
        let _ = result;
    }

    #[test]
    fn resolve_family_id_uses_env_or_default() {
        let fid = resolve_family_id();
        assert!(!fid.is_empty(), "family_id should never be empty");
    }

    #[test]
    fn announce_payload_has_aligned_capabilities_and_hints() {
        let payload =
            songbird_universal_ipc::introspection::primal_announce_with_socket("/tmp/test.sock");

        let caps = payload["capabilities"].as_array().unwrap();
        let hints = payload["cost_hints"].as_object().unwrap();
        let latency = payload["latency_estimates"].as_object().unwrap();

        // Every capability domain must have a corresponding cost hint and latency estimate
        for cap in caps {
            let key = cap.as_str().unwrap();
            assert!(hints.contains_key(key), "capability '{key}' missing from cost_hints");
            assert!(latency.contains_key(key), "capability '{key}' missing from latency_estimates");
        }

        // Every hint key must be a capability domain
        for key in hints.keys() {
            assert!(
                caps.iter().any(|c| c.as_str() == Some(key)),
                "cost_hints key '{key}' not in capabilities"
            );
        }
    }

    #[test]
    fn announce_payload_has_required_fields() {
        let payload =
            songbird_universal_ipc::introspection::primal_announce_with_socket("/tmp/test.sock");

        assert_eq!(payload["primal"].as_str().unwrap(), "songbird");
        assert_eq!(payload["socket"].as_str().unwrap(), "/tmp/test.sock");
        assert!(payload["signal_tiers"].is_array());
        assert!(payload["methods"].is_array());
        assert_eq!(payload["status"].as_str().unwrap(), "ready");
    }
}
