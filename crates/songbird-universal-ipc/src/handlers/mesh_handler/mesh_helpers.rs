// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pure utility functions for mesh operations — peer list parsing, stale gate detection.

use serde_json::Value;
use std::net::SocketAddr;

/// Parse a peer list from a JSON array field.
///
/// Supports two formats:
/// - Object: `{"node_id": "...", "address": "host:port"}`
/// - String: `"node_id@host:port"` or `"host:port"` (auto-named)
///
/// If `require_node_id` is true, bare `"host:port"` strings without `@` are skipped.
pub fn parse_peer_list(
    params: &Value,
    keys: &[&str],
    require_node_id: bool,
) -> Vec<(String, SocketAddr)> {
    let arr = keys.iter().find_map(|k| params.get(*k).and_then(Value::as_array));
    let Some(arr) = arr else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|entry| {
            if let Some(obj_id) = entry.get("node_id").and_then(Value::as_str) {
                let addr_str = entry.get("address")?.as_str()?;
                let addr: SocketAddr = addr_str.parse().ok()?;
                return Some((obj_id.to_string(), addr));
            }
            let s = entry.as_str()?;
            if let Some((nid, addr_part)) = s.split_once('@') {
                let addr: SocketAddr = addr_part.parse().ok()?;
                Some((nid.to_string(), addr))
            } else if require_node_id {
                None
            } else {
                let addr: SocketAddr = s.parse().ok()?;
                Some((format!("peer-{}", addr.ip()), addr))
            }
        })
        .collect()
}

/// Scan `wateringHole/heads/*.toml` for gate head files older than 24 hours.
///
/// Returns a list of `{ gate, age_hours, file }` entries for stale peers.
/// Used by `mesh.status` to enrich the response with convergence health.
pub fn detect_stale_gate_heads() -> Vec<serde_json::Value> {
    const STALE_THRESHOLD_SECS: u64 = 24 * 3600;

    let workspace =
        std::env::var("ECOPRIMALS_ROOT").unwrap_or_else(|_| String::from("/opt/ecoPrimals"));
    let heads_dir =
        std::path::PathBuf::from(&workspace).join("infra").join("wateringHole").join("heads");

    let Ok(entries) = std::fs::read_dir(&heads_dir) else {
        return Vec::new();
    };

    let now = std::time::SystemTime::now();
    let mut stale = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let gate = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();

        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let age = now.duration_since(modified).unwrap_or_default();
        if age.as_secs() > STALE_THRESHOLD_SECS {
            let age_hours = age.as_secs() / 3600;
            stale.push(serde_json::json!({
                "gate": gate,
                "age_hours": age_hours,
                "file": path.display().to_string(),
            }));
        }
    }

    stale
}
