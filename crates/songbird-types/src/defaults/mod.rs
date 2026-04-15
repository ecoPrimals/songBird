// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical defaults for Songbird configuration
//!
//! All defaults are defined here to eliminate magic numbers and hardcoded values
//! across the codebase. Prefer runtime discovery over these defaults.

pub mod beacon;
pub mod hosts;
pub mod network;
pub mod paths;
pub mod ports;
pub mod timeouts;
