// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Port Configuration
//!
//! Port range and port-related configuration structures.

#![allow(missing_docs, reason = "simple inclusive port range tuple")]

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

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::PortRange;

    #[test]
    fn default_range_is_valid_span() {
        let p = PortRange::default();
        assert!(p.end >= p.start);
    }

    #[test]
    fn port_range_json_roundtrip() {
        let p = PortRange {
            start: 1000,
            end: 2000,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: PortRange = serde_json::from_str(&json).unwrap();
        assert_eq!(p.start, back.start);
        assert_eq!(p.end, back.end);
    }
}
