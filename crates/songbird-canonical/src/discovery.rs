//! # 🔍 Canonical Service Discovery
//!
//! **UNIVERSAL DISCOVERY SYSTEM** ✅
//!
//! This module provides canonical service discovery patterns that work
//! with any service discovery mechanism through capability-based interfaces.

// use async_trait::async_trait; // Unused after trait consolidation
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canonical service information structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name identifier
    pub name: String,
    /// Service address
    pub address: String,
    /// Service port
    pub port: u16,
    /// Service metadata
    pub metadata: HashMap<String, String>,
}

impl ServiceInfo {
    /// Create a new service info
    #[must_use]
    pub fn new(name: String, address: String, port: u16) -> Self {
        Self {
            name,
            address,
            port,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the service info
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

// All discovery functionality now uses songbird_types::traits::DiscoveryProvider
// This module serves as a compatibility layer during migration
