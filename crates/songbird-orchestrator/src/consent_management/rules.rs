// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Auto-approval rules

/// Rule for auto-approving consent
#[derive(Debug, Clone)]
pub struct AutoApprovalRule {
    pub name: String,
    pub max_cost: Option<f64>,
    pub operations: Vec<String>,
}

impl AutoApprovalRule {
    #[must_use]
    pub fn matches(&self, operation: &str, cost: Option<f64>) -> bool {
        // Check operation
        if !self.operations.is_empty() && !self.operations.contains(&operation.to_string()) {
            return false;
        }

        // Check cost
        if let Some(max) = self.max_cost
            && let Some(actual_cost) = cost
            && actual_cost > max
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::AutoApprovalRule;

    fn rule_named(ops: &[&str], max: Option<f64>) -> AutoApprovalRule {
        AutoApprovalRule {
            name: "t".to_string(),
            max_cost: max,
            operations: ops.iter().map(ToString::to_string).collect(),
        }
    }

    #[test]
    fn matches_empty_operations_allows_any_operation() {
        let r = rule_named(&[], Some(100.0));
        assert!(r.matches("anything", Some(50.0)));
        assert!(r.matches("other", None));
    }

    #[test]
    fn matches_requires_listed_operation_when_non_empty() {
        let r = rule_named(&["read", "write"], None);
        assert!(r.matches("read", None));
        assert!(r.matches("write", None));
        assert!(!r.matches("delete", None));
    }

    #[test]
    fn matches_rejects_cost_above_max() {
        let r = rule_named(&[], Some(10.0));
        assert!(!r.matches("op", Some(10.01)));
        assert!(r.matches("op", Some(10.0)));
    }

    #[test]
    fn matches_allows_missing_cost_when_max_set() {
        let r = rule_named(&[], Some(5.0));
        assert!(r.matches("op", None));
    }

    #[test]
    fn matches_allows_any_cost_when_max_none() {
        let r = rule_named(&["x"], None);
        assert!(r.matches("x", Some(1e9)));
    }

    #[test]
    fn matches_operation_string_conversion() {
        let r = rule_named(&["sync"], None);
        assert!(r.matches("sync", None));
        assert!(!r.matches("Sync", None));
    }

    #[test]
    fn matches_zero_max_rejects_positive_cost() {
        let r = rule_named(&[], Some(0.0));
        assert!(!r.matches("op", Some(0.0001)));
        assert!(r.matches("op", Some(0.0)));
    }

    #[test]
    fn clone_and_debug_do_not_panic() {
        let r = rule_named(&["a"], Some(1.0));
        let _ = format!("{r:?}");
    }

    #[test]
    fn matches_exact_cost_at_max() {
        let r = rule_named(&["op"], Some(5.0));
        assert!(r.matches("op", Some(5.0)));
    }

    #[test]
    fn empty_ops_with_zero_max_still_checks_cost() {
        let r = rule_named(&[], Some(0.0));
        assert!(!r.matches("x", Some(0.0001)));
    }

    #[test]
    fn multiple_ops_any_match() {
        let r = rule_named(&["a", "b", "c"], None);
        assert!(r.matches("b", None));
    }

    #[test]
    fn cost_none_skips_cost_check_with_max() {
        let r = rule_named(&["z"], Some(1.0));
        assert!(r.matches("z", None));
    }

    #[test]
    fn unicode_operation_name() {
        let r = rule_named(&["日本語"], None);
        assert!(r.matches("日本語", None));
    }

    #[test]
    fn long_operation_list() {
        let ops: Vec<String> = (0..20).map(|i| format!("op{i}")).collect();
        let r = AutoApprovalRule {
            name: "many".to_string(),
            max_cost: None,
            operations: ops,
        };
        assert!(r.matches("op7", None));
        assert!(!r.matches("missing", None));
    }

    #[test]
    fn max_cost_negative_values() {
        let r = rule_named(&[], Some(-1.0));
        assert!(!r.matches("op", Some(0.0)));
    }

    #[test]
    fn fractional_cost_boundary() {
        let r = rule_named(&[], Some(1.0));
        assert!(r.matches("op", Some(1.0)));
        assert!(!r.matches("op", Some(2.0)));
    }

    #[test]
    fn operation_filter_empty_means_wildcard_for_ops() {
        let r = rule_named(&[], Some(100.0));
        assert!(r.matches("any-operation-name", Some(50.0)));
    }
}
