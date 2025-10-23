//! Comprehensive tests for CLI core functionality

use songbird_types::SongbirdError;
use songbird_cli::cli::core::{errors::*, types::*};
use songbird_cli::cli::types::OutputFormat;
use std::collections::HashMap;

// ============================================================================
// CliConfig Tests
// ============================================================================

#[test]
fn test_cli_config_creation() {
    let config = CliConfig {
        verbose: false,
        quiet: false,
        output_format: OutputFormat::Auto,
        config_path: None,
    };
    
    assert!(!config.verbose);
    assert!(!config.quiet);
    assert_eq!(config.output_format, OutputFormat::Auto);
}

#[test]
fn test_cli_config_verbose() {
    let config = CliConfig {
        verbose: true,
        quiet: false,
        output_format: OutputFormat::Auto,
        config_path: None,
    };
    
    assert!(config.verbose);
}

#[test]
fn test_cli_config_quiet() {
    let config = CliConfig {
        verbose: false,
        quiet: true,
        output_format: OutputFormat::Auto,
        config_path: None,
    };
    
    assert!(config.quiet);
}

#[test]
fn test_cli_config_json_output() {
    let config = CliConfig {
        verbose: false,
        quiet: false,
        output_format: OutputFormat::Json,
        config_path: None,
    };
    
    assert_eq!(config.output_format, OutputFormat::Json);
}

#[test]
fn test_cli_config_yaml_output() {
    let config = CliConfig {
        verbose: false,
        quiet: false,
        output_format: OutputFormat::Yaml,
        config_path: None,
    };
    
    assert_eq!(config.output_format, OutputFormat::Yaml);
}

#[test]
fn test_cli_config_table_output() {
    let config = CliConfig {
        verbose: false,
        quiet: false,
        output_format: OutputFormat::Table,
        config_path: None,
    };
    
    assert_eq!(config.output_format, OutputFormat::Table);
}

#[test]
fn test_cli_config_with_config_path() {
    let config = CliConfig {
        verbose: false,
        quiet: false,
        output_format: OutputFormat::Auto,
        config_path: Some("/path/to/config.toml".to_string()),
    };
    
    assert!(config.config_path.is_some());
    assert_eq!(config.config_path.map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?, "/path/to/config.toml");
}

#[test]
fn test_cli_config_serialization() {
    let config = CliConfig {
        verbose: true,
        quiet: false,
        output_format: OutputFormat::Json,
        config_path: Some("/path/to/config".to_string()),
    };
    
    let serialized = serde_json::to_string(&config);
    assert!(serialized.is_ok());
}

#[test]
fn test_cli_config_deserialization() {
    let config = CliConfig {
        verbose: true,
        quiet: false,
        output_format: OutputFormat::Json,
        config_path: Some("/path/to/config".to_string()),
    };
    
    let serialized = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization { format: Some("JSON".to_string()), message: format!("Serialization failed: {}", e), debug_info: None })?;
    let deserialized: Result<CliConfig, _> = serde_json::from_str(&serialized);
    
    assert!(deserialized.is_ok());
}

#[test]
fn test_cli_config_clone() {
    let config = CliConfig {
        verbose: true,
        quiet: false,
        output_format: OutputFormat::Json,
        config_path: None,
    };
    
    let cloned = config.clone();
    assert_eq!(config.verbose, cloned.verbose);
}

// ============================================================================
// CommandContext Tests
// ============================================================================

#[test]
fn test_command_context_creation() {
    let context = CommandContext {
        command: "test".to_string(),
        args: HashMap::new(),
        timestamp: std::time::SystemTime::now(),
    };
    
    assert_eq!(context.command, "test");
    assert!(context.args.is_empty());
}

#[test]
fn test_command_context_with_args() {
    let mut args = HashMap::new();
    args.insert("arg1".to_string(), "value1".to_string());
    args.insert("arg2".to_string(), "value2".to_string());
    
    let context = CommandContext {
        command: "test".to_string(),
        args,
        timestamp: std::time::SystemTime::now(),
    };
    
    assert_eq!(context.args.len(), 2);
    assert_eq!(context.args.get("arg1").map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?, "value1");
}

#[test]
fn test_command_context_clone() {
    let context = CommandContext {
        command: "test".to_string(),
        args: HashMap::new(),
        timestamp: std::time::SystemTime::now(),
    };
    
    let cloned = context.clone();
    assert_eq!(context.command, cloned.command);
}

#[test]
fn test_command_context_debug() {
    let context = CommandContext {
        command: "test".to_string(),
        args: HashMap::new(),
        timestamp: std::time::SystemTime::now(),
    };
    
    let debug_output = format!("{:?}", context);
    assert!(debug_output.contains("CommandContext"));
}

// ============================================================================
// CliResult Tests
// ============================================================================

