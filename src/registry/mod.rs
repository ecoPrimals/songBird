//! Service registry module
//!
//! This module provides service registry functionality for songbird orchestration.

use songbird_errors::Result;

pub use songbird_registry::{health::*, plugin::*, service::*};

// Additional convenience functions for backward compatibility
pub async fn default_service_registry() -> Result<ServiceRegistry> {
    ServiceRegistry::new().await
}

pub fn create_plugin_registry() -> DynamicPluginRegistry {
    DynamicPluginRegistry::new()
}

pub async fn create_service_registry() -> Result<ServiceRegistry> {
    ServiceRegistry::new().await
}
