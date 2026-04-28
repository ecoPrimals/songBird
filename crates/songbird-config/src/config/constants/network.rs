// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network-related constants

use std::time::Duration;

pub use songbird_types::constants::{
    LOCALHOST as DEFAULT_HOST_V4, LOCALHOST_HOSTNAME as DEFAULT_HOST,
};
pub use songbird_types::defaults::ports::{
    DEFAULT_DASHBOARD_PORT, DEFAULT_HTTP_PORT as DEFAULT_DEV_PORT, DEFAULT_ORCHESTRATOR_PORT,
};

#[allow(deprecated, reason = "re-exporting deprecated item for backward compatibility")]
pub use super::bind_and_ports::DEFAULT_BIND_ADDRESS;
pub use songbird_types::constants::PRODUCTION_BIND_ADDRESS;

/// Default retry delay
pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(1000);