#[test]
fn test_cli_result_success() {
    let result: CliResult<String> = crate::cli::core::types::CliResult::success("test data".to_string(), 100);
    
    assert!(result.success);
    assert!(result.data.is_some());
    assert_eq!(result.data.map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?, "test data");
    assert!(result.error.is_none());
}

#[test]
fn test_cli_result_error() {
    let result: CliResult<String> = crate::cli::core::types::CliResult::error("test error".to_string(), 50);
    
    assert!(!result.success);
    assert!(result.data.is_none());
    assert!(result.error.is_some());
    assert_eq!(result.error.map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?, "test error");
}

#[test]
fn test_cli_result_execution_time() {
    let result: CliResult<String> = crate::cli::core::types::CliResult::success("data".to_string(), 250);
    
    assert_eq!(result.execution_time_ms, 250);
}

#[test]
fn test_cli_result_success_serialization() {
    let result: CliResult<String> = crate::cli::core::types::CliResult::success("test".to_string(), 100);
    let serialized = serde_json::to_string(&result);
    
    assert!(serialized.is_ok());
}

#[test]
fn test_cli_result_error_serialization() {
    let result: CliResult<String> = crate::cli::core::types::CliResult::error("error".to_string(), 50);
    let serialized = serde_json::to_string(&result);
    
    assert!(serialized.is_ok());
}

#[test]
fn test_cli_result_clone() {
    let result: CliResult<String> = crate::cli::core::types::CliResult::success("data".to_string(), 100);
    let cloned = result.clone();
    
    assert_eq!(result.success, cloned.success);
}

// ============================================================================
// ProgressIndicator Tests
// ============================================================================

#[test]
fn test_progress_indicator_creation() {
    let progress = ProgressIndicator {
        progress: 50,
        message: "Processing...".to_string(),
        eta_seconds: Some(30),
    };
    
    assert_eq!(progress.progress, 50);
    assert_eq!(progress.message, "Processing...");
    assert_eq!(progress.eta_seconds, Some(30));
}

#[test]
fn test_progress_indicator_zero_progress() {
    let progress = ProgressIndicator {
        progress: 0,
        message: "Starting...".to_string(),
        eta_seconds: None,
    };
    
    assert_eq!(progress.progress, 0);
}

#[test]
fn test_progress_indicator_complete() {
    let progress = ProgressIndicator {
        progress: 100,
        message: "Complete!".to_string(),
        eta_seconds: Some(0),
    };
    
    assert_eq!(progress.progress, 100);
}

#[test]
fn test_progress_indicator_no_eta() {
    let progress = ProgressIndicator {
        progress: 25,
        message: "Working...".to_string(),
        eta_seconds: None,
    };
    
    assert!(progress.eta_seconds.is_none());
}

#[test]
fn test_progress_indicator_clone() {
    let progress = ProgressIndicator {
        progress: 75,
        message: "Almost done".to_string(),
        eta_seconds: Some(10),
    };
    
    let cloned = progress.clone();
    assert_eq!(progress.progress, cloned.progress);
}

// ============================================================================
// CliError Tests
// ============================================================================

#[test]
fn test_cli_error_command_error() {
    let error = CliError::CommandError("test command error".to_string());
    let error_string = format!("{}", error);
    
    assert!(error_string.contains("Command error"));
    assert!(error_string.contains("test command error"));
}

#[test]
fn test_cli_error_config_error() {
    let error = CliError::ConfigError("test config error".to_string());
    let error_string = format!("{}", error);
    
    assert!(error_string.contains("Configuration error"));
    assert!(error_string.contains("test config error"));
}

#[test]
fn test_cli_error_network_error() {
    let error = CliError::NetworkError("test network error".to_string());
    let error_string = format!("{}", error);
    
    assert!(error_string.contains("Network error"));
    assert!(error_string.contains("test network error"));
}

#[test]
fn test_cli_error_serialization_error() {
    let error = CliError::SerializationError("test serialization error".to_string());
    let error_string = format!("{}", error);
    
    assert!(error_string.contains("Serialization error"));
    assert!(error_string.contains("test serialization error"));
}

#[test]
fn test_cli_error_io_error() {
    let error = CliError::IoError("test io error".to_string());
    let error_string = format!("{}", error);
    
    assert!(error_string.contains("IO error"));
    assert!(error_string.contains("test io error"));
}

#[test]
fn test_cli_error_debug() {
    let error = CliError::CommandError("test".to_string());
    let debug_output = format!("{:?}", error);
    
    assert!(debug_output.contains("CommandError"));
}

#[test]
fn test_cli_error_conversion_command() {
    let cli_error = CliError::CommandError("test".to_string());
    let songbird_error: songbird_types::SongbirdError = cli_error.into();
    
    let error_string = format!("{}", songbird_error);
    assert!(error_string.contains("CLI command error"));
}

