// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Periodic checkpoint cleanup.

use super::super::TaskStorageBackend;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

pub fn spawn_checkpoint_cleanup_task(
    storage: Arc<dyn TaskStorageBackend>,
    interval: Duration,
    max_age_seconds: u64,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;

            debug!("Running checkpoint cleanup");

            match storage.cleanup_old_checkpoints(max_age_seconds).await {
                Ok(count) => {
                    if count > 0 {
                        info!("Cleaned up {} old checkpoints", count);
                    }
                }
                Err(e) => {
                    warn!("Checkpoint cleanup failed: {}", e);
                }
            }
        }
    });

    debug!("Background cleanup task started");
}
