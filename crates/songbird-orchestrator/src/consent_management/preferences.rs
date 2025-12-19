//! User preferences for consent

use serde::{Deserialize, Serialize};

/// User consent preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Auto-approve operations under this cost
    pub auto_approve_under_cost: Option<f64>,

    /// Always require consent for these operations
    pub always_require_consent: Vec<String>,

    /// Never allow these operations
    pub blocked_operations: Vec<String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            auto_approve_under_cost: Some(10.0), // Default: auto-approve under $10
            always_require_consent: vec![],
            blocked_operations: vec![],
        }
    }
}
