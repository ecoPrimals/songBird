// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Centralized primal name constants.
//!
//! Eliminates raw string literals scattered across production code.
//! Self-knowledge constants are used freely; external primal names
//! are kept here to track dependency on specific primals — prefer
//! capability-based discovery over hardcoded names at runtime.

/// This primal's canonical name (self-knowledge).
pub const SELF_NAME: &str = "songbird";

/// Filesystem directory name for XDG-style paths (`~/.config/songbird/`).
pub const APP_DIR: &str = "songbird";

/// External: crypto/security primal used for BearDog capability discovery.
///
/// Prefer `capability_discovery("crypto")` at runtime over using this
/// constant directly for IPC routing.
pub const BEARDOG: &str = "beardog";

/// External: coordinator / AI orchestration primal.
pub const SQUIRREL: &str = "squirrel";

/// External: compute primal.
pub const TOADSTOOL: &str = "toadstool";

/// External: storage / gateway primal.
pub const NESTGATE: &str = "nestgate";
