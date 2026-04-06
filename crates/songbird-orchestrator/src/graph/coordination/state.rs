// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Coordination validation result types and severity.

use serde::{Deserialize, Serialize};

/// Coordination pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoordinationPattern {
    /// Sequential execution (A → B → C)
    Sequential,
    /// Parallel execution (A → (B1, B2, B3) → C)
    Parallel,
    /// Pipeline execution (streaming data through stages)
    Pipeline,
    /// `MapReduce` pattern (map phase + reduce phase)
    MapReduce,
    /// Hybrid (complex graph with multiple patterns)
    Hybrid,
}

/// Coordination validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationValidationResult {
    /// Is the coordination pattern valid?
    pub valid: bool,

    /// Detected pattern
    pub pattern: CoordinationPattern,

    /// Pattern description
    pub description: String,

    /// Validation issues (errors and warnings)
    pub issues: Vec<CoordinationIssue>,
}

impl CoordinationValidationResult {
    pub(super) const fn new(pattern: CoordinationPattern, description: String) -> Self {
        Self {
            valid: true,
            pattern,
            description,
            issues: Vec::new(),
        }
    }

    pub(crate) fn add_issue(&mut self, issue: CoordinationIssue) {
        if issue.severity == IssueSeverity::Error {
            self.valid = false;
        }
        self.issues.push(issue);
    }
}

/// Coordination validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Human-readable message
    pub message: String,
}

impl CoordinationIssue {
    pub(crate) const fn error(message: String) -> Self {
        Self {
            severity: IssueSeverity::Error,
            message,
        }
    }

    pub(crate) const fn warning(message: String) -> Self {
        Self {
            severity: IssueSeverity::Warning,
            message,
        }
    }
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Resource availability check result
pub struct ResourceCheck {
    pub feasible: bool,
    pub reason: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::{
        CoordinationIssue, CoordinationPattern, CoordinationValidationResult, IssueSeverity,
        ResourceCheck,
    };

    #[test]
    fn coordination_pattern_serde_roundtrip() {
        for p in [
            CoordinationPattern::Sequential,
            CoordinationPattern::Parallel,
            CoordinationPattern::Pipeline,
            CoordinationPattern::MapReduce,
            CoordinationPattern::Hybrid,
        ] {
            let j = serde_json::to_string(&p).unwrap();
            let back: CoordinationPattern = serde_json::from_str(&j).unwrap();
            assert_eq!(p, back);
        }
    }

    #[test]
    fn issue_severity_serde_roundtrip() {
        for s in [IssueSeverity::Error, IssueSeverity::Warning, IssueSeverity::Info] {
            let j = serde_json::to_string(&s).unwrap();
            let back: IssueSeverity = serde_json::from_str(&j).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn coordination_validation_result_serde_roundtrip() {
        let v = CoordinationValidationResult {
            valid: false,
            pattern: CoordinationPattern::Hybrid,
            description: "desc".to_string(),
            issues: vec![
                CoordinationIssue {
                    severity: IssueSeverity::Error,
                    message: "e".to_string(),
                },
                CoordinationIssue {
                    severity: IssueSeverity::Warning,
                    message: "w".to_string(),
                },
            ],
        };
        let j = serde_json::to_string(&v).unwrap();
        let back: CoordinationValidationResult = serde_json::from_str(&j).unwrap();
        assert_eq!(v.valid, back.valid);
        assert_eq!(v.pattern, back.pattern);
        assert_eq!(v.description, back.description);
        assert_eq!(v.issues.len(), back.issues.len());
    }

    #[test]
    fn coordination_issue_constructors_match_fields() {
        let e = CoordinationIssue {
            severity: IssueSeverity::Error,
            message: "m".to_string(),
        };
        assert_eq!(e.severity, IssueSeverity::Error);
        let w = CoordinationIssue {
            severity: IssueSeverity::Warning,
            message: "m2".to_string(),
        };
        assert_eq!(w.severity, IssueSeverity::Warning);
    }

    #[test]
    fn resource_check_fields() {
        let ok = ResourceCheck {
            feasible: true,
            reason: "ok".to_string(),
        };
        assert!(ok.feasible);
        let bad = ResourceCheck {
            feasible: false,
            reason: "no".to_string(),
        };
        assert!(!bad.feasible);
    }

    #[test]
    fn coordination_pattern_exhaustive_match() {
        let p = CoordinationPattern::Hybrid;
        let s = match p {
            CoordinationPattern::Sequential => "seq",
            CoordinationPattern::Parallel => "par",
            CoordinationPattern::Pipeline => "pipe",
            CoordinationPattern::MapReduce => "mr",
            CoordinationPattern::Hybrid => "hybrid",
        };
        assert_eq!(s, "hybrid");
    }

    #[test]
    fn issue_severity_info_distinct() {
        assert_ne!(IssueSeverity::Info, IssueSeverity::Error);
    }

    #[test]
    fn validation_result_new_starts_valid_with_empty_issues() {
        let r = CoordinationValidationResult::new(
            CoordinationPattern::Pipeline,
            "pipeline".to_string(),
        );
        assert!(r.valid);
        assert_eq!(r.pattern, CoordinationPattern::Pipeline);
        assert!(r.issues.is_empty());
    }

    #[test]
    fn add_issue_warning_keeps_valid() {
        let mut r =
            CoordinationValidationResult::new(CoordinationPattern::Sequential, "seq".into());
        r.add_issue(CoordinationIssue::warning("heads up".into()));
        assert!(r.valid);
        assert_eq!(r.issues.len(), 1);
        assert_eq!(r.issues[0].severity, IssueSeverity::Warning);
    }

    #[test]
    fn add_issue_error_invalidates() {
        let mut r = CoordinationValidationResult::new(CoordinationPattern::Hybrid, "h".into());
        r.add_issue(CoordinationIssue::error("blocked".into()));
        assert!(!r.valid);
        assert_eq!(r.issues[0].severity, IssueSeverity::Error);
    }

    #[test]
    fn add_issue_error_after_warning_stays_invalid() {
        let mut r = CoordinationValidationResult::new(CoordinationPattern::Parallel, "p".into());
        r.add_issue(CoordinationIssue::warning("w".into()));
        assert!(r.valid);
        r.add_issue(CoordinationIssue::error("e".into()));
        assert!(!r.valid);
        assert_eq!(r.issues.len(), 2);
    }

    #[test]
    fn coordination_issue_constructors_match_severity() {
        let e = CoordinationIssue::error("x".into());
        let w = CoordinationIssue::warning("y".into());
        assert_eq!(e.severity, IssueSeverity::Error);
        assert_eq!(w.severity, IssueSeverity::Warning);
    }

    #[test]
    fn issue_severity_exhaustive_in_match() {
        let i = IssueSeverity::Info;
        let s = match i {
            IssueSeverity::Error => "e",
            IssueSeverity::Warning => "w",
            IssueSeverity::Info => "i",
        };
        assert_eq!(s, "i");
    }
}
