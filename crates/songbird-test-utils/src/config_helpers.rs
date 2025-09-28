// Configuration test helpers
///
/// Centralizes common configuration creation patterns used across tests
/// Performance configuration test helpers
pub mod performance {
    use songbird_config::config::SongbirdConfig;

    /// Create a performance-optimized test configuration
    #[must_use]
    pub fn create_performance_test_config() -> SongbirdConfig {
        // Configure performance settings
        // NOTE: Performance config will be re-enabled in next phase of canonical modernization
        // config.performance.enable_async_batching = true;
        // config.performance.batch_size = 100;
        // config.performance.batch_timeout = Duration::from_millis(50);
        SongbirdConfig::default()
    }

    /// Create a minimal test configuration for basic functionality
    #[must_use]
    pub fn create_minimal_test_config() -> SongbirdConfig {
        // Configure minimal settings
        // NOTE: Performance config will be re-enabled in next phase of canonical modernization
        // config.performance.enable_async_batching = false;
        // config.performance.batch_size = 1;
        // config.performance.batch_timeout = Duration::from_millis(1);
        SongbirdConfig::default()
    }

    /// Create a high-performance test configuration for load testing
    #[must_use]
    pub fn create_high_performance_config() -> SongbirdConfig {
        // Configure high performance settings
        // NOTE: Performance config will be re-enabled in next phase of canonical modernization
        // config.performance.enable_async_batching = true;
        // config.performance.batch_size = 1000;
        // config.performance.batch_timeout = Duration::from_millis(500);
        SongbirdConfig::default()
    }
}

/// Network configuration test helpers
pub mod network {
    use songbird_config::config::SongbirdConfig;

    /// Create a test configuration with custom network settings
    #[must_use]
    pub fn create_network_test_config() -> SongbirdConfig {
        SongbirdConfig::default()
    }
}

/// Mock circuit breaker for testing
pub mod circuit_breaker  {
    /// Mock circuit breaker implementation for testing
    pub struct MockCircuitBreaker {
        /// Whether the circuit is open
        pub is_open: bool,
    }

    impl MockCircuitBreaker  {/// Create a new mock circuit breaker
        #[must_use]
        pub fn new() -> Self {
            Self {
                is_open: false,
            }
        }

        /// Check if the circuit is open
        #[must_use]
        pub fn is_open(&self) -> bool {
            self.is_open
        }

        /// Open the circuit
        pub fn open(&mut self) {
            self.is_open = true;
        }

        /// Close the circuit
        pub fn close(&mut self) {
            self.is_open = false;
        }
    }

    impl Default for MockCircuitBreaker {
        fn default() -> Self {
            Self::new()
        }
    }
}
