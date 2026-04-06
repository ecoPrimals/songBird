// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Genesis ceremony exchange via NFC
//!
//! Implements secure genesis credential exchange with Dark Forest compliance

mod crypto;
mod exchange;
mod types;

pub use exchange::GenesisExchange;
pub use types::GenesisCredentials;
