// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in deployment contexts"
)]
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")
)]
//! Remote deployment library for Songbird federation (SSH / HTTP).
//!
//! Used by the `songbird-deploy` binary and the unified `songbird deploy` subcommand.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// HTTP deployment client, capability negotiation, and adaptive upload helpers.
pub mod http_deploy;

mod deploy;

pub use deploy::{Args, run};
