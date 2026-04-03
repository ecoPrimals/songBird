// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use super::super::config::{ExtensionSet, ExtensionType};

/// Profile for a specific server
#[derive(Debug, Clone)]
pub struct ServerProfile {
    /// Server hostname (shared via `Arc` across clones and snapshots)
    pub hostname: Arc<str>,

    /// Last successful extension set
    pub successful_extensions: Vec<ExtensionType>,

    /// Last successful cipher suite
    pub successful_cipher: Option<u16>,

    /// Extensions that caused failures
    pub failed_extensions: Vec<ExtensionType>,

    /// Cipher suites that caused failures
    pub failed_ciphers: Vec<u16>,

    /// Success count
    pub success_count: u32,

    /// Failure count
    pub failure_count: u32,

    /// Average handshake duration (successful)
    pub avg_handshake_duration: Duration,

    /// Last successful connection
    pub last_success: Option<SystemTime>,

    /// Last failure
    pub last_failure: Option<SystemTime>,

    /// Connection reliability (0.0 - 1.0)
    pub reliability: f32,
}

/// Global statistics across all servers
#[derive(Debug, Clone)]
pub struct GlobalStats {
    /// Total successful connections
    pub total_successes: u64,

    /// Total failures
    pub total_failures: u64,

    /// Most successful extension set
    pub best_extension_set: Vec<ExtensionType>,

    /// Most successful cipher suite
    pub best_cipher: Option<u16>,

    /// Extensions that frequently cause failures
    pub problematic_extensions: HashMap<ExtensionType, u32>,
}

impl ServerProfile {
    /// Create new server profile
    pub(super) fn new(hostname: &str) -> Self {
        Self {
            hostname: Arc::from(hostname),
            successful_extensions: ExtensionSet::standard().extensions,
            successful_cipher: None,
            failed_extensions: Vec::new(),
            failed_ciphers: Vec::new(),
            success_count: 0,
            failure_count: 0,
            avg_handshake_duration: Duration::from_secs(0),
            last_success: None,
            last_failure: None,
            reliability: 0.0,
        }
    }

    /// Get success rate (0.0 - 1.0)
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "acceptable precision for metrics/profiling")]
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 0.0;
        }
        self.success_count as f32 / total as f32
    }

    /// Is this server reliable? (>= 80% success rate)
    #[must_use]
    pub fn is_reliable(&self) -> bool {
        self.reliability >= 0.8
    }

    /// Should we retry with different settings?
    #[must_use]
    pub fn should_retry_with_fallback(&self) -> bool {
        // If reliability is low and we have enough data
        let total = self.success_count + self.failure_count;
        total >= 3 && self.reliability < 0.5
    }
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self {
            total_successes: 0,
            total_failures: 0,
            best_extension_set: ExtensionSet::standard().extensions,
            best_cipher: Some(0x1301), // AES-128-GCM (most compatible)
            problematic_extensions: HashMap::new(),
        }
    }
}

impl GlobalStats {
    /// Get overall success rate
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "acceptable precision for metrics/profiling")]
    pub fn success_rate(&self) -> f32 {
        let total = self.total_successes + self.total_failures;
        if total == 0 {
            return 0.0;
        }
        self.total_successes as f32 / total as f32
    }

    /// Get most problematic extensions
    #[must_use]
    pub fn most_problematic_extensions(&self, count: usize) -> Vec<(ExtensionType, u32)> {
        let mut sorted: Vec<_> = self.problematic_extensions.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        sorted.into_iter().take(count).map(|(k, v)| (*k, *v)).collect()
    }
}
