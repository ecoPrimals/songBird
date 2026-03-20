// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Unused imports removed for cleaner code
pub mod handlers;
pub mod server;
pub mod state;
pub mod types;

// Re-export canonical types
// pub use handlers::*;
// pub use songbird_types::CanonicalSongbirdConfig;
pub use state::{ApiState, AppState};
pub use types::ApiResponse;
