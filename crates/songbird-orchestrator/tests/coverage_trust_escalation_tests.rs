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
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Comprehensive tests for trust escalation and types
//!
//! Covers edge cases not hit by existing tests:
//! - `TrustLevel` Display trait
//! - `TrustLevel` `description()`
//! - `TrustRelationship` `can_perform()` with expiration
//! - `TrustEscalationManager`: role verification edge cases
//! - `TrustEscalationManager`: `get_all_relationships`, `get_trust_level_counts`
//! - `SecurityTrustClient` creation and defaults
//! - `CapabilityProof` verification edge cases
//! - `IdentityProof` verification edge cases
//! - `PeerMetadata` serde

mod trust_escalation;
