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
    /// Server hostname
    pub hostname: String,

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
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(GlobalStats::default())),
        }
    }

    /// Get profile for a server (or create new)
    pub fn get_profile(&self, hostname: &str) -> Option<ServerProfile> {
        let profiles = self
            .profiles
            .read()
            .expect("BUG: TLS profiler lock poisoned - indicates panic during profile update");
        profiles.get(hostname).cloned()
    }

    /// Record successful connection
    pub fn record_success(
        &self,
        hostname: &str,
        extensions: Vec<ExtensionType>,
        cipher: u16,
        handshake_duration: Duration,
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

            profile.success_count += 1;
            profile.successful_extensions = extensions.clone();
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

            // Update best extension set if this is more successful
            if extensions.len() >= stats.best_extension_set.len() {
                stats.best_extension_set = extensions;
            }

            stats.best_cipher = Some(cipher);
        }
    }

    /// Record failed connection
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
            if let Some(cipher_suite) = cipher {
                if !profile.failed_ciphers.contains(&cipher_suite) {
                    profile.failed_ciphers.push(cipher_suite);
                }
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
    pub fn recommend_extensions(&self, hostname: &str) -> Vec<ExtensionType> {
        // Check if we have a profile for this server
        if let Some(profile) = self.get_profile(hostname) {
            if profile.success_count > 0 {
                // Use last successful extension set
                return profile.successful_extensions.clone();
            }
        }

        // Fall back to global best extension set
        let stats = self
            .stats
            .read()
            .expect("BUG: TLS profiler stats lock poisoned - indicates panic during stats update");
        if !stats.best_extension_set.is_empty() {
            return stats.best_extension_set.clone();
        }

        // Fall back to standard set
        ExtensionSet::standard().extensions
    }

    /// Get recommended cipher suite for a server
    pub fn recommend_cipher(&self, hostname: &str) -> Option<u16> {
        // Check server profile
        if let Some(profile) = self.get_profile(hostname) {
            if let Some(cipher) = profile.successful_cipher {
                return Some(cipher);
            }
        }

        // Fall back to global best cipher
        let stats = self
            .stats
            .read()
            .expect("BUG: TLS profiler stats lock poisoned - indicates panic during stats update");
        stats.best_cipher
    }

    /// Get global statistics
    pub fn get_stats(&self) -> GlobalStats {
        self.stats
            .read()
            .expect("BUG: TLS profiler stats lock poisoned - indicates panic during stats update")
            .clone()
    }

    /// Get all profiles (for debugging/analysis)
    pub fn get_all_profiles(&self) -> HashMap<String, ServerProfile> {
        self.profiles
            .read()
            .expect("BUG: TLS profiler lock poisoned - indicates panic during profile update")
            .clone()
    }

    /// Clear all profiles (reset learning)
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
            hostname: hostname.to_string(),
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
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 0.0;
        }
        self.success_count as f32 / total as f32
    }

    /// Is this server reliable? (>= 80% success rate)
    pub fn is_reliable(&self) -> bool {
        self.reliability >= 0.8
    }

    /// Should we retry with different settings?
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
    pub fn success_rate(&self) -> f32 {
        let total = self.total_successes + self.total_failures;
        if total == 0 {
            return 0.0;
        }
        self.total_successes as f32 / total as f32
    }

    /// Get most problematic extensions
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

        profiler.record_success(hostname, extensions.clone(), cipher, Duration::from_secs(1));

        let profile = profiler.get_profile(hostname).unwrap();
        assert_eq!(profile.success_count, 1);
        assert_eq!(profile.failure_count, 0);
        assert_eq!(profile.successful_cipher, Some(cipher));
        assert_eq!(profile.reliability, 1.0);
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
        assert_eq!(profile.reliability, 0.8);
        assert!(profile.is_reliable());
    }

    #[test]
    fn test_recommendations() {
        let profiler = ServerProfiler::new();
        let hostname = "www.example.com";
        let extensions =
            vec![ExtensionType::Sni, ExtensionType::Alpn, ExtensionType::SupportedVersions];

        profiler.record_success(hostname, extensions.clone(), 0x1303, Duration::from_secs(1));

        let recommended = profiler.recommend_extensions(hostname);
        assert_eq!(recommended, extensions);

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
        assert_eq!(stats.success_rate(), 2.0 / 3.0);
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
}
