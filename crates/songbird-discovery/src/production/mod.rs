// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Production Discovery Implementations
//!
//! This module contains production-ready service discovery implementations
//! that replace all mock and placeholder discovery providers.

pub mod real_service_discovery;

pub use real_service_discovery::ServiceHealthStatus;