#[test]
fn test_cli_error_conversion_config() {
    let cli_error = CliError::ConfigError("test".to_string());
    let songbird_error: songbird_types::SongbirdError = cli_error.into();
    
    let error_string = format!("{}", songbird_error);
    assert!(error_string.contains("CLI configuration error"));
}

#[test]
fn test_cli_error_conversion_network() {
    let cli_error = CliError::NetworkError("test".to_string());
    let songbird_error: songbird_types::SongbirdError = cli_error.into();
    
    let error_string = format!("{}", songbird_error);
    assert!(error_string.contains("CLI network error"));
}

#[test]
fn test_cli_error_conversion_serialization() {
    let cli_error = CliError::SerializationError("test".to_string());
    let songbird_error: songbird_types::SongbirdError = cli_error.into();
    
    let error_string = format!("{}", songbird_error);
    assert!(error_string.contains("CLI serialization error"));
}

#[test]
fn test_cli_error_conversion_io() {
    let cli_error = CliError::IoError("test".to_string());
    let songbird_error: songbird_types::SongbirdError = cli_error.into();
    
    let error_string = format!("{}", songbird_error);
    assert!(error_string.contains("CLI IO error"));
}

// ============================================================================
// OutputFormat Tests
// ============================================================================

#[test]
fn test_output_format_default() {
    let format = OutputFormat::default();
    assert_eq!(format, OutputFormat::Auto);
}

#[test]
fn test_output_format_auto() {
    let format = OutputFormat::Auto;
    assert_eq!(format, OutputFormat::Auto);
}

#[test]
fn test_output_format_json() {
    let format = OutputFormat::Json;
    assert_eq!(format, OutputFormat::Json);
}

#[test]
fn test_output_format_yaml() {
    let format = OutputFormat::Yaml;
    assert_eq!(format, OutputFormat::Yaml);
}

#[test]
fn test_output_format_table() {
    let format = OutputFormat::Table;
    assert_eq!(format, OutputFormat::Table);
}

#[test]
fn test_output_format_text() {
    let format = OutputFormat::Text;
    assert_eq!(format, OutputFormat::Text);
}

#[test]
fn test_output_format_plain() {
    let format = OutputFormat::Plain;
    assert_eq!(format, OutputFormat::Plain);
}

#[test]
fn test_output_format_equality() {
    assert_eq!(OutputFormat::Json, OutputFormat::Json);
    assert_ne!(OutputFormat::Json, OutputFormat::Yaml);
}

#[test]
fn test_output_format_debug() {
    let format = OutputFormat::Json;
    let debug_output = format!("{:?}", format);
    
    assert!(debug_output.contains("Json"));
}

#[test]
fn test_output_format_clone() {
    let format = OutputFormat::Json;
    let cloned = format.clone();
    
    assert_eq!(format, cloned);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_cli_workflow_success() {
    let config = CliConfig {
        verbose: true,
        quiet: false,
        output_format: OutputFormat::Json,
        config_path: None,
    };
    
    let mut args = HashMap::new();
    args.insert("action".to_string(), "test".to_string());
    
    let context = CommandContext {
        command: "test-command".to_string(),
        args,
        timestamp: std::time::SystemTime::now(),
    };
    
    let result: CliResult<String> = crate::cli::core::types::CliResult::success("success".to_string(), 100);
    
    assert!(config.verbose);
    assert_eq!(context.command, "test-command");
    assert!(result.success);
}

#[test]
fn test_cli_workflow_error() {
    let config = CliConfig {
        verbose: false,
        quiet: true,
        output_format: OutputFormat::Text,
        config_path: None,
    };
    
    let error = CliError::CommandError("command failed".to_string());
    let result: CliResult<String> = crate::cli::core::types::CliResult::error(format!("{}", error), 50);
    
    assert!(config.quiet);
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[test]
fn test_cli_progress_tracking() {
    let stages = vec![0, 25, 50, 75, 100];
    
    for stage in stages {
        let progress = ProgressIndicator {
            progress: stage,
            message: format!("{}% complete", stage),
            eta_seconds: if stage < 100 { Some((100 - stage) / 10) } else { Some(0) },
        };
        
        assert_eq!(progress.progress, stage);
        assert!(progress.eta_seconds.is_some());
    }
}

#[test]
fn test_cli_config_combinations() {
    let configs = vec![
        (true, false, OutputFormat::Json),
        (false, true, OutputFormat::Yaml),
        (true, true, OutputFormat::Table), // Both verbose and quiet
        (false, false, OutputFormat::Auto),
    ];
    
    for (verbose, quiet, format) in configs {
        let config = CliConfig {
            verbose,
            quiet,
            output_format: format.clone(),
            config_path: None,
        };
        
        assert_eq!(config.verbose, verbose);
        assert_eq!(config.quiet, quiet);
        assert_eq!(config.output_format, format);
    }
}

