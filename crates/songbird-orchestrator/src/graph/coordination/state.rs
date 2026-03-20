// SPDX-License-Identifier: AGPL-3.0-only
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

    pub(super) fn add_issue(&mut self, issue: CoordinationIssue) {
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
    pub(super) const fn error(message: String) -> Self {
        Self {
            severity: IssueSeverity::Error,
            message,
        }
    }

    pub(super) const fn warning(message: String) -> Self {
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
