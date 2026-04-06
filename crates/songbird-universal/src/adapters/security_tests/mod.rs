// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

//! Tests for Security Capability Adapter
//!
//! Separated from security.rs for file size compliance (1000-line policy)

mod auth_and_traits;
mod metrics_extremes_and_combined;
mod metrics_health_core;
mod security_adapter;
mod serialization;
