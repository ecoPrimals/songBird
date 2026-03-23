// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Historical patterns in this crate (locks, tests); inherited workspace pedantic lints."
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
