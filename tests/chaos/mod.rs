//! Chaos Engineering Tests for Songbird
//!
//! These tests intentionally introduce failures and unexpected conditions
//! to verify system resilience and recovery capabilities.
//!
//! ## Chaos Testing Principles
//! 1. Hypothesis: Define expected behavior under chaos
//! 2. Inject: Introduce controlled failures
//! 3. Observe: Monitor system response
//! 4. Verify: Confirm system meets expectations

#![cfg(test)]

pub mod network_chaos;
pub mod resource_chaos;
pub mod timing_chaos;
pub mod state_chaos;

/// Common chaos testing utilities
pub mod common {
    use std::time::Duration;
    
    /// Chaos injection configuration
    pub struct ChaosConfig {
        pub failure_rate: f64,
        pub duration: Duration,
        pub impact_radius: ImpactRadius,
    }
    
    /// Scope of chaos injection
    pub enum ImpactRadius {
        Single,      // Single component
        Service,     // Entire service
        System,      // Entire system
    }
    
    impl Default for ChaosConfig {
        fn default() -> Self {
            Self {
                failure_rate: 0.1, // 10% failure rate
                duration: Duration::from_secs(30),
                impact_radius: ImpactRadius::Single,
            }
        }
    }
}

