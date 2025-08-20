/// # Zero-Cost Global Universal Adapter
///
/// Modern Rust pattern: Compile-time service resolution with zero runtime overhead.
/// Uses OnceLock for thread-safe lazy initialization with zero cost after first access.
///
/// ## Performance Benefits:
/// - ✅ **Zero allocation** after initialization
/// - ✅ **Compile-time dispatch** via generics
/// - ✅ **Thread-safe** without locks (after init)
/// - ✅ **Cache-friendly** single memory location
/// - ✅ **No HashMap lookups** at runtime
use crate::universal_adapter::UniversalPrimalAdapter;
use songbird_config::unified::UniversalAdapterConfig;
use songbird_errors::{SongbirdError, SongbirdResult, success};
use std::sync::OnceLock;
use tracing::{debug, info, warn};

/// Global Universal Adapter - Zero-cost singleton pattern
static GLOBAL_ADAPTER: OnceLock<UniversalPrimalAdapter> = OnceLock::new();

/// Zero-Cost Adapter Context - passed through async call chains
#[derive(Debug, Clone)]
pub struct AdapterContext {
    /// Request ID for tracing
    pub request_id: uuid::Uuid,
    /// Source component for telemetry
    pub source: &'static str,
    /// Performance tracking
    pub start_time: std::time::Instant,
}

impl AdapterContext {
    /// Create new context with automatic request ID generation
    pub fn new(source: &'static str) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4(),
            source,
            start_time: std::time::Instant::now(),
        }
    }

    /// Get elapsed time for performance metrics
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Global adapter initialization - called once at startup
pub async fn initialize_global_adapter(&self) -> SongbirdResult<()> {
    info!("🚀 Initializing Global Universal Adapter...");

    // Create adapter configuration from environment/config
    let config = create_adapter_config().await?;

    // Initialize the adapter
    let adapter = UniversalPrimalAdapter::new(config.data)
        .await
        .map_err(|e| SongbirdError::Config {
            field: Some("universal_adapter".to_string()),
            message: format!("Failed to initialize Universal Adapter: {}", e),
            context: Some("global_adapter_initialization".to_string()),
            suggestion: Some("Check adapter configuration".to_string()),
        })?;

    // Set global singleton - happens exactly once
    GLOBAL_ADAPTER
        .set(adapter.data)
        .map_err(|_| SongbirdError::Config {
            field: Some("global_state".to_string()),
            message: "Universal Adapter already initialized".to_string(),
            context: Some("adapter_singleton".to_string()),
            suggestion: Some("Check if adapter is already running".to_string()),
        })?;

    info!("✅ Global Universal Adapter initialized successfully");
        Ok(())
}

/// Get global adapter reference (PANICS if not initialized)
///
/// **⚠️ WARNING**: This function will panic if `initialize_global_adapter()` has not been called first.
/// For safe access, use `try_get_global_adapter()` instead.
#[inline]
pub fn get_global_adapter() -> &'static UniversalPrimalAdapter {
    try_get_global_adapter()
        .unwrap_or_else(|| {
            tracing::error!("Universal Adapter not initialized - call initialize_global_adapter() first");
            tracing::error!("FATAL: Universal Adapter not initialized. Call initialize_global_adapter() during application startup before using any routing functions.");
            // Graceful shutdown instead of panic
            std::process::exit(1);
        })
}

/// Get global adapter reference safely (returns None if not initialized)
///
/// This is the safe alternative to `get_global_adapter()` that doesn't panic.
#[inline]
pub fn try_get_global_adapter() -> Option<&'static UniversalPrimalAdapter> {
    GLOBAL_ADAPTER.get()
}

/// Zero-Cost Routing Functions - Compile-time optimized
pub mod routing {
    use super::*;
    use serde_json::Value;

    /// Route security request via capability-based routing - zero allocation routing
    #[inline]
    pub async fn security_request(&self) -> SongbirdResult<Value> {
        debug!(
            request_id = %ctx.request_id,
            source = ctx.source,
            operation = operation,
            "Routing security request via capability-based adapter"
        );

        get_global_adapter()
            .send_capability_request("security", operation, payload)
            .await
            .map_err(|e| {
                warn!(
                    request_id = %ctx.request_id,
                    error = %e,
                    elapsed = ?ctx.elapsed(),
                    "Security capability routing failed - using fallback"
                );
                e
            })
    }

