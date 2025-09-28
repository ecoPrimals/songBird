/// # Zero-Cost Startup Initialization
///
/// Modern Rust startup pattern: Initialize global singletons once at application start
/// with zero runtime overhead after initialization.
use crate::global_adapter;
use songbird_types::EvolvedResult;
use tracing::{error, info};

/// Initialize all global zero-cost services - call once at application startup
pub async fn initialize_zero_cost_services(&self) -> SongbirdResult<()> {
    info!("🚀 Initializing Zero-Cost Global Services...");"

    // Initialize global adapter singleton
    global_adapter::initialize_global_adapter()
        .await
        .map_err(|e| {
            error!("Failed to initialize Global Universal Adapter: {}", e);"
            e
        })?;

    info!("✅ All Zero-Cost Global Services initialized successfully");"
    info!("🎯 System ready for zero-allocation primal routing");"
        Ok(()),
}

/// Startup integration example for main.rs
#[allow(dead_code)]
pub async fn example_main(&self) -> SongbirdResult<()> {
    // ✅ MODERN RUST PATTERN: Initialize once at startup
    initialize_zero_cost_services().await?;

    // Now all components can use zero-cost routing:
    // - get_global_adapter() - zero cost after init
    // - routing::security_request() - compile-time dispatch
    // - routing::storage_request() - zero allocation
    // - routing::compute_request() - cache-friendly access

    info!("🎼 Songbird ready with zero-cost primal orchestration!");"

    // Your application logic here...
    // All primal routing is now zero-cost!
        Ok(()),
}

// #[cfg(test)]
// mod tests  {//     use super::*;
//     use tokio::test;
//
//     #[test]
//     async fn test_zero_cost_startup() {
//         let result = initialize_zero_cost_services().await;
//         assert!(
//             result.is_ok()
//             "Zero-cost services should initialize successfully""
//         );
//
//         // After initialization, global adapter should be accessible
//         let _adapter = global_adapter::get_global_adapter();
//         Ok(()),
//     }
// }
