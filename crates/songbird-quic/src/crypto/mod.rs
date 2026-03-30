// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC crypto layer: BearDog-delegated packet protection, header protection,
//! and key derivation.

pub mod header_protection;
pub mod initial_keys;
pub mod key_update;
pub mod packet_protection;
pub mod provider;
