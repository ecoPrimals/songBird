// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Capability-Based Routing E2E Tests
//!
//! Tests for multi-capability service routing and discovery

#![cfg(test)]

#[path = "../../common/mod.rs"]
mod common;

mod affinity_hierarchy;
mod policies_observability;
mod routing;
