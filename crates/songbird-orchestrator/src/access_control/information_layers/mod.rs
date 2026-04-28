// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Information layer builders for graduated disclosure.
//!
//! Five layers of progressively detailed task information:
//! - **Public** (L0): status + completion time
//! - **Educational** (L1): sharding, anonymized topology, learning notes
//! - **Operational** (L2): node health, failure context
//! - **Administrative** (L3): node identity, utilization metrics
//! - **Infrastructure** (L4): full node details including IPs

mod builders;
mod types;

pub use types::*;

#[cfg(test)]
mod tests;
