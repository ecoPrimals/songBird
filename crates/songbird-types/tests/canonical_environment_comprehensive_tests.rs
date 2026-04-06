// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for canonical Environment type
//!
//! Focused on edge cases, parsing, and utility methods

use songbird_config::canonical::environment::Environment;
use std::str::FromStr;

// ============================================================================
// ENVIRONMENT ENUM TESTS
// ============================================================================

#[test]
fn test_environment_default() {
    assert_eq!(Environment::default(), Environment::Development);
}

#[test]
fn test_environment_display() {
    assert_eq!(Environment::Development.to_string(), "development");
    assert_eq!(Environment::Staging.to_string(), "staging");
    assert_eq!(Environment::Production.to_string(), "production");
    assert_eq!(Environment::Testing.to_string(), "testing");
    assert_eq!(Environment::Local.to_string(), "local");
}

#[test]
fn test_environment_from_str_exact() {
    // Test using pattern matching instead of unwrap
    assert!(matches!(Environment::from_str("development"), Ok(Environment::Development)));
    assert!(matches!(Environment::from_str("staging"), Ok(Environment::Staging)));
    assert!(matches!(Environment::from_str("production"), Ok(Environment::Production)));
    assert!(matches!(Environment::from_str("testing"), Ok(Environment::Testing)));
    assert!(matches!(Environment::from_str("local"), Ok(Environment::Local)));
}

#[test]
fn test_environment_from_str_aliases() {
    // Test using pattern matching instead of unwrap
    assert!(matches!(Environment::from_str("dev"), Ok(Environment::Development)));
    assert!(matches!(Environment::from_str("stage"), Ok(Environment::Staging)));
    assert!(matches!(Environment::from_str("prod"), Ok(Environment::Production)));
    assert!(matches!(Environment::from_str("test"), Ok(Environment::Testing)));
}

#[test]
fn test_environment_from_str_case_insensitive() {
    // Test using pattern matching instead of unwrap
    assert!(matches!(Environment::from_str("DEVELOPMENT"), Ok(Environment::Development)));
    assert!(matches!(Environment::from_str("Production"), Ok(Environment::Production)));
    assert!(matches!(Environment::from_str("STAGING"), Ok(Environment::Staging)));
    assert!(matches!(Environment::from_str("TEST"), Ok(Environment::Testing)));
    assert!(matches!(Environment::from_str("LOCAL"), Ok(Environment::Local)));
}

#[test]
fn test_environment_from_str_mixed_case() {
    // Test error case using pattern matching
    assert!(matches!(
        Environment::from_str("DeVeL opment"),
        Err(ref e) if e == "Unknown environment: DeVeL opment"
    ));
    assert_eq!(
        Environment::from_str("PrOd").expect("should parse valid environment"),
        Environment::Production
    );
}

#[test]
fn test_environment_from_str_invalid() {
    assert!(Environment::from_str("invalid").is_err());
    assert!(Environment::from_str("").is_err());
    assert!(Environment::from_str("devv").is_err());
    assert!(Environment::from_str("prod123").is_err());
}

#[test]
fn test_environment_from_str_whitespace() {
    // Should fail because we don't trim
    assert!(Environment::from_str(" development").is_err());
    assert!(Environment::from_str("production ").is_err());
    assert!(Environment::from_str(" prod ").is_err());
}

// ============================================================================
// ENVIRONMENT UTILITY METHODS
// ============================================================================

#[test]
fn test_is_production() {
    assert!(Environment::Production.is_production());
    assert!(!Environment::Development.is_production());
    assert!(!Environment::Staging.is_production());
    assert!(!Environment::Testing.is_production());
    assert!(!Environment::Local.is_production());
}

#[test]
fn test_is_development() {
    assert!(Environment::Development.is_development());
    assert!(Environment::Local.is_development());
    assert!(!Environment::Production.is_development());
    assert!(!Environment::Staging.is_development());
    assert!(!Environment::Testing.is_development());
}

#[test]
fn test_enable_debug() {
    assert!(Environment::Development.enable_debug());
    assert!(Environment::Testing.enable_debug());
    assert!(Environment::Local.enable_debug());
    assert!(!Environment::Production.enable_debug());
    assert!(!Environment::Staging.enable_debug());
}

