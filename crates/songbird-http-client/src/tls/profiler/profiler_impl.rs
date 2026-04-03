// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::{Duration, SystemTime};

use super::super::config::{ExtensionSet, ExtensionType};
use super::types::{GlobalStats, ServerProfile};

/// Server profile database (thread-safe, persistent)
#[derive(Debug, Clone)]
pub struct ServerProfiler {
    /// Profiles indexed by hostname
    profiles: Arc<RwLock<HashMap<String, ServerProfile>>>,

    /// Global success/failure statistics
    stats: Arc<RwLock<GlobalStats>>,
}

impl Default for ServerProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerProfiler {
    /// Create new profiler
    #[must_use]
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(GlobalStats::default())),
        }
    }

    /// Get profile for a server (or create new)
    #[must_use]
    pub fn get_profile(&self, hostname: &str) -> Option<ServerProfile> {
        let profiles = self.profiles.read().unwrap_or_else(PoisonError::into_inner);
        profiles.get(hostname).cloned()
    }

    /// Record successful connection
    #[expect(clippy::cast_precision_loss, reason = "acceptable precision for metrics/profiling")]
    #[expect(
        clippy::significant_drop_tightening,
        reason = "write lock held for coherent ServerProfile update"
    )]
    pub fn record_success(
        &self,
        hostname: &str,
        extensions: Vec<ExtensionType>,
        cipher: u16,
        handshake_duration: Duration,
    ) {
        let ext_len = extensions.len();
        let promote_extensions = {
            let stats = self.stats.read().unwrap_or_else(PoisonError::into_inner);
            ext_len >= stats.best_extension_set.len()
        };

        let promoted_snapshot = if promote_extensions {
            Some(extensions.clone())
        } else {
            None
        };

        // Update server profile
        {
            let mut profiles = self.profiles.write().unwrap_or_else(PoisonError::into_inner);
            let profile = profiles
                .entry(hostname.to_string())
                .or_insert_with(|| ServerProfile::new(hostname));

            profile.success_count += 1;
            profile.successful_extensions = extensions;
            profile.successful_cipher = Some(cipher);
            profile.last_success = Some(SystemTime::now());

            // Update average handshake duration (exponential moving average)
            let alpha = 0.2; // Smoothing factor
            let new_duration_secs = handshake_duration.as_secs_f32();
            let old_duration_secs = profile.avg_handshake_duration.as_secs_f32();
            let avg_duration_secs = alpha * new_duration_secs + (1.0 - alpha) * old_duration_secs;
            profile.avg_handshake_duration = Duration::from_secs_f32(avg_duration_secs);

            // Update reliability
            let total = profile.success_count + profile.failure_count;
            profile.reliability = profile.success_count as f32 / total as f32;
        }

        // Update global stats
        {
            let mut stats = self.stats.write().unwrap_or_else(PoisonError::into_inner);
            stats.total_successes += 1;

            if let Some(ext) = promoted_snapshot {
                stats.best_extension_set = ext;
            }

            stats.best_cipher = Some(cipher);
        }
    }

    /// Record failed connection
    #[expect(clippy::cast_precision_loss, reason = "acceptable precision for metrics/profiling")]
    #[expect(
        clippy::significant_drop_tightening,
        reason = "write lock held for coherent ServerProfile update"
    )]
    pub fn record_failure(
        &self,
        hostname: &str,
        extensions: Vec<ExtensionType>,
        cipher: Option<u16>,
        _error_message: &str,
    ) {
        // Update server profile
        {
            let mut profiles = self.profiles.write().unwrap_or_else(PoisonError::into_inner);
            let profile = profiles
                .entry(hostname.to_string())
                .or_insert_with(|| ServerProfile::new(hostname));

            profile.failure_count += 1;
            profile.last_failure = Some(SystemTime::now());

            // Track which extensions might have caused failure
            for ext in &extensions {
                if !profile.successful_extensions.contains(ext)
                    && !profile.failed_extensions.contains(ext)
                {
                    profile.failed_extensions.push(*ext);
                }
            }

            // Track cipher failure
            if let Some(cipher_suite) = cipher
                && !profile.failed_ciphers.contains(&cipher_suite)
            {
                profile.failed_ciphers.push(cipher_suite);
            }

            // Update reliability
            let total = profile.success_count + profile.failure_count;
            if total > 0 {
                profile.reliability = profile.success_count as f32 / total as f32;
            }
        }

        // Update global stats
        {
            let mut stats = self.stats.write().unwrap_or_else(PoisonError::into_inner);
            stats.total_failures += 1;

            // Track problematic extensions
            for ext in extensions {
                *stats.problematic_extensions.entry(ext).or_insert(0) += 1;
            }
        }
    }

    /// Get recommended extension set for a server
    #[must_use]
    pub fn recommend_extensions(&self, hostname: &str) -> Vec<ExtensionType> {
        // Check if we have a profile for this server
        if let Some(profile) = self.get_profile(hostname)
            && profile.success_count > 0
        {
            // Use last successful extension set
            return profile.successful_extensions;
        }

        // Fall back to global best extension set
        let best_extensions =
            self.stats.read().unwrap_or_else(PoisonError::into_inner).best_extension_set.clone();
        if !best_extensions.is_empty() {
            return best_extensions;
        }

        // Fall back to standard set
        ExtensionSet::standard().extensions
    }

    /// Get recommended cipher suite for a server
    #[must_use]
    pub fn recommend_cipher(&self, hostname: &str) -> Option<u16> {
        // Check server profile
        if let Some(profile) = self.get_profile(hostname)
            && let Some(cipher) = profile.successful_cipher
        {
            return Some(cipher);
        }

        // Fall back to global best cipher
        self.stats.read().unwrap_or_else(PoisonError::into_inner).best_cipher
    }

    /// Get global statistics
    #[must_use]
    pub fn get_stats(&self) -> GlobalStats {
        self.stats.read().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// Get all profiles (for debugging/analysis)
    #[must_use]
    pub fn get_all_profiles(&self) -> HashMap<String, ServerProfile> {
        self.profiles.read().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// Clear all profiles (reset learning)
    pub fn clear(&self) {
        self.profiles.write().unwrap_or_else(PoisonError::into_inner).clear();
        *self.stats.write().unwrap_or_else(PoisonError::into_inner) = GlobalStats::default();
    }

    /// Get profile count
    #[must_use]
    pub fn profile_count(&self) -> usize {
        self.profiles.read().unwrap_or_else(PoisonError::into_inner).len()
    }
}
