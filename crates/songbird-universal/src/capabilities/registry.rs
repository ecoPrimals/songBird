//! Capability registry for tracking primal capabilities

use std::collections::HashMap;
use std::sync::Arc;

use super::qos_selection::QoSProviderSelector;
use super::types::Capability;

/// Registry of primal capabilities discovered dynamically
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    /// Map of primal name to their declared capabilities
    pub primal_capabilities: HashMap<String, Vec<Capability>>,
    /// Map of capability type to primals that provide it
    pub capability_providers: HashMap<String, Vec<String>>,
    /// Last update timestamp for each primal
    pub last_updated: HashMap<String, chrono::DateTime<chrono::Utc>>,
    /// Optional QoS-aware provider selector (✨ NEW: intelligent selection)
    pub qos_selector: Option<Arc<QoSProviderSelector>>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self {
            primal_capabilities: HashMap::new(),
            capability_providers: HashMap::new(),
            last_updated: HashMap::new(),
            qos_selector: Some(Arc::new(QoSProviderSelector::new())),
        }
    }
}
