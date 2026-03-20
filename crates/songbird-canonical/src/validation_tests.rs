// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unit tests for canonical validation types

#[cfg(test)]
mod tests {
    #![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]

    use crate::validation::*;
    use songbird_types::{SongbirdError, SongbirdResult};

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_failure() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = ValidationResult::failure(errors.clone());

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.errors, errors);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_clone() {
        let result1 = ValidationResult::success();
        let result2 = result1.clone();

        assert_eq!(result1.is_valid, result2.is_valid);
        assert_eq!(result1.errors.len(), result2.errors.len());
        assert_eq!(result1.warnings.len(), result2.warnings.len());
    }

    #[test]
    fn test_validation_result_with_warnings() {
        let mut result = ValidationResult::success();
        result.warnings.push("Warning 1".to_string());
        result.warnings.push("Warning 2".to_string());

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 2);
    }

    #[test]
    fn test_validation_result_multiple_errors() {
        let errors = vec![
            "Invalid configuration".to_string(),
            "Missing required field".to_string(),
            "Type mismatch".to_string(),
        ];
        let result = ValidationResult::failure(errors.clone());

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 3);
        for (i, error) in result.errors.iter().enumerate() {
            assert_eq!(error, &errors[i]);
        }
    }

    #[test]
    fn test_validation_result_empty_errors() {
        let result = ValidationResult::failure(Vec::new());

        assert!(!result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_result_debug() {
        let result = ValidationResult::success();
        let debug_str = format!("{result:?}");

        assert!(debug_str.contains("ValidationResult"));
        assert!(debug_str.contains("is_valid"));
    }

    #[test]
    fn test_validation_result_serialization() -> SongbirdResult<()> {
        let result = ValidationResult::success();
        let json_str = serde_json::to_string(&result).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
        assert!(json_str.contains("is_valid"));
        assert!(json_str.contains("true"));
        Ok(())
    }

    #[test]
    fn test_validation_result_deserialization() -> SongbirdResult<()> {
        let json = r#"{"is_valid":true,"errors":[],"warnings":[]}"#;
        let val_result: ValidationResult = serde_json::from_str(json).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
        assert!(val_result.is_valid);
        assert!(val_result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_validation_result_with_both_errors_and_warnings() {
        let mut result = ValidationResult::failure(vec!["Critical error".to_string()]);
        result.warnings.push("Minor warning".to_string());

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validation_result_large_error_list() {
        let errors: Vec<String> = (0..100).map(|i| format!("Error {i}")).collect();
        let result = ValidationResult::failure(errors);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 100);
    }

    #[test]
    fn test_validation_result_unicode_errors() {
        let errors = vec![
            "Error with unicode: 日本語".to_string(),
            "Error with emoji: 🚀".to_string(),
            "Error with greek: Ελληνικά".to_string(),
        ];
        let result = ValidationResult::failure(errors);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 3);
        assert!(result.errors[0].contains("日本語"));
        assert!(result.errors[1].contains("🚀"));
    }

    #[test]
    fn test_validation_result_special_characters() {
        let errors = vec![
            r#"Error with quotes: "test""#.to_string(),
            r"Error with backslash: \".to_string(),
            "Error with newline: \n".to_string(),
        ];
        let result = ValidationResult::failure(errors);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 3);
    }

    #[test]
    fn test_validation_result_empty_string_error() {
        let errors = vec![String::new()];
        let result = ValidationResult::failure(errors);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].is_empty());
    }

    #[test]
    fn test_validation_result_very_long_error() {
        let long_error = "x".repeat(10000);
        let errors = vec![long_error];
        let result = ValidationResult::failure(errors);

        assert!(!result.is_valid);
        assert_eq!(result.errors[0].len(), 10000);
    }
}
