// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Distributed Beacon Mesh
//!
//! Every connected node becomes a potential relay for others.
//! The mesh grows organically - Tor is just bootstrap.
//!
//! ## How It Works
//!
//! 1. First device creates Tor onion (HPC beacon)
//! 2. Second device connects via Tor, exchanges addresses
//! 3. Both can now relay for others
//! 4. Third device can connect via EITHER of the first two
//! 5. Mesh keeps growing, Tor becomes fallback only
//!
//! ## Relay Selection Priority
//!
//! 1. Direct P2P (if hole punch succeeded)
//! 2. Family relay with best latency
//! 3. Any family relay available
//! 4. Tor onion (last resort)
//!
//! Submodules: `types` (endpoint model), `beacon` (routing and lifecycle).

mod beacon;
mod types;

pub use beacon::BeaconMesh;
pub use types::{EndpointType, RelayEndpoint};
