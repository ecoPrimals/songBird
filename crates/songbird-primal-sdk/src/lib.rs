//! # Songbird Primal SDK - Comprehensive Integration Platform
//!
//! This crate provides a comprehensive SDK for integrating with Songbird Primals,
//! including universal components, BearDog integration, and zero-cost abstractions.
//! 
//! ## Features
//! 
//! - **Universal Registry**: Comprehensive primal discovery and management
//! - **Capability System**: AI-driven capability orchestration
//! - **BearDog Integration**: Secure entropy and validation systems
//! - **Modern APIs**: Async-first with zero-cost abstractions
//! - **Storage Systems**: Capability-aware storage management
//! - **Discovery**: Advanced service and primal discovery
//! - **Security**: Integrated security and provider systems
//! - **Performance**: Const generics and compile-time optimizations

#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use songbird_types::{SongbirdError,
    performance::{ConstBuffer, PerformanceConfig, ProductionConfig, StackString, StackVec}
};

// ============================================================================
// CORE SDK EXPORTS
// ============================================================================

// Re-export primal-specific types
pub use songbird_types::primal::*;

// Compatibility type aliases for common usage patterns
// (SongbirdResult removed - use songbird_types::errors::SongbirdResult directly)
/// Error type alias for Primal SDK operations
pub type PrimalError = SongbirdError;

// Re-export for backward compatibility during migration
pub use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig as PrimalConfig;

// ============================================================================
// CONSOLIDATED UNIVERSAL PRIMALS FUNCTIONALITY
// ============================================================================

// Core modules from universal-primals consolidation
pub mod adaptive_discovery;

// ============================================================================
// CAPABILITY-BASED MODULES (Primal-Agnostic) ⭐ NEW!
// ============================================================================

/// Security capability provider (replaces beardog hardcoding)
pub mod capability_security;
/// Compute capability provider (replaces toadstool hardcoding)  
pub mod capability_compute;
/// AI capability provider (replaces squirrel hardcoding)
pub mod capability_ai;
/// AI capability client (replaces SquirrelPrimal - zero hardcoding)
pub mod ai_capability;
/// Compute capability client (replaces ToadstoolPrimal - zero hardcoding)
pub mod compute_capability;
/// Security capability client (replaces BeardogPrimal - zero hardcoding)
pub mod security_capability_client;

// Other capability modules
pub mod capability_orchestrator;
pub mod capability_storage;

// ============================================================================
// DEPRECATED: Primal-Specific Modules (Use capability_* instead)
// ============================================================================

// ✅ REMOVED: Deprecated hardcoded primal modules (Nov 9, 2025)
// Use capability_security, capability_compute, capability_ai instead
// #[deprecated(since = "0.4.0")]
// pub mod beardog; // Use capability_security instead
// pub mod toadstool; // Use capability_compute instead
// pub mod squirrel; // Use capability_ai instead
pub mod config;
pub mod discovery;
pub mod global_adapter;
pub mod modern_api;
pub mod modernization_utils;
pub mod providers;
pub mod registry;
pub mod security_provider;
pub mod simple_primal_registry;
pub mod startup;
pub mod storage;
pub mod traits;
pub mod types;
pub mod universal_adapter;
pub mod universal_registry;
pub mod zero_cost_registry;

// ============================================================================
// PERFORMANCE-OPTIMIZED API SURFACE
// ============================================================================

/// **ZERO-COST**: Performance-optimized primal connection pool
#[derive(Debug)]
pub struct OptimizedPrimalPool<const MAX_CONNECTIONS: usize = 16> {
    connections: ConstBuffer<PrimalConnection, MAX_CONNECTIONS>,
    config: PerformanceConfig<true, false>, // Production config
}

impl<const MAX_CONNECTIONS: usize> OptimizedPrimalPool<MAX_CONNECTIONS>  {/// Create new optimized pool - compile-time sized
    #[must_use]
    pub const fn new() -> Self  {Self {
            connections: ConstBuffer::new(),
            config: PerformanceConfig::new(),
        }
    }

    /// Add connection to pool - zero-cost bounds checking
    pub fn add_connection(&mut self, connection: PrimalConnection) -> SongbirdResult<()>  {
        self.connections.try_push(connection)
            .map_err(|_| SongbirdError::Configuration {
                field: "connection_pool".to_string(),
                message: format!("Pool at maximum capacity: {}", MAX_CONNECTIONS),
                current_value: Some(self.connections.len().to_string()),
                expected_format: Some(format!("< {}", MAX_CONNECTIONS)),
                suggestion: Some("Increase MAX_CONNECTIONS const generic parameter ".to_string()),
            })
    }

    /// Get pool statistics - zero allocation
    #[must_use]
    pub const fn stats(&self) -> PoolStats  {
        PoolStats  {
            active_connections: self.connections.len(),
            max_connections: MAX_CONNECTIONS,
            utilization_percent: (self.connections.len() * 100) / MAX_CONNECTIONS,
        }
    }
}

/// **ZERO-COST**: Pool statistics with const generics
#[derive(Debug, Clone, Copy)]
pub struct PoolStats  {pub active_connections: usize,
    pub max_connections: usize,
    pub utilization_percent: usize,
}