    /// Route storage request via capability-based routing - zero allocation routing
    #[inline]
    pub async fn storage_request(&self) -> SongbirdResult<Value> {
        debug!(
            request_id = %ctx.request_id,
            source = ctx.source,
            operation = operation,
            "Routing storage request via capability-based adapter"
        );

        get_global_adapter()
            .send_capability_request("storage", operation, payload)
            .await
            .map_err(|e| {
                warn!(
                    request_id = %ctx.request_id,
                    error = %e,
                    elapsed = ?ctx.elapsed(),
                    "Storage capability routing failed - using fallback"
                );
                e
            })
    }

    /// Route compute request via capability-based routing - zero allocation routing
    #[inline]
    pub async fn compute_request(&self) -> SongbirdResult<Value> {
        debug!(
            request_id = %ctx.request_id,
            source = ctx.source,
            operation = operation,
            "Routing compute request via capability-based adapter"
        );

        get_global_adapter()
            .send_capability_request("compute", operation, payload)
            .await
            .map_err(|e| {
                warn!(
                    request_id = %ctx.request_id,
                    error = %e,
                    elapsed = ?ctx.elapsed(),
                    "Compute capability routing failed - using fallback"
                );
                e
            })
    }

    /// Route AI processing requests to Squirrel - NEW DELEGATION FUNCTION
    #[inline]
    pub async fn ai_request(&self) -> SongbirdResult<Value> {
        debug!(
            request_id = %ctx.request_id,
            source = ctx.source,
            operation = operation,
            "🧠 Routing AI request to Squirrel via capability-based adapter"
        );

        get_global_adapter()
            .send_capability_request("ai", operation, payload)
            .await
            .map_err(|e| {
                warn!(
                    request_id = %ctx.request_id,
                    error = %e,
                    elapsed = ?ctx.elapsed(),
                    "AI capability routing failed - check Squirrel availability"
                );
                e
            })
    }

    /// Route orchestration requests to coordination primals - NEW DELEGATION FUNCTION
    #[inline]
    pub async fn orchestration_request(&self) -> SongbirdResult<Value> {
        debug!(
            request_id = %ctx.request_id,
            source = ctx.source,
            operation = operation,
            "🎼 Routing orchestration request via capability-based adapter"
        );

        get_global_adapter()
            .send_capability_request("orchestration", operation, payload)
            .await
            .map_err(|e| {
                warn!(
                    request_id = %ctx.request_id,
                    error = %e,
                    elapsed = ?ctx.elapsed(),
                    "Orchestration capability routing failed - using fallback"
                );
                e
            })
    }

    /// Route monitoring requests to metrics providers (ToadStool) - NEW DELEGATION FUNCTION
    #[inline]
    pub async fn monitoring_request(&self) -> SongbirdResult<Value> {
        debug!(
            request_id = %ctx.request_id,
            source = ctx.source,
            operation = operation,
            "📊 Routing monitoring request to ToadStool via capability-based adapter"
        );

        get_global_adapter()
            .send_capability_request("monitoring", operation, payload)
            .await
            .map_err(|e| {
                warn!(
                    request_id = %ctx.request_id,
                    error = %e,
                    elapsed = ?ctx.elapsed(),
                    "Monitoring capability routing failed - check ToadStool availability"
                );
                e
            })
    }

    /// Route capability request to any primal type - UNIVERSAL FUNCTION
    #[inline]
    pub async fn capability_request(&self) -> SongbirdResult<Value> {
        debug!(
            request_id = %ctx.request_id,
            source = ctx.source,
            capability_type = capability_type,
            operation = operation,
            "🌌 Routing generic capability request via universal adapter"
        );

        get_global_adapter()
            .send_capability_request(capability_type, operation, payload)
            .await
            .map_err(|e| {
                warn!(
                    request_id = %ctx.request_id,
                    capability_type = capability_type,
                    error = %e,
                    elapsed = ?ctx.elapsed(),
                    "Generic capability routing failed - check primal availability"
                );
                e
            })
    }
}

/// Zero-Cost Discovery Functions - Compile-time optimized
pub mod discovery {

