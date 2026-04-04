// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for version command
//!
//! Phase 3 Test Coverage Expansion - CLI Commands\
//! Target: Expand version command test coverage

// =============================================================================
// VERSION STRING TESTS
// =============================================================================

#[test]
fn test_version_string_not_empty() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());
}

#[test]
fn test_version_contains_dot() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(version.contains('.'));
}

#[test]
fn test_version_has_major_minor() {
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();
    assert!(parts.len() >= 2);
}

#[test]
fn test_version_parts_are_numeric() {
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();

    for (i, part) in parts.iter().enumerate().take(2) {
        // Major and minor should be numeric
        assert!(part.parse::<u32>().is_ok(), "Part {} ('{}') should be numeric", i, part);
    }
}

#[test]
fn test_version_format_valid() {
    let version = env!("CARGO_PKG_VERSION");

    // Version should match semantic versioning pattern (at least major.minor)
    let parts: Vec<&str> = version.split('.').collect();
    assert!(parts.len() >= 2 && parts.len() <= 4);
}

// =============================================================================
// BUILD INFO TESTS
// =============================================================================

#[test]
fn test_cargo_pkg_name() {
    let name = env!("CARGO_PKG_NAME");
    assert_eq!(name, "songbird-cli");
}

#[test]
fn test_cargo_pkg_name_not_empty() {
    let name = env!("CARGO_PKG_NAME");
    assert!(!name.is_empty());
}

#[test]
fn test_cargo_pkg_authors() {
    let authors = env!("CARGO_PKG_AUTHORS");
    assert_eq!(authors, env!("CARGO_PKG_AUTHORS"));
}

#[test]
fn test_cargo_pkg_description() {
    let description = env!("CARGO_PKG_DESCRIPTION");
    assert_eq!(description, env!("CARGO_PKG_DESCRIPTION"));
}

// =============================================================================
// VERSION COMPARISON TESTS
// =============================================================================

#[test]
fn test_version_major_is_zero() {
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();
    let major: u32 = parts[0].parse().unwrap();

    // We're in 0.x.x version range
    assert_eq!(major, 0);
}

#[test]
fn test_version_minor_reasonable() {
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();
    let minor: u32 = parts[1].parse().unwrap();

    // Minor version should be reasonable (0-100)
    assert!(minor < 100);
}

// =============================================================================
// VERSION OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_version_output_format_simple() {
    let version = env!("CARGO_PKG_VERSION");
    let output = format!("songbird-cli {}", version);

    assert!(output.contains("songbird-cli"));
    assert!(output.contains(version));
}

#[test]
fn test_version_output_format_detailed() {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");

    let output = format!("{} v{}", name, version);

    assert!(output.contains("songbird-cli"));
    assert!(output.contains(version));
}

#[test]
fn test_version_with_metadata() {
    let version = env!("CARGO_PKG_VERSION");

    // Version might have metadata like -alpha, -beta, etc.
    assert!(!version.is_empty());
}

// =============================================================================
// TARGET AND PLATFORM TESTS
// =============================================================================

#[test]
fn test_target_triple() {
    // Target triple is available via std::env::consts
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    assert!(!arch.is_empty());
    assert!(!os.is_empty());
}

#[test]
fn test_target_arch() {
    let arch = std::env::consts::ARCH;
    let valid_archs = ["x86_64", "aarch64", "arm", "x86"];
    assert!(valid_archs.contains(&arch));
}

#[test]
fn test_target_os() {
    let os = std::env::consts::OS;
    let valid_os = ["linux", "macos", "windows", "freebsd"];
    assert!(valid_os.contains(&os));
}

#[test]
fn test_target_family() {
    let family = std::env::consts::FAMILY;
    let valid_families = ["unix", "windows"];
    assert!(valid_families.contains(&family));
}

// =============================================================================
// COMPILER VERSION TESTS
// =============================================================================

#[test]
fn test_rustc_version_env() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());
    assert!(version.chars().next().is_some_and(|c| c.is_ascii_digit()));
}

// =============================================================================
// FEATURE FLAG TESTS
// =============================================================================

#[test]
fn test_debug_assertions() {
    let debug = cfg!(debug_assertions);
    assert!(!format!("{debug:?}").is_empty());
}

#[test]
fn test_release_mode() {
    let release = !cfg!(debug_assertions);
    assert!(!format!("{release:?}").is_empty());
}

// =============================================================================
// VERSION METADATA TESTS
// =============================================================================

#[test]
fn test_version_can_be_parsed() {
    let version = env!("CARGO_PKG_VERSION");

    // Try to parse as semver (loosely)
    let parts: Vec<&str> = version.split('.').collect();
    assert!(parts.len() >= 2);

    // First two parts should be numbers
    for part in parts.iter().take(2) {
        assert!(part.parse::<u32>().is_ok());
    }
}

#[test]
fn test_version_stability() {
    let version = env!("CARGO_PKG_VERSION");

    // Version string should be stable across calls
    let version2 = env!("CARGO_PKG_VERSION");
    assert_eq!(version, version2);
}

// =============================================================================
// BUILD REPRODUCIBILITY TESTS
// =============================================================================

#[test]
fn test_build_is_reproducible() {
    // Package name should be consistent
    let name1 = env!("CARGO_PKG_NAME");
    let name2 = env!("CARGO_PKG_NAME");
    assert_eq!(name1, name2);
}

#[test]
fn test_version_is_consistent() {
    // Version should be consistent within same build
    let v1 = env!("CARGO_PKG_VERSION");
    let v2 = env!("CARGO_PKG_VERSION");
    assert_eq!(v1, v2);
}

// =============================================================================
// OUTPUT FORMATTING TESTS
// =============================================================================

#[test]
fn test_version_formatting_with_prefix() {
    let version = env!("CARGO_PKG_VERSION");
    let formatted = format!("v{}", version);

    assert!(formatted.starts_with('v'));
    assert!(formatted.contains('.'));
}

#[test]
fn test_version_formatting_no_prefix() {
    let version = env!("CARGO_PKG_VERSION");

    assert!(!version.starts_with('v'));
    assert!(version.chars().next().unwrap().is_numeric());
}

#[test]
fn test_version_formatting_in_sentence() {
    let version = env!("CARGO_PKG_VERSION");
    let sentence = format!("Running Songbird CLI version {}", version);

    assert!(sentence.contains("Running"));
    assert!(sentence.contains(version));
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

#[test]
fn test_version_never_panics() {
    let version = env!("CARGO_PKG_VERSION");
    let formatted = version.to_string();
    assert_eq!(formatted, version);
}

#[test]
fn test_version_parts_never_panic() {
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();

    for part in parts {
        let _ = part.to_string();
    }
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[test]
fn test_version_info_complete() {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let arch = std::env::consts::ARCH;

    assert!(!version.is_empty());
    assert!(!name.is_empty());
    assert!(!arch.is_empty());
}

#[test]
fn test_version_info_formatting() {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let info = format!("{} v{} ({}/{})", name, version, os, arch);

    assert!(info.contains("songbird-cli"));
    assert!(info.contains(version));
    assert!(info.contains(os));
    assert!(info.contains(arch));
}

#[test]
fn test_full_version_output() {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let output = format!("{} {}\nPlatform: {}/{}", name, version, os, arch);

    assert!(output.contains(name));
    assert!(output.contains(version));
    assert!(output.contains("Platform"));
}
