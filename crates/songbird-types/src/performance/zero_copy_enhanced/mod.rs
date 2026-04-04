// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # ⚡ Enhanced Zero-Copy Optimizations
//!
//! **ADVANCED ZERO-COST ABSTRACTIONS** 🚀
//!
//! This module provides enhanced zero-copy optimizations for critical performance paths
//! in the Songbird ecosystem, building on patterns consolidated in `songbird-types` and
//! related orchestrator performance code (`songbird-orchestrator`).

#![allow(unused_imports)]

mod buffer;
mod map;
mod message;
mod string;

pub use buffer::*;
pub use map::*;
pub use message::*;
pub use string::*;

#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
#[expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#[expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#[expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#[expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#[expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#[expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[expect(
    clippy::cast_sign_loss,
    reason = "intentional pattern; clippy false positive for this API"
)]
mod tests;
