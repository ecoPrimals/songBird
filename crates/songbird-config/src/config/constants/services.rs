// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Service related constants

use std::time::Duration;

/// Default shutdown timeout
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Default startup timeout
pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);

/// Default service check interval
pub const DEFAULT_SERVICE_CHECK_INTERVAL: Duration = Duration::from_secs(15);
