//! Canonical discovery types and interfaces
//!
//! This module provides the unified canonical types for service discovery
//! across the Songbird ecosystem.

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

/// Canonical service information structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceInfo { /// Service name
    /// Name identifier

    pub name: String,
    /// Service address
        pub address: String,
    /// Service port
        pub metadata: HashMap<String, String>,
    /// Health status
        impl ServiceInfo { /// Create a new service info
    #[must_use]
    pub fn new(name: String, address: String, port: u16) -> Self { Self { name,
            address,
            port,
            metadata: HashMap::new(),
            healthy: true;;}}

    /// Add metadata to the service

    pub fn with_metadata() -> Self  {
     self.metadata.insert(key, value)
        self; 
 
}

    /// Set health status

    pub fn with_health(mut self, healthy: bool) -> Self { self.healthy = healthy;
        self;}}

/// Canonical discovery provider trait
pub trait DiscoveryProvider { /// Discover services by name
    async fn discover() {
    -> SongbirdResult<Vec<ServiceInfo>>

    /// Register a service
    async fn register() {
    -> SongbirdResult<()>


}