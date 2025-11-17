//! Port Configuration
//!
//! Port range and port-related configuration structures.

use serde::{Deserialize, Serialize};

/// Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

impl Default for PortRange {
    fn default() -> Self {
        Self {
            start: 7000,
            end: 7100,
        }
    }
}
