// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Resource management related constants

use std::time::Duration;

/// Default resource cleanup interval
pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

/// Default resource timeout
pub const DEFAULT_RESOURCE_TIMEOUT: Duration = Duration::from_secs(60);

/// Default max memory usage percentage
pub const DEFAULT_MAX_MEMORY_USAGE: f64 = 0.8;

/// Default max CPU usage percentage
pub const DEFAULT_MAX_CPU_USAGE: f64 = 0.7;

/// Default leak detection interval
pub const DEFAULT_LEAK_DETECTION_INTERVAL: Duration = Duration::from_secs(600);

/// Default max resource age
pub const DEFAULT_MAX_RESOURCE_AGE: Duration = Duration::from_secs(3600);

/// Default monitoring interval
pub const DEFAULT_MONITORING_INTERVAL: Duration = Duration::from_secs(60);

/// Default tracking interval
pub const DEFAULT_TRACKING_INTERVAL: Duration = Duration::from_secs(10);
