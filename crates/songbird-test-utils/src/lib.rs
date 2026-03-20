// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
        clippy::await_holding_lock,
        clippy::float_cmp,
        clippy::absurd_extreme_comparisons,
        clippy::nonminimal_bool,
        clippy::needless_collect,
        clippy::used_underscore_binding,
        clippy::overly_complex_bool_expr,
        clippy::assertions_on_constants,
        clippy::unreadable_literal,
        clippy::empty_line_after_doc_comments,
        clippy::field_reassign_with_default,
        clippy::unnecessary_wraps,
        clippy::no_effect_underscore_binding,
        clippy::return_self_not_must_use,
        clippy::duplicated_attributes,
        clippy::needless_pass_by_value,
        clippy::must_use_candidate,
        clippy::doc_markdown,
        clippy::wildcard_imports,
        clippy::enum_glob_use,
        unused_imports,
        unused_variables,
        clippy::unused_self,
        clippy::unnecessary_cast,
        clippy::items_after_test_module,
        clippy::default_trait_access,
        clippy::needless_range_loop,
        clippy::similar_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::unnecessary_unwrap,
        clippy::ignore_without_reason,
        clippy::case_sensitive_file_extension_comparisons,
    )
)]
// Songbird Test Utilities
//
// Canonical testing infrastructure following modernization patterns.
// Provides comprehensive testing capabilities for the Songbird ecosystem.
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::uninlined_format_args,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used
)]

pub mod async_helpers;
pub mod canonical_test_framework;
pub mod chaos_engineering;
pub mod cli_helpers;
pub mod concurrent_helpers; // Modern async testing patterns (Week 1 LiveSpore evolution)
pub mod config_helpers;
pub mod env_isolation;
pub mod error_testing;
pub mod fixtures;
pub mod integration;
pub mod mocks;
pub mod network_fixtures;
pub mod network_mocks;
pub mod performance;
pub mod performance_testing;
pub mod service_fixtures;
pub mod test_env;

// Re-export core testing types (canonical pattern)
pub use canonical_test_framework::{MockService, TestEnvironment};
pub use chaos_engineering::ChaosEngineeringManager;
pub use error_testing::ErrorTestingFramework;
pub use fixtures::{test_bind_address, test_endpoint, test_port, test_socket_addr};
pub use integration::IntegrationTestContext;
pub use network_mocks::NetworkMockManager;
pub use performance_testing::PerformanceTestFramework;
pub use songbird_types::SongbirdError;

// Re-export mocks for test convenience
pub use mocks::{
    HealthStatus,
    MockPrimalServer,
    MockResponse,
    // ✅ REMOVED: Deprecated legacy mock exports (Nov 9, 2025)
    // Use MockCapabilityServer::new(CapabilityType::*) instead
};

// Re-export helper modules
pub use async_helpers::*;
pub use cli_helpers::*;
pub use config_helpers::*;
pub use env_isolation::{ScopedEnv, ScopedEnvMultiple};
pub use network_fixtures::*;
pub use service_fixtures::*;
pub use test_env::*;
