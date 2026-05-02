// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Default timeout values

use std::time::Duration;

/// Default upper bound for completing a single request.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
/// Default time to wait when establishing a connection.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Default idle period before closing or recycling a connection.
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
/// Default interval for health-check polling loops.
pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
/// Default timeout for long-running compute / execution tasks.
pub const DEFAULT_COMPUTE_TIMEOUT: Duration = Duration::from_secs(300);
/// Default cache TTL for capability/session caches.
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);
/// Default SSO / auth token validation timeout.
pub const DEFAULT_AUTH_VALIDATION_TIMEOUT: Duration = Duration::from_secs(10);
/// Default peer/socket peek timeout during protocol auto-detection.
pub const DEFAULT_PEEK_TIMEOUT: Duration = Duration::from_secs(5);
/// Default discovery bridge poll interval.
pub const DEFAULT_DISCOVERY_POLL_INTERVAL: Duration = Duration::from_secs(10);
/// Default graceful shutdown timeout.
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);
