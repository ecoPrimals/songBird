// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Shared test harness for Songbird: fixtures, mock servers, env isolation, and load/chaos helpers.
//!
//! Import this crate in integration tests to get consistent ports, fake services, and benchmarking
//! utilities without duplicating boilerplate across the workspace.

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
#![warn(missing_docs)]
#![forbid(unsafe_code)]
// `#[expect]` below targets lints that fire only under some toolchain/feature sets.
#![allow(unfulfilled_lint_expectations)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::uninlined_format_args,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "intentional pattern; clippy false positive for this API"
)]

/// Async test utilities (timeouts, polling).
pub mod async_helpers;
/// Canonical [`TestEnvironment`] and assertion helpers.
pub mod canonical_test_framework;
/// Fault-injection configs and [`ChaosEngineeringManager`](chaos_engineering::ChaosEngineeringManager).
pub mod chaos_engineering;
/// CLI parsing and argv builders for command-line integration tests.
pub mod cli_helpers;
/// Concurrent task helpers for stress-style async tests.
pub mod concurrent_helpers; // Modern async testing patterns (Week 1 LiveSpore evolution)
/// Sample configs and loaders for tests that need canonical JSON/TOML snippets.
pub mod config_helpers;
/// Temporarily override environment variables with RAII guards.
pub mod env_isolation;
/// Helpers that assert on [`SongbirdError`] shapes and error chains.
pub mod error_testing;
/// Deterministic ports, bind addresses, and socket addresses for localhost tests.
pub mod fixtures;
/// Cross-crate integration test context wiring.
pub mod integration;
/// Lightweight HTTP/mock primal servers for black-box tests.
pub mod mocks;
/// Pre-built discovery and endpoint fixtures for networking tests.
pub mod network_fixtures;
/// Stub transports and latency simulation for protocol tests.
pub mod network_mocks;
/// Micro-benchmark and load-test harness ([`performance::LoadTester`]).
pub mod performance;
/// Higher-level performance scenario runners built on [`performance`].
pub mod performance_testing;
/// Service lifecycle helpers (start/stop fake daemons).
pub mod service_fixtures;
/// Temp dirs and process-wide test environment setup.
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
