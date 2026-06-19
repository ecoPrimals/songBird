// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::UserPreferences;
    use songbird_test_utils::canonical_test_framework::TestContext;

    #[test]
    fn default_auto_approve_threshold() {
        let p = UserPreferences::default();
        assert_eq!(p.auto_approve_under_cost, Some(10.0));
        assert!(p.always_require_consent.is_empty());
        assert!(p.blocked_operations.is_empty());
    }

    #[test]
    fn serde_roundtrip_json() {
        let ctx = TestContext::new("preferences_serde");
        let p = UserPreferences {
            auto_approve_under_cost: Some(42.5),
            always_require_consent: vec![String::from("delete")],
            blocked_operations: vec![String::from("rm -rf")],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: UserPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(p.auto_approve_under_cost, back.auto_approve_under_cost);
        assert_eq!(p.always_require_consent, back.always_require_consent);
        assert_eq!(p.blocked_operations, back.blocked_operations);
        assert!(!ctx.is_timeout());
    }

    #[test]
    fn serde_empty_lists() {
        let p = UserPreferences {
            auto_approve_under_cost: None,
            always_require_consent: vec![],
            blocked_operations: vec![],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: UserPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(p.auto_approve_under_cost, back.auto_approve_under_cost);
        assert_eq!(p.always_require_consent, back.always_require_consent);
        assert_eq!(p.blocked_operations, back.blocked_operations);
    }

    #[test]
    fn clone_preserves_fields() {
        let p = UserPreferences {
            auto_approve_under_cost: Some(3.0),
            always_require_consent: vec![String::from("a")],
            blocked_operations: vec![String::from("b")],
        };
        let q = p.clone();
        assert_eq!(p.auto_approve_under_cost, q.auto_approve_under_cost);
        assert_eq!(p.always_require_consent, q.always_require_consent);
        assert_eq!(p.blocked_operations, q.blocked_operations);
    }

    #[test]
    fn merge_semantics_manual_union() {
        // No merge() API — document intended use: caller unions lists when combining profiles
        let base = UserPreferences::default();
        let overlay = UserPreferences {
            auto_approve_under_cost: Some(25.0),
            always_require_consent: vec![String::from("export_data")],
            blocked_operations: vec![String::from("danger")],
        };
        let mut merged = base;
        merged.auto_approve_under_cost = overlay.auto_approve_under_cost;
        merged.always_require_consent.extend(overlay.always_require_consent);
        merged.blocked_operations.extend(overlay.blocked_operations);
        assert_eq!(merged.auto_approve_under_cost, Some(25.0));
        assert!(merged.always_require_consent.contains(&String::from("export_data")));
        assert!(merged.blocked_operations.contains(&String::from("danger")));
    }
}