/// **ZERO-COST**: Primal connection with stack-allocated metadata
#[derive(Debug, Clone)]
pub struct PrimalConnection  {pub id: uuid::Uuid,
    pub primal_type: CanonicalPrimalType,
    pub endpoint: StackString<256>, // Stack-allocated, no heap
    pub metadata: StackVec<(StackString<64>, StackString<256>), 8>, // Stack-allocated key-value pairs
}

impl PrimalConnection  {/// Create new connection - zero heap allocation for metadata
    #[must_use]
    pub fn new(id: uuid::Uuid, primal_type: CanonicalPrimalType, endpoint: &str) -> Self  {let mut stack_endpoint = StackString::new();
        let _ = stack_endpoint.try_push_str(endpoint); // Truncate if too long
        
        Self {
            id,
            primal_type,
            endpoint: stack_endpoint,
            metadata: StackVec::new(),
        }
    }

    /// Add metadata - zero heap allocation
    pub fn add_metadata(&mut self, key: &str, value: &str) -> Result<(), &'static str> {
        let mut stack_key = StackString::new();
        let mut stack_value = StackString::new();
        
        stack_key.try_push_str(key).map_err(|_| "Key too long ")?;
        stack_value.try_push_str(value).map_err(|_| "Value too long ")?;
        
        self.metadata.try_push((stack_key, stack_value))
            .map_err(|_| "Metadata capacity exceeded ")
    }
}

/// **PERFORMANCE-OPTIMIZED**: Main SDK interface with const generics
pub struct PrimalSDK<const POOL_SIZE: usize = 16>  {registry: Box<dyn crate::traits::PrimalRegistry>,
    orchestrator: capability_orchestrator::CapabilityOrchestrator,
    discovery: adaptive_discovery::AdaptiveDiscovery,
    connection_pool: OptimizedPrimalPool<POOL_SIZE>,
    performance_config: ProductionConfig,
}

impl<const POOL_SIZE: usize> PrimalSDK<POOL_SIZE>  {/// Create a new PrimalSDK instance with compile-time optimizations
    pub async fn new() -> SongbirdResult<Self>  {
        let registry = Box::new(simple_primal_registry::SimplePrimalRegistry::new());
        let orchestrator = capability_orchestrator::CapabilityOrchestrator::new().await?;
        let discovery = adaptive_discovery::AdaptiveDiscovery::new().await?;

        Ok(Self {
            registry,
            orchestrator,
            discovery,
            connection_pool: OptimizedPrimalPool::new(),
            performance_config: ProductionConfig::new(),
        })
    }

    /// Create optimized SDK for high-performance scenarios
    pub async fn new_optimized() -> SongbirdResult<Self> {
        let mut sdk = Self::new().await?;
        
        // Apply performance optimizations
        sdk.performance_config.debug_only(|| {
            tracing::debug!("PrimalSDK initialized in production mode with pool size: {}", POOL_SIZE);
        });

        Ok(sdk)
    }

    /// Get access to the primal registry
    pub fn registry(&self) -> &dyn crate::traits::PrimalRegistry {
        self.registry.as_ref()
    }

    /// Get access to the capability orchestrator
    pub fn orchestrator(&self) -> &capability_orchestrator::CapabilityOrchestrator {
        &self.orchestrator
    }

    /// Get access to the adaptive discovery system
    pub fn discovery(&self) -> &adaptive_discovery::AdaptiveDiscovery {
        &self.discovery
    }

    /// Get access to the optimized connection pool
    pub fn connection_pool(&self) -> &OptimizedPrimalPool<POOL_SIZE> {
        &self.connection_pool
    }

    /// Get mutable access to connection pool for management
    pub fn connection_pool_mut(&mut self) -> &mut OptimizedPrimalPool<POOL_SIZE> {
        &mut self.connection_pool
    }

    /// Get pool statistics - zero cost
    #[must_use]
    pub fn pool_stats(&self) -> PoolStats {
        self.connection_pool.stats()
    }
}

// ============================================================================
// TYPE ALIASES FOR COMMON CONFIGURATIONS
// ============================================================================

/// Standard SDK with default pool size
pub type StandardPrimalSDK = PrimalSDK<16>;

/// High-performance SDK with large connection pool
pub type HighPerformancePrimalSDK = PrimalSDK<64>;

/// Lightweight SDK for resource-constrained environments
pub type LightweightPrimalSDK = PrimalSDK<4>;

// ============================================================================
// PERFORMANCE UTILITIES
// ============================================================================

/// **ZERO-COST**: Create compile-time primal type identifiers
#[macro_export]
macro_rules! primal_type_id {
    ("security") => { $crate::const_string_id!("security") };
    ("storage") => { $crate::const_string_id!("storage") };
    ("compute") => { $crate::const_string_id!("compute") };
    ("network") => { $crate::const_string_id!("network") };
    ("gaming") => { $crate::const_string_id!("gaming") };
    ("intelligence") => { $crate::const_string_id!("intelligence") };
    ("observability") => { $crate::const_string_id!("observability") };
    ($custom:expr) => { $crate::const_string_id!($custom) };
}
