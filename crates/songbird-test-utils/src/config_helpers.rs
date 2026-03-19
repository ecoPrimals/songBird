// Configuration test helpers
///
/// Centralizes common configuration creation patterns used across tests
///
/// **MODERNIZED**: Now uses canonical config types from songbird-config
/// Performance configuration test helpers
///
/// **CANONICAL**: Uses modern `PerformanceConfig` from songbird-config
pub mod performance {
    use songbird_config::PerformanceConfig;

    /// Create a performance-optimized test configuration
    #[must_use]
    pub const fn create_performance_test_config() -> PerformanceConfig {
        PerformanceConfig {
            buffer_pool_size: Some(1024),
            max_memory_mb: Some(256),
            worker_threads: None, // Auto-detect
            connection_pool_size: Some(100),
            request_timeout_ms: Some(30000),
            enable_zero_copy: Some(true),
            batch_size: Some(100),
            custom_params: None,
        }
    }

    /// Create a minimal test configuration for basic functionality
    #[must_use]
    pub const fn create_minimal_test_config() -> PerformanceConfig {
        PerformanceConfig {
            buffer_pool_size: Some(64),
            max_memory_mb: Some(64),
            worker_threads: Some(1),
            connection_pool_size: Some(10),
            request_timeout_ms: Some(5000),
            enable_zero_copy: Some(false),
            batch_size: Some(1),
            custom_params: None,
        }
    }

    /// Create a high-performance test configuration for load testing
    #[must_use]
    pub const fn create_high_performance_config() -> PerformanceConfig {
        PerformanceConfig {
            buffer_pool_size: Some(4096),
            max_memory_mb: Some(1024),
            worker_threads: None, // Auto-detect
            connection_pool_size: Some(500),
            request_timeout_ms: Some(60000),
            enable_zero_copy: Some(true),
            batch_size: Some(1000),
            custom_params: None,
        }
    }
}

/// Network configuration test helpers
///
/// **CANONICAL**: Uses modern `NetworkConfig` from canonical module
pub mod network {
    use songbird_config::canonical::NetworkConfig;

    /// Create a test configuration with custom network settings
    #[must_use]
    pub fn create_network_test_config() -> NetworkConfig {
        NetworkConfig::default()
    }
}

/// Mock circuit breaker for testing
pub mod circuit_breaker {
    /// Mock circuit breaker implementation for testing
    pub struct MockCircuitBreaker {
        /// Whether the circuit is open
        pub is_open: bool,
    }

    impl MockCircuitBreaker {
        /// Create a new mock circuit breaker
        #[must_use]
        pub const fn new() -> Self {
            Self {
                is_open: false,
            }
        }

        /// Check if the circuit is open
        #[must_use]
        pub const fn is_open(&self) -> bool {
            self.is_open
        }

        /// Open the circuit
        pub const fn open(&mut self) {
            self.is_open = true;
        }

        /// Close the circuit
        pub const fn close(&mut self) {
            self.is_open = false;
        }
    }

    impl Default for MockCircuitBreaker {
        fn default() -> Self {
            Self::new()
        }
    }
}