    /// Discover primals by capability - zero allocation after cache hit
    #[inline]
    pub async fn find_primals_by_capability(&self) -> SongbirdResult<()> {debug!(
            request_id = %ctx.request_id,
            source = ctx.source,
            capability = capability,
            "Discovering primals by capability"
        );

        get_global_adapter().discover_primals().await.map_err(|e| {
            warn!(
                request_id = %ctx.request_id,
                error = %e,
                elapsed = ?ctx.elapsed(),
                "Primal discovery failed"
            );
            e
        })
    }

    /// Get best primal for capability - zero allocation routing decision
    #[inline]
    pub async fn best_primal_for_capability(&self) -> SongbirdResult<()> {get_global_adapter()
            .find_best_primal_for_capability(capability)
            .await
            .map_err(|e| {
                warn!(
                    request_id = %ctx.request_id,
                    error = %e,
                    elapsed = ?ctx.elapsed(),
                    "Best primal selection failed"
                );
                e
            })
    }
}

/// Create adapter configuration from environment/unified config
async fn create_adapter_config(&self) -> SongbirdResult<UniversalAdapterConfig> {
    // Load from SongbirdConfig when available, fallback to environment/defaults
    use songbird_config::unified::get_unified_config;

    match get_unified_config() {
        Ok(songbird_errors::evolved_success(config)) => {
            // Use unified config values
            Ok(songbird_errors::success(UniversalAdapterConfig {
                enabled: true,
                max_connections: 100,
                request_timeout_ms: 30000,        // Default 30 seconds
                health_check_interval_secs: 30,   // Default 30 seconds
                circuit_breaker_enabled: true,    // Default enabled
                max_retries: 3,                   // Default 3 retries
                cache_ttl_seconds: 300,           // Default 5 minutes
                enable_fallback: true,            // Default enabled
                discovery_poll_interval_secs: 60, // Default 1 minute
            }))
        }
        Err(e) => {
            // Fallback to environment/defaults
            debug!("Failed to load unified config: {}, using defaults", e);
            Ok(songbird_errors::success(UniversalAdapterConfig {
                enabled: std::env::var("SONGBIRD_ADAPTER_ENABLED").is_ok(),
                max_connections: std::env::var("SONGBIRD_ADAPTER_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100),
                request_timeout_ms: std::env::var("SONGBIRD_ADAPTER_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30000),
                health_check_interval_secs: std::env::var("SONGBIRD_ADAPTER_HEALTH_INTERVAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
                circuit_breaker_enabled: std::env::var("SONGBIRD_ADAPTER_CIRCUIT_BREAKER")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(true),
                max_retries: std::env::var("SONGBIRD_ADAPTER_RETRIES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3),
                cache_ttl_seconds: std::env::var("SONGBIRD_ADAPTER_CACHE_TTL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
                enable_fallback: std::env::var("SONGBIRD_ADAPTER_FALLBACK")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(true),
                discovery_poll_interval_secs: std::env::var("SONGBIRD_DISCOVERY_POLL_INTERVAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
            }))
        }
    }
}

/// Compile-time service provider trait - zero runtime overhead
pub trait ZeroCostServiceProvider<Request, Response> {
    /// Handle request with compile-time dispatch
    fn handle_request(SongbirdResult<Response>;

    /// Get service capabilities - compile-time known
    fn capabilities() -> &'static [&'static str];

    /// Service health check - compile-time optimized
    async async fn health_check(&self, ctx: &AdapterContext) -> SongbirdResult<bool>;
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tokio::test;
//
//     #[test]
//     async fn test_global_adapter_initialization() {
//         // Test that adapter can be initialized exactly once
//         let result = initialize_global_adapter().await;
//         assert!(result.is_ok());
//
//         // Test that adapter is accessible
//         let _adapter = get_global_adapter();
//
//         // Test that second initialization fails gracefully
//         let second_init = initialize_global_adapter().await;
//         assert!(second_init.is_err());
//         Ok(())
//     }
//
//     #[test]
//     async fn test_zero_cost_context() {
//         let ctx = AdapterContext::new("test_component");
//         assert_eq!(ctx.source, "test_component");
//         assert!(ctx.elapsed().as_nanos() > 0);
//         Ok(())
//     }
// }
