// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Remote deployment library for Songbird federation (SSH / HTTP).
//!
//! Used by the `songbird-deploy` binary and the unified `songbird deploy` subcommand.

#![forbid(unsafe_code)]

pub mod http_deploy;

mod deploy;

pub use deploy::{Args, run};
