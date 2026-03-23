// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
//! Resource Management & Fairness
//!
//! Implements:
//! - Per-user resource quotas
//! - Fair scheduling algorithm
//! - Admission control
//! - Usage tracking
//!
//! Modern Rust with no unsafe code, capability-based design.

use anyhow::Result;
use serde::{Deserialize, Serialize};

mod admission;
mod fairness;
mod quota;
mod scheduler;
mod tracker;

pub use admission::*;
pub use fairness::*;
pub use quota::*;
pub use scheduler::*;
pub use tracker::*;

/// Resource types that can be managed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Cpu,
    Memory,
    Gpu,
    Network,
    Storage,
}

/// Resource amount (generic across types)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResourceAmount {
    pub value: f64,
    pub unit: ResourceUnit,
}

/// Resource units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceUnit {
    Cores,     // CPU cores
    Megabytes, // Memory/Storage
    Devices,   // GPU devices
    Mbps,      // Network bandwidth
}

impl ResourceAmount {
    #[must_use]
    pub const fn new(value: f64, unit: ResourceUnit) -> Self {
        Self {
            value,
            unit,
        }
    }

    #[must_use]
    pub const fn zero(unit: ResourceUnit) -> Self {
        Self {
            value: 0.0,
            unit,
        }
    }

    /// Add two resource amounts (must have same unit)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn add(&self, other: &Self) -> Result<Self> {
        if self.unit != other.unit {
            anyhow::bail!("Cannot add resources with different units");
        }
        Ok(Self {
            value: self.value + other.value,
            unit: self.unit,
        })
    }

    /// Subtract two resource amounts (must have same unit)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn sub(&self, other: &Self) -> Result<Self> {
        if self.unit != other.unit {
            anyhow::bail!("Cannot subtract resources with different units");
        }
        Ok(Self {
            value: (self.value - other.value).max(0.0),
            unit: self.unit,
        })
    }

    /// Check if this amount is less than or equal to another
    #[must_use]
    pub fn le(&self, other: &Self) -> bool {
        self.unit == other.unit && self.value <= other.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_amount_operations() {
        let a = ResourceAmount::new(10.0, ResourceUnit::Cores);
        let b = ResourceAmount::new(5.0, ResourceUnit::Cores);

        // Addition
        let sum = a.add(&b).unwrap();
        assert_eq!(sum.value, 15.0);

        // Subtraction
        let diff = a.sub(&b).unwrap();
        assert_eq!(diff.value, 5.0);

        // Comparison
        assert!(b.le(&a));
        assert!(!a.le(&b));

        // Different units should error
        let c = ResourceAmount::new(10.0, ResourceUnit::Megabytes);
        assert!(a.add(&c).is_err());
    }

    #[test]
    fn test_resource_amount_underflow_prevention() {
        let a = ResourceAmount::new(5.0, ResourceUnit::Cores);
        let b = ResourceAmount::new(10.0, ResourceUnit::Cores);

        // Should clamp to 0, not go negative
        let diff = a.sub(&b).unwrap();
        assert_eq!(diff.value, 0.0);
    }
}
