//! Capability registry for tracking primal capabilities

use std::collections::HashMap;

use super::types::Capability;

/// Registry of primal capabilities discovered dynamically
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    /// Map of primal name to their declared capabilities
    pub primal_capabilities: HashMap<String, Vec<Capability>>,
    /// Map of capability type to primals that provide it
    pub capability_providers: HashMap<String, Vec<String>>,
    /// Last update timestamp for each primal
    pub last_updated: HashMap<String, chrono::DateTime<chrono::Utc>>,
}
