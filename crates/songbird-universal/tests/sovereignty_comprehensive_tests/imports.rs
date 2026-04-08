// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared imports matching the original `sovereignty_comprehensive_tests` crate root.

pub use songbird_test_utils::network_fixtures::*;
pub use songbird_test_utils::test_discovery_port;
pub use songbird_test_utils::test_federation_port;
pub use songbird_test_utils::test_health_port;
pub use songbird_test_utils::test_orchestrator_port;
pub use songbird_types::{SongbirdError, SongbirdResult};
pub use songbird_universal::sovereignty::types::{
    SecurityCapability, SecurityLevel, SovereigntyLevel,
};
pub use songbird_universal::sovereignty::{
    PathSegment, RoutingPath, SovereigntyAdapterConfig, SovereigntyAwareAdapter,
};
pub use songbird_universal::types::{
    DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo, UniversalRequest,
};
pub use std::collections::HashMap;
pub use std::time::Duration;
