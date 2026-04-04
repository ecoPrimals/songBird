// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
//! Integration Tests Module
//!
//! This module contains integration tests that test Songbird components
//! with real external dependencies (BearDog, Squirrel, etc.).
//!
//! # Test Organization
//!
//! - `btsp_beardog_integration` - BTSP client tests with live BearDog server
//! - Future: HTTP gateway tests with live external APIs
//! - Future: Multi-primal integration tests with BiomeOS orchestration
//!
//! # Running Integration Tests
//!
//! Integration tests are marked with `#[ignore = "..."]` and require external services:
//!
//! ```bash
//! # Run specific integration test suite
//! cargo test --test btsp_beardog_integration -- --ignored --test-threads=1
//!
//! # Run all integration tests
//! cargo test --tests -- --ignored --test-threads=1
//! ```
//!
//! # Philosophy
//!
//! - **Real Integration**: Test with actual services, not just mocks
//! - **Deep Validation**: Comprehensive test coverage for critical paths
//! - **Modern Patterns**: Async testing with proper cleanup
//! - **Clear Documentation**: Each test explains what it validates

pub mod btsp_beardog_integration;

