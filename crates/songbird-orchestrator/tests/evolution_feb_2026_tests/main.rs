// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Evolution Tests - February 2026
//!
//! Comprehensive test coverage for the deep debt evolution work:
//! - Sled/JSON serialization (migrated from bincode)
//! - `BirdSong` `family_id` integration
//! - Standard JSON-RPC methods (health, identity, `beacon_exchange`)
//! - Socket discovery `PRIMAL_DEPLOYMENT_STANDARD` compliance
//!
//! Test categories:
//! - Unit tests: Component-level validation
//! - E2E tests: Integrated flow testing
//! - Chaos tests: Resilience under adverse conditions
//! - Fault injection: Error handling verification

mod common;
mod evolution_chaos;
mod evolution_e2e;
mod evolution_fault_injection;
mod evolution_integration;
mod family_id;
mod jsonrpc_methods;
mod protocol_detection;
mod task_serialization;
