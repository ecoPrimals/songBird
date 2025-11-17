//! CLI-specific type definitions
//!
//! This module provides type definitions specific to the CLI interface

use crate::cli::types::OutputFormat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CLI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Verbose output enabled
    pub verbose: bool,
    /// Quiet mode enabled
    pub quiet: bool,
    /// Output format preference
    pub output_format: OutputFormat,
    /// Configuration file path
    pub config_path: Option<String>,
}

// OutputFormat moved to crate::types module to avoid duplication

/// Command execution context
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Command name
    pub command: String,
    /// Command arguments
    pub args: HashMap<String, String>,
    /// Execution timestamp
    pub timestamp: std::time::SystemTime,
}

/// CLI operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdResult<T> {
    /// Success status
    pub success: bool,
    /// Result data
    pub data: Option<T>,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

impl<T> SongbirdResult<T> {
    /// Create a successful result
    pub fn success(data: T, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            execution_time_ms,
        }
    }

    /// Create an error result
    #[must_use]
    pub fn error(error: String, execution_time_ms: u64) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            execution_time_ms,
        }
    }
}

/// Progress indicator for long-running operations
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    /// Current progress (0-100)
    pub progress: u8,
    /// Status message
    pub message: String,
    /// Estimated time remaining
    pub eta_seconds: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdError;

    #[test]
    fn test_cli_config_default_values() {
        let config = CliConfig {
            verbose: false,
            quiet: false,
            output_format: OutputFormat::Auto,
            config_path: None,
        };

        assert!(!config.verbose);
        assert!(!config.quiet);
        assert_eq!(config.output_format, OutputFormat::Auto);
        assert!(config.config_path.is_none());
    }

    #[test]
    fn test_cli_config_with_values() {
        let config = CliConfig {
            verbose: true,
            quiet: false,
            output_format: OutputFormat::Json,
            config_path: Some("/etc/songbird.toml".to_string()),
        };

        assert!(config.verbose);
        assert!(!config.quiet);
        assert_eq!(config.output_format, OutputFormat::Json);
        assert_eq!(config.config_path, Some("/etc/songbird.toml".to_string()));
    }

    #[test]
    fn test_cli_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = CliConfig {
            verbose: true,
            quiet: false,
            output_format: OutputFormat::Table,
            config_path: Some("config.toml".to_string()),
        };

        let serialized = serde_json::to_string(&config).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
        let deserialized: CliConfig = serde_json::from_str(&serialized).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;

        assert_eq!(config.verbose, deserialized.verbose);
        assert_eq!(config.quiet, deserialized.quiet);
        assert_eq!(config.output_format, deserialized.output_format);
        assert_eq!(config.config_path, deserialized.config_path);
        Ok(())
    }

    #[test]
    fn test_cli_config_clone() {
        let config = CliConfig {
            verbose: true,
            quiet: true,
            output_format: OutputFormat::Yaml,
            config_path: Some("test.toml".to_string()),
        };

        let cloned = config.clone();
        assert_eq!(config.verbose, cloned.verbose);
        assert_eq!(config.quiet, cloned.quiet);
        assert_eq!(config.output_format, cloned.output_format);
        assert_eq!(config.config_path, cloned.config_path);
    }

    #[test]
    fn test_command_context_creation() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), "test".to_string());
        args.insert("value".to_string(), "123".to_string());

        let context = CommandContext {
            command: "init".to_string(),
            args: args.clone(),
            timestamp: std::time::SystemTime::now(),
        };

        assert_eq!(context.command, "init");
        assert_eq!(context.args.get("name"), Some(&"test".to_string()));
        assert_eq!(context.args.get("value"), Some(&"123".to_string()));
    }

    #[test]
    fn test_command_context_clone() {
        let mut args = HashMap::new();
        args.insert("key".to_string(), "value".to_string());

        let context = CommandContext {
            command: "test".to_string(),
            args,
            timestamp: std::time::SystemTime::now(),
        };

        let cloned = context.clone();
        assert_eq!(context.command, cloned.command);
        assert_eq!(context.args, cloned.args);
    }

    #[test]
    fn test_cli_result_success() {
        let result = SongbirdResult::success("test data".to_string(), 100);

        assert!(result.success);
        assert_eq!(result.data, Some("test data".to_string()));
        assert!(result.error.is_none());
        assert_eq!(result.execution_time_ms, 100);
    }

    #[test]
    fn test_cli_result_error() {
        let result: SongbirdResult<String> =
            SongbirdResult::error("Something failed".to_string(), 50);

        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("Something failed".to_string()));
        assert_eq!(result.execution_time_ms, 50);
    }

    #[test]
    fn test_cli_result_with_different_types() {
        let int_result = SongbirdResult::success(42, 10);
        assert_eq!(int_result.data, Some(42));

        let vec_result = SongbirdResult::success(vec![1, 2, 3], 20);
        assert_eq!(vec_result.data, Some(vec![1, 2, 3]));

        let bool_result = SongbirdResult::success(true, 5);
        assert_eq!(bool_result.data, Some(true));
    }

    #[test]
    fn test_cli_result_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let result = SongbirdResult::success(vec!["a", "b", "c"], 150);
        let serialized = serde_json::to_string(&result).map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
        let deserialized: SongbirdResult<Vec<String>> = serde_json::from_str(&serialized)
            .map_err(|e| SongbirdError::configuration(format!("Error: {}", e)))?;

        assert_eq!(result.success, deserialized.success);
        assert_eq!(result.execution_time_ms, deserialized.execution_time_ms);
        Ok(())
    }

    #[test]
    fn test_cli_result_clone() {
        let result = SongbirdResult::success("data".to_string(), 75);
        let cloned = result.clone();

        assert_eq!(result.success, cloned.success);
        assert_eq!(result.data, cloned.data);
        assert_eq!(result.error, cloned.error);
        assert_eq!(result.execution_time_ms, cloned.execution_time_ms);
    }

    #[test]
    fn test_progress_indicator_creation() {
        let progress = ProgressIndicator {
            progress: 50,
            message: "Processing...".to_string(),
            eta_seconds: Some(120),
        };

        assert_eq!(progress.progress, 50);
        assert_eq!(progress.message, "Processing...");
        assert_eq!(progress.eta_seconds, Some(120));
    }

    #[test]
    fn test_progress_indicator_no_eta() {
        let progress = ProgressIndicator {
            progress: 100,
            message: "Complete".to_string(),
            eta_seconds: None,
        };

        assert_eq!(progress.progress, 100);
        assert_eq!(progress.message, "Complete");
        assert!(progress.eta_seconds.is_none());
    }

    #[test]
    fn test_progress_indicator_clone() {
        let progress = ProgressIndicator {
            progress: 75,
            message: "Almost done".to_string(),
            eta_seconds: Some(30),
        };

        let cloned = progress.clone();
        assert_eq!(progress.progress, cloned.progress);
        assert_eq!(progress.message, cloned.message);
        assert_eq!(progress.eta_seconds, cloned.eta_seconds);
    }

    #[test]
    fn test_progress_indicator_boundary_values() {
        let start = ProgressIndicator {
            progress: 0,
            message: "Starting".to_string(),
            eta_seconds: Some(300),
        };
        assert_eq!(start.progress, 0);

        let end = ProgressIndicator {
            progress: 100,
            message: "Done".to_string(),
            eta_seconds: Some(0),
        };
        assert_eq!(end.progress, 100);
    }

    #[test]
    fn test_cli_config_debug() {
        let config = CliConfig {
            verbose: true,
            quiet: false,
            output_format: OutputFormat::Json,
            config_path: None,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CliConfig"));
        assert!(debug_str.contains("verbose"));
    }

    #[test]
    fn test_cli_result_debug() {
        let result = SongbirdResult::success(123, 50);
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("SongbirdResult"));
        assert!(debug_str.contains("success"));
    }
}
