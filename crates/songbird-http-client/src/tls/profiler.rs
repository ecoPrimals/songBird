// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Server Profiling System
//!
//! Learns from successful and failed TLS connections to optimize future handshakes.
//! Profiles are persisted and shared across connections for continuous improvement.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use super::config::{ExtensionSet, ExtensionType};

/// Server profile database (thread-safe, persistent)
#[derive(Debug, Clone)]
pub struct ServerProfiler {
    /// Profiles indexed by hostname
    profiles: Arc<RwLock<HashMap<String, ServerProfile>>>,

    /// Global success/failure statistics
    stats: Arc<RwLock<GlobalStats>>,
}

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
    ///
    /// # Panics
    ///
    /// Panics if the profile lock is poisoned (bug condition).
    #[must_use]
    pub fn get_profile(&self, hostname: &str) -> Option<ServerProfile> {
        let profiles = self
            .profiles
            .read()
            .expect("BUG: TLS profiler lock poisoned - indicates panic during profile update");
        profiles.get(hostname).cloned()
    }

    /// Record successful connection
    ///
    /// # Panics
    ///
    /// Panics if a profiler lock is poisoned (bug condition).
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
            let stats = self.stats.read().expect(
                "BUG: TLS profiler stats lock poisoned - indicates panic during stats update",
            );
            ext_len >= stats.best_extension_set.len()
        };

        let promoted_snapshot = if promote_extensions {
            Some(extensions.clone())
        } else {
            None
        };

        // Update server profile
        {
            let mut profiles = self
                .profiles
                .write()
                .expect("BUG: TLS profiler lock poisoned - indicates panic during profile update");
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
            let mut stats = self.stats.write().expect(
                "BUG: TLS profiler stats lock poisoned - indicates panic during stats update",
            );
            stats.total_successes += 1;

            if let Some(ext) = promoted_snapshot {
                stats.best_extension_set = ext;
            }

            stats.best_cipher = Some(cipher);
        }
    }

    /// Record failed connection
    ///
    /// # Panics
    ///
    /// Panics if a profiler lock is poisoned (bug condition).
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
            let mut profiles = self
                .profiles
                .write()
                .expect("BUG: TLS profiler lock poisoned - indicates panic during profile update");
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
            let mut stats = self.stats.write().expect(
                "BUG: TLS profiler stats lock poisoned - indicates panic during stats update",
            );
            stats.total_failures += 1;

            // Track problematic extensions
            for ext in extensions {
                *stats.problematic_extensions.entry(ext).or_insert(0) += 1;
            }
        }
    }

    /// Get recommended extension set for a server
    ///
    /// # Panics
    ///
    /// Panics if a profiler lock is poisoned (bug condition).
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
        let best_extensions = self
            .stats
            .read()
            .expect("BUG: TLS profiler stats lock poisoned - indicates panic during stats update")
            .best_extension_set
            .clone();
        if !best_extensions.is_empty() {
            return best_extensions;
        }

        // Fall back to standard set
        ExtensionSet::standard().extensions
    }

    /// Get recommended cipher suite for a server
    ///
    /// # Panics
    ///
    /// Panics if a profiler lock is poisoned (bug condition).
    #[must_use]
    pub fn recommend_cipher(&self, hostname: &str) -> Option<u16> {
        // Check server profile
        if let Some(profile) = self.get_profile(hostname)
            && let Some(cipher) = profile.successful_cipher
        {
            return Some(cipher);
        }

        // Fall back to global best cipher
        self.stats
            .read()
            .expect("BUG: TLS profiler stats lock poisoned - indicates panic during stats update")
            .best_cipher
    }

    /// Get global statistics
    ///
    /// # Panics
    ///
    /// Panics if the stats lock is poisoned (bug condition).
    #[must_use]
    pub fn get_stats(&self) -> GlobalStats {
        self.stats
            .read()
            .expect("BUG: TLS profiler stats lock poisoned - indicates panic during stats update")
            .clone()
    }

    /// Get all profiles (for debugging/analysis)
    ///
    /// # Panics
    ///
    /// Panics if the profile lock is poisoned (bug condition).
    #[must_use]
    pub fn get_all_profiles(&self) -> HashMap<String, ServerProfile> {
        self.profiles
            .read()
            .expect("BUG: TLS profiler lock poisoned - indicates panic during profile update")
            .clone()
    }

    /// Clear all profiles (reset learning)
    ///
    /// # Panics
    ///
    /// Panics if a profiler lock is poisoned (bug condition).
    pub fn clear(&self) {
        self.profiles
            .write()
            .expect("BUG: TLS profiler lock poisoned - indicates panic during profile update")
            .clear();
        *self.stats.write().expect(
            "BUG: TLS profiler stats lock poisoned - indicates panic during stats update",
        ) = GlobalStats::default();
    }

    /// Get profile count
    ///
    /// # Panics
    ///
    /// Panics if the profile lock is poisoned (bug condition).
    #[must_use]
    pub fn profile_count(&self) -> usize {
        self.profiles
            .read()
            .expect("BUG: TLS profiler lock poisoned - indicates panic during profile update")
            .len()
    }
}

