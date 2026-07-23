// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Stale peer pruning for the mesh handler.
//!
//! Handles `mesh.prune_stale` — removes legacy/dead peers from the mesh
//! that fail health probes or have not been seen within the staleness window.

use serde_json::{Value, json};
use std::time::Duration;
use tracing::{info, warn};

use super::MeshHandler;

/// Default staleness threshold: peers unseen for this long are prunable.
const DEFAULT_STALE_THRESHOLD_SECS: u64 = 300;

impl MeshHandler {
    /// Handle `mesh.prune_stale` — remove dead/legacy peers from the mesh.
    ///
    /// Params (all optional):
    /// - `node_ids`: array of specific peer IDs to prune (skips threshold check)
    /// - `threshold_secs`: max age in seconds before a peer is considered stale (default 300)
    /// - `dry_run`: if true, report what would be pruned without removing (default false)
    ///
    /// Returns list of pruned peer IDs and count.
    pub async fn handle_prune_stale(&self, params: Value) -> Result<Value, String> {
        let dry_run = params.get("dry_run").and_then(Value::as_bool).unwrap_or(false);
        let threshold_secs = params
            .get("threshold_secs")
            .and_then(Value::as_u64)
            .unwrap_or(DEFAULT_STALE_THRESHOLD_SECS);
        let threshold = Duration::from_secs(threshold_secs);

        let mesh = self
            .mesh
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let explicit_ids: Option<Vec<String>> = params.get("node_ids").and_then(|v| {
            v.as_array().map(|arr| arr.iter().filter_map(Value::as_str).map(String::from).collect())
        });

        let mut pruned = Vec::new();
        let mut skipped = Vec::new();

        let candidates: Vec<String> = if let Some(ids) = explicit_ids {
            ids
        } else {
            let known = mesh.get_known_nodes().await;
            let mut stale = Vec::new();
            for node_id in &known {
                let is_stale = match mesh.get_best_path(node_id).await {
                    Some(path) => path.last_seen.elapsed() > threshold,
                    None => true,
                };
                if is_stale {
                    stale.push(node_id.clone());
                }
            }
            stale
        };

        for node_id in &candidates {
            let removed = dry_run || mesh.remove_peer(node_id).await;
            if removed {
                pruned.push(node_id.clone());
            } else {
                skipped.push(node_id.clone());
            }
        }

        // Also clean peer_capabilities and peer_metadata for pruned peers
        if !dry_run && !pruned.is_empty() {
            {
                let mut caps = self.peer_capabilities.write().await;
                for id in &pruned {
                    caps.remove(id);
                }
            }
            {
                let mut meta = self.peer_metadata.write().await;
                for id in &pruned {
                    meta.remove(id);
                }
            }
            info!(count = pruned.len(), peers = ?pruned, "Pruned stale peers from mesh");
        }

        if !skipped.is_empty() {
            warn!(skipped = ?skipped, "Some requested peers not found in mesh");
        }

        Ok(json!({
            "pruned": pruned,
            "count": pruned.len(),
            "dry_run": dry_run,
            "threshold_secs": threshold_secs,
            "skipped": skipped
        }))
    }
}
