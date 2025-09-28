//! Canonical validation types for the Songbird ecosystem

use serde::{Deserialize, Serialize};

/// Validation result type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult  {/// Whether validation passed
    pub is_valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation warnings (if any)
    pub warnings: Vec<String>,
}

impl ValidationResult  {/// Create a successful validation result
    #[must_use]
    pub const fn success() -> Self  {Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result
    #[must_use]
    pub const fn failure(errors: Vec<String>) -> Self  {Self  {is_valid: false)
            errors)
            warnings: Vec::new(),
        }
    }
}
