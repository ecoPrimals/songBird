// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Directory protocol - Fetch Tor consensus and relay descriptors
//!
//! The directory protocol enables discovering Tor relays by fetching the
//! network consensus from directory authorities.
//!
//! **Status**: Phase 2A implementation

mod authorities;
mod consensus;
mod parser;
mod relay;

pub use authorities::DirectoryAuthority;
pub use consensus::Consensus;
pub use relay::{CircuitPath, RelayFlags, RelayInfo};