#[test]
fn test_log_level() {
    assert_eq!(Environment::Development.log_level(), "debug");
    assert_eq!(Environment::Local.log_level(), "debug");
    assert_eq!(Environment::Testing.log_level(), "info");
    assert_eq!(Environment::Staging.log_level(), "info");
    assert_eq!(Environment::Production.log_level(), "warn");
}

// ============================================================================
// ENVIRONMENT DETECT TESTS
// ============================================================================

#[test]
fn test_detect_with_songbird_env() {
    let env = Environment::detect_with(|k| {
        if k == "SONGBIRD_ENV" {
            Ok("production".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(env, Environment::Production);
}

#[test]
fn test_detect_with_environment_fallback() {
    let env = Environment::detect_with(|k| {
        if k == "ENVIRONMENT" {
            Ok("staging".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(env, Environment::Staging);
}

#[test]
fn test_detect_defaults_to_development() {
    let env = Environment::detect_with(|_| Err(std::env::VarError::NotPresent));
    assert_eq!(env, Environment::Development);
}

#[test]
fn test_detect_invalid_value_defaults() {
    let env = Environment::detect_with(|k| {
        if k == "SONGBIRD_ENV" {
            Ok("invalid_env".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(env, Environment::Development);
}

// ============================================================================
// ENVIRONMENT COMBINATIONS
// ============================================================================

#[test]
fn test_production_characteristics() {
    let env = Environment::Production;
    assert!(env.is_production());
    assert!(!env.is_development());
    assert!(!env.enable_debug());
    assert_eq!(env.log_level(), "warn");
    assert_eq!(env.to_string(), "production");
}

#[test]
fn test_development_characteristics() {
    let env = Environment::Development;
    assert!(!env.is_production());
    assert!(env.is_development());
    assert!(env.enable_debug());
    assert_eq!(env.log_level(), "debug");
    assert_eq!(env.to_string(), "development");
}

#[test]
fn test_staging_characteristics() {
    let env = Environment::Staging;
    assert!(!env.is_production());
    assert!(!env.is_development());
    assert!(!env.enable_debug());
    assert_eq!(env.log_level(), "info");
    assert_eq!(env.to_string(), "staging");
}

#[test]
fn test_testing_characteristics() {
    let env = Environment::Testing;
    assert!(!env.is_production());
    assert!(!env.is_development());
    assert!(env.enable_debug());
    assert_eq!(env.log_level(), "info");
    assert_eq!(env.to_string(), "testing");
}

#[test]
fn test_local_characteristics() {
    let env = Environment::Local;
    assert!(!env.is_production());
    assert!(env.is_development()); // Local is considered development
    assert!(env.enable_debug());
    assert_eq!(env.log_level(), "debug");
    assert_eq!(env.to_string(), "local");
}

// ============================================================================
// SERIALIZATION / CLONE / EQ TESTS
// ============================================================================

#[test]
fn test_environment_clone() {
    let env1 = Environment::Production;
    let env2 = env1; // Copy
    assert_eq!(env1, env2);
}

#[test]
fn test_environment_equality() {
    assert_eq!(Environment::Production, Environment::Production);
    assert_ne!(Environment::Production, Environment::Development);
    assert_eq!(Environment::Development, Environment::Development);
}

#[test]
fn test_environment_debug() {
    let env = Environment::Production;
    let debug_str = format!("{:?}", env);
    assert!(debug_str.contains("Production"));
}

// ============================================================================
// ERROR MESSAGES
// ============================================================================

#[test]
fn test_from_str_error_messages() {
    let result = Environment::from_str("unknown");
    assert!(result.is_err());
    let err = result.expect_err("should be error for unknown environment");
    assert!(err.contains("Unknown environment"));
    assert!(err.contains("unknown"));
}

#[test]
fn test_from_str_empty_error() {
    let result = Environment::from_str("");
    assert!(result.is_err());
    assert!(
        result.expect_err("should be error for empty string").contains("Unknown environment: ")
    );
}

// ============================================================================
// MATCH EXHAUSTIVENESS
// ============================================================================

#[test]
fn test_all_variants_covered() {
    // Ensure all variants are tested
    let variants = vec![
        Environment::Development,
        Environment::Staging,
        Environment::Production,
        Environment::Testing,
        Environment::Local,
    ];

    for variant in variants {
        // Should not panic
        let _ = variant.to_string();
        let _ = variant.log_level();
        let _ = variant.is_production();
        let _ = variant.is_development();
        let _ = variant.enable_debug();
    }
}