impl ServerProfile {
    /// Create new server profile
    fn new(hostname: &str) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = ServerProfiler::new();
        assert_eq!(profiler.profile_count(), 0);

        let stats = profiler.get_stats();
        assert_eq!(stats.total_successes, 0);
        assert_eq!(stats.total_failures, 0);
    }

    #[test]
    fn test_record_success() {
        let profiler = ServerProfiler::new();
        let hostname = "www.example.com";
        let extensions = ExtensionSet::standard().extensions;
        let cipher = 0x1301;

        profiler.record_success(hostname, extensions, cipher, Duration::from_secs(1));

        let profile = profiler.get_profile(hostname).unwrap();
        assert_eq!(profile.success_count, 1);
        assert_eq!(profile.failure_count, 0);
        assert_eq!(profile.successful_cipher, Some(cipher));
        assert!((profile.reliability - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_record_failure() {
        let profiler = ServerProfiler::new();
        let hostname = "www.example.com";
        let extensions = ExtensionSet::modern().extensions;

        profiler.record_failure(hostname, extensions, Some(0x1303), "connection refused");

        let profile = profiler.get_profile(hostname).unwrap();
        assert_eq!(profile.failure_count, 1);
        assert!(profile.last_failure.is_some());
    }

    #[test]
    fn test_reliability_calculation() {
        let profiler = ServerProfiler::new();
        let hostname = "www.example.com";
        let extensions = ExtensionSet::standard().extensions;

        // 8 successes, 2 failures = 80% reliability
        for _ in 0..8 {
            profiler.record_success(hostname, extensions.clone(), 0x1301, Duration::from_secs(1));
        }
        for _ in 0..2 {
            profiler.record_failure(hostname, extensions.clone(), Some(0x1301), "timeout");
        }

        let profile = profiler.get_profile(hostname).unwrap();
        assert_eq!(profile.success_count, 8);
        assert_eq!(profile.failure_count, 2);
        assert!((profile.reliability - 0.8).abs() < 1e-5);
        assert!(profile.is_reliable());
    }

    #[test]
    fn test_recommendations() {
        let profiler = ServerProfiler::new();
        let hostname = "www.example.com";
        let extensions =
            vec![ExtensionType::Sni, ExtensionType::Alpn, ExtensionType::SupportedVersions];
        let expected = extensions.clone();

        profiler.record_success(hostname, extensions, 0x1303, Duration::from_secs(1));

        let recommended = profiler.recommend_extensions(hostname);
        assert_eq!(recommended, expected);

        let recommended_cipher = profiler.recommend_cipher(hostname);
        assert_eq!(recommended_cipher, Some(0x1303));
    }

    #[test]
    fn test_global_stats() {
        let profiler = ServerProfiler::new();

        // Record multiple connections
        profiler.record_success(
            "server1.com",
            ExtensionSet::standard().extensions,
            0x1301,
            Duration::from_secs(1),
        );
        profiler.record_success(
            "server2.com",
            ExtensionSet::standard().extensions,
            0x1301,
            Duration::from_secs(1),
        );
        profiler.record_failure(
            "server3.com",
            ExtensionSet::modern().extensions,
            Some(0x1303),
            "refused",
        );

        let stats = profiler.get_stats();
        assert_eq!(stats.total_successes, 2);
        assert_eq!(stats.total_failures, 1);
        assert!((stats.success_rate() - (2.0_f32 / 3.0)).abs() < 1e-5);
    }

    #[test]
    fn test_clear_profiles() {
        let profiler = ServerProfiler::new();

        profiler.record_success(
            "test.com",
            ExtensionSet::standard().extensions,
            0x1301,
            Duration::from_secs(1),
        );
        assert_eq!(profiler.profile_count(), 1);

        profiler.clear();
        assert_eq!(profiler.profile_count(), 0);

        let stats = profiler.get_stats();
        assert_eq!(stats.total_successes, 0);
    }

    #[test]
    fn server_profile_success_rate_zero_total() {
        let profile = ServerProfile::new("empty.com");
        assert!((profile.success_rate() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn server_profile_success_rate_all_success() {
        let mut profile = ServerProfile::new("good.com");
        profile.success_count = 10;
        profile.failure_count = 0;
        assert!((profile.success_rate() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn server_profile_success_rate_mixed() {
        let mut profile = ServerProfile::new("mixed.com");
        profile.success_count = 3;
        profile.failure_count = 7;
        assert!((profile.success_rate() - 0.3).abs() < 1e-5);
    }

    #[test]
    fn should_retry_with_fallback_insufficient_data() {
        let mut profile = ServerProfile::new("new.com");
        profile.success_count = 0;
        profile.failure_count = 2;
        profile.reliability = 0.0;
        assert!(!profile.should_retry_with_fallback(), "total < 3 → no retry");
    }

    #[test]
    fn should_retry_with_fallback_low_reliability() {
        let mut profile = ServerProfile::new("bad.com");
        profile.success_count = 1;
        profile.failure_count = 4;
        profile.reliability = 0.2;
        assert!(profile.should_retry_with_fallback());
    }

    #[test]
    fn should_retry_with_fallback_high_reliability() {
        let mut profile = ServerProfile::new("good.com");
        profile.success_count = 9;
        profile.failure_count = 1;
        profile.reliability = 0.9;
        assert!(!profile.should_retry_with_fallback());
    }

    #[test]
    fn should_retry_with_fallback_boundary() {
        let mut profile = ServerProfile::new("edge.com");
        profile.success_count = 1;
        profile.failure_count = 2;
        profile.reliability = 0.5;
        assert!(!profile.should_retry_with_fallback(), "reliability == 0.5 → no (< 0.5 required)");

        profile.reliability = 0.499;
        assert!(profile.should_retry_with_fallback());
    }

    #[test]
    fn global_stats_success_rate_zero_total() {
        let stats = GlobalStats::default();
        assert!((stats.success_rate() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn most_problematic_extensions_empty() {
        let stats = GlobalStats::default();
        let result = stats.most_problematic_extensions(5);
        assert!(result.is_empty());
    }

    #[test]
    fn most_problematic_extensions_sorted_and_truncated() {
        let mut stats = GlobalStats::default();
        stats.problematic_extensions.insert(ExtensionType::Sni, 3);
        stats.problematic_extensions.insert(ExtensionType::Alpn, 10);
        stats.problematic_extensions.insert(ExtensionType::SupportedVersions, 7);

        let top2 = stats.most_problematic_extensions(2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0], (ExtensionType::Alpn, 10));
        assert_eq!(top2[1], (ExtensionType::SupportedVersions, 7));
    }

    #[test]
    fn most_problematic_extensions_count_exceeds_entries() {
        let mut stats = GlobalStats::default();
        stats.problematic_extensions.insert(ExtensionType::Sni, 1);
        let result = stats.most_problematic_extensions(100);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn get_all_profiles_returns_snapshot() {
        let profiler = ServerProfiler::new();
        profiler.record_success(
            "a.com",
            ExtensionSet::standard().extensions,
            0x1301,
            Duration::from_secs(1),
        );
        profiler.record_success("b.com", vec![ExtensionType::Sni], 0x1302, Duration::from_secs(2));

        let all = profiler.get_all_profiles();
        assert_eq!(all.len(), 2);
        assert!(all.contains_key("a.com"));
        assert!(all.contains_key("b.com"));
        assert_eq!(all["a.com"].success_count, 1);
    }

    #[test]
    fn is_reliable_boundary() {
        let mut profile = ServerProfile::new("test.com");
        profile.reliability = 0.79;
        assert!(!profile.is_reliable());

        profile.reliability = 0.80;
        assert!(profile.is_reliable());
    }

    #[test]
    fn global_stats_default_has_standard_extensions() {
        let stats = GlobalStats::default();
        assert!(!stats.best_extension_set.is_empty());
        assert_eq!(stats.best_cipher, Some(0x1301));
    }

    #[test]
    fn record_success_promotes_extension_set() {
        let profiler = ServerProfiler::new();
        let small_set = vec![ExtensionType::Sni];
        let large_set = vec![
            ExtensionType::Sni,
            ExtensionType::Alpn,
            ExtensionType::SupportedVersions,
            ExtensionType::KeyShare,
            ExtensionType::SignatureAlgorithms,
            ExtensionType::PskKeyExchangeModes,
            ExtensionType::SupportedGroups,
            ExtensionType::SessionTicket,
            ExtensionType::StatusRequest,
            ExtensionType::Sct,
        ];

        profiler.record_success("a.com", small_set, 0x1301, Duration::from_secs(1));
        let stats_after_small = profiler.get_stats();

        profiler.record_success("b.com", large_set.clone(), 0x1302, Duration::from_secs(1));
        let stats_after_large = profiler.get_stats();

        assert!(
            stats_after_large.best_extension_set.len()
                >= stats_after_small.best_extension_set.len(),
            "larger successful extension set should be promoted"
        );
    }
}
