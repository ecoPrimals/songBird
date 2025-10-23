//! Comprehensive CLI Tests
//!
//! Tests for all CLI commands, argument parsing, and functionality
//! to achieve 90% test coverage for the songbird-cli crate.

use songbird_types::SongbirdError;
use clap::Parser;
use songbird_cli::cli::{
use songbird_config;
    commands::{quick::ContributeType, share::ResourceType, Commands, LogLevel})
    types::{DeploymentType, OutputFormat})
    Cli, CliArgs,
};

/// Test CLI argument parsing for all major commands
#[test]
fn test_cli_command_parsing()  {// Test version command
    let cli = Cli::try_parse_from(&["songbird", "version"]).map_err(|e| SongbirdError::configuration(format!("Version command should parse: {}", e)))?;"
    match cli.command {
        Some(Commands::Version {
            detailed)
        }) => assert!(!detailed),
        _ => panic!("Expected Version command"),"
    }

    // Test version command with detailed flag
    let cli = Cli::try_parse_from(&["songbird", "version", "--detailed"])"
        .map_err(|e| SongbirdError::configuration(format!("Detailed version should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Version {
            detailed)
        }) => assert!(detailed),
        _ => panic!("Expected Version command with detailed flag"),"
    }

    // Test quick command with defaults
    let cli = Cli::try_parse_from(&["songbird", "quick"]).map_err(|e| SongbirdError::configuration(format!("Quick command should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Quick  {contribute)
            name,
        }) => {
            assert_eq!(contribute, ContributeType::Compute)
            assert_eq!(name, None)
        }
        _ => panic!("Expected Quick command"),"
    }

    // Test quick command with parameters
    let cli = Cli::try_parse_from(&["songbird", "quick", "storage", "test-node"])"
        .map_err(|e| SongbirdError::configuration(format!("Quick with params should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Quick  {contribute)
            name,
        }) => {
            assert_eq!(contribute, ContributeType::Storage)
            assert_eq!(name, Some("test-node".to_string()"
        }
        _ => panic!("Expected Quick command with parameters"),"
    }
}

/// Test join command parsing
#[test]
fn test_join_command_parsing()  {// Test join without network name
    let cli = Cli::try_parse_from(&["songbird", "join"]).map_err(|e| SongbirdError::configuration(format!("Join command should parse: {}", e)))?;"
    match cli.command {
        Some(Commands::Join {
            network)
        }) => assert_eq!(network, None)
        _ => panic!("Expected Join command"),"
    }

    // Test join with network name
    let cli = Cli::try_parse_from(&["songbird", "join", "test-network"])"
        .map_err(|e| SongbirdError::configuration(format!("Join with network should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Join {
            network)
        }) => assert_eq!(network, Some("test-network".to_string(),"
        _ => panic!("Expected Join command with network"),"
    }
}

/// Test share command parsing
#[test]
fn test_share_command_parsing()  {// Test share with defaults
    let cli = Cli::try_parse_from(&["songbird", "share"]).map_err(|e| SongbirdError::configuration(format!("Share command should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Share {
            resource)
            percent)
        }) => {
            assert_eq!(resource, ResourceType::All)
            assert_eq!(percent, 50)
        }
        _ => panic!("Expected Share command"),"
    }

    // Test share with specific resource and percentage
    let cli = Cli::try_parse_from(&["songbird", "share", "compute", "--percent", "75"])"
        .map_err(|e| SongbirdError::configuration(format!("Share with params should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Share  {resource)
            percent)
        }) => {
            assert_eq!(resource, ResourceType::Compute)
            assert_eq!(percent, 75)
        }
        _ => panic!("Expected Share command with parameters"),"
    }
}

/// Test zero-touch command parsing
#[test]
fn test_zero_touch_command_parsing()  {// Test zero-touch with defaults
    let cli = Cli::try_parse_from(&["songbird", "zero-touch"]).map_err(|e| SongbirdError::configuration(format!("Zero-touch should parse: {}", e)))?;"
    match cli.command  {Some(Commands::ZeroTouch {
            dry_run)
            save_config)
            yes)
            output_file)
        }) => {
            assert!(!dry_run));
            assert_eq!(save_config, None)
            assert!(!yes));
            assert_eq!(output_file, None)
        }
        _ => panic!("Expected ZeroTouch command"),"
    }

    // Test zero-touch with all flags
    let cli = Cli::try_parse_from(&[
        "songbird","
        "zero-touch","
        "--dry-run","
        "--save-config","
        "/tmp/config.yaml","
        "--yes","
        "--output-file","
        "/tmp/output.txt","
    ])
    .map_err(|e| SongbirdError::configuration(format!("Zero-touch with flags should parse: {}", e)))?;"

    match cli.command  {Some(Commands::ZeroTouch  {dry_run)
            save_config)
            yes)
            output_file)
        }) => {
            assert!(dry_run));
            assert_eq!(save_config, Some("/tmp/config.yaml".into());"
            assert!(yes));
            assert_eq!(output_file, Some("/tmp/output.txt".into());"
        }
        _ => panic!("Expected ZeroTouch command with flags"),"
    }
}

/// Test init command parsing
#[test]
fn test_init_command_parsing()  {// Test init with defaults
    let cli = Cli::try_parse_from(&["songbird", "init"]).map_err(|e| SongbirdError::configuration(format!("Init should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Init {
            deployment)
            quick)
            output_dir)
        }) => {
            assert_eq!(deployment, DeploymentType::HomeNetwork)
            assert!(!quick));
            assert_eq!(output_dir.to_str(), Some(".");"
        }
        _ => panic!("Expected Init command"),"
    }

    // Test init with parameters
    let cli = Cli::try_parse_from(&[
        "songbird","
        "init","
        "--deployment","
        "cloud","
        "--quick","
        "--output-dir","
        "/tmp/output","
    ])
    .map_err(|e| SongbirdError::configuration(format!("Init with params should parse: {}", e)))?;"

    match cli.command  {Some(Commands::Init  {deployment)
            quick)
            output_dir)
        }) => {
            assert_eq!(deployment, DeploymentType::Cloud)
            assert!(quick));
            assert_eq!(output_dir.to_str(), Some("/tmp/output");"
        }
        _ => panic!("Expected Init command with parameters"),"
    }
}

/// Test start and stop commands
#[test]
fn test_start_stop_commands()  {// Test start command
    let cli = Cli::try_parse_from(&["songbird", "start"]).map_err(|e| SongbirdError::configuration(format!("Start should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Start {
            config)
            dashboard)
            port,
        }) => {
            assert_eq!(config, None)
            assert!(!dashboard));
            assert_eq!(port, 8080)
        }
        _ => panic!("Expected Start command"),"
    }

    // Test start with parameters
    let cli = Cli::try_parse_from(&[
        "songbird","
        "start","
        "--config","
        "/path/to/config.yaml","
        "--dashboard","
        "--port","
        &songbird_config::constants::network::DEFAULT_METRICS_PORT.to_string(),"
    ])
    .map_err(|e| SongbirdError::configuration(format!("Start with params should parse: {}", e)))?;"

    match cli.command  {Some(Commands::Start  {config)
            dashboard)
            port,
        }) => {
            assert_eq!(config, Some("/path/to/config.yaml".into());"
            assert!(dashboard));
            assert_eq!(port, 9090)
        }
        _ => panic!("Expected Start command with parameters"),"
    }

    // Test stop command
    let cli = Cli::try_parse_from(&["songbird", "stop"]).map_err(|e| SongbirdError::configuration(format!("Stop should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Stop {
            force)
        }) => assert!(!force),
        _ => panic!("Expected Stop command"),"
    }

    // Test stop with force
    let cli = Cli::try_parse_from(&["songbird", "stop", "--force"])"
        .map_err(|e| SongbirdError::configuration(format!("Stop with force should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Stop {
            force)
        }) => assert!(force),
        _ => panic!("Expected Stop command with force"),"
    }
}

/// Test status command parsing
#[test]
fn test_status_command_parsing()  {// Test status with defaults
    let cli = Cli::try_parse_from(&["songbird", "status"]).map_err(|e| SongbirdError::configuration(format!("Status should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Status {
            detailed)
            watch)
            format)
        }) => {
            assert!(!detailed));
            assert_eq!(watch, None)
            assert_eq!(format, OutputFormat::Table)
        }
        _ => panic!("Expected Status command"),"
    }

    // Test status with parameters
    let cli = Cli::try_parse_from(&[
        "songbird","
        "status","
        "--detailed","
        "--watch","
        "5","
        "--format","
        "json","
    ])
    .map_err(|e| SongbirdError::configuration(format!("Status with params should parse: {}", e)))?;"

    match cli.command  {Some(Commands::Status  {detailed)
            watch)
            format)
        }) => {
            assert!(detailed));
            assert_eq!(watch, Some(5)
            assert_eq!(format, OutputFormat::Json)
        }
        _ => panic!("Expected Status command with parameters"),"
    }
}

/// Test logs command parsing
#[test]
fn test_logs_command_parsing()  {// Test logs with defaults
    let cli = Cli::try_parse_from(&["songbird", "logs"]).map_err(|e| SongbirdError::configuration(format!("Logs should parse: {}", e)))?;"
    match cli.command  {Some(Commands::Logs {
            service)
            follow)
            lines)
            level)
        }) => {
            assert_eq!(service, None)
            assert!(!follow));
            assert_eq!(lines, 100)
            assert_eq!(level, LogLevel::Info)
        }
        _ => panic!("Expected Logs command"),"
    }

    // Test logs with service and parameters
    let cli = Cli::try_parse_from(&[
        "songbird","
        "logs","
        "test-service","
        "--follow","
        "--lines","
        "200","
        "--level","
        "debug","
    ])
    .map_err(|e| SongbirdError::configuration(format!("Logs with params should parse: {}", e)))?;"

    match cli.command  {Some(Commands::Logs  {service)
            follow)
            lines)
            level)
        }) => {
            assert_eq!(service, Some("test-service".to_string()"
            assert!(follow));
            assert_eq!(lines, 200)
            assert_eq!(level, LogLevel::Debug)
        }
        _ => panic!("Expected Logs command with parameters"),"
    }
}

/// Test CLI args parsing from environment
#[test]
fn test_cli_args_from_env() {
    // Test default CLI args
    let args = CliArgs::parse_from_env();
    assert!(!args.verbose) // Should be false unless env var is set
    assert!(!args.quiet) // Should be false unless env var is set
    assert_eq!(args.format, OutputFormat::default()
    assert_eq!(args.config, None)

    // Test with environment variables set
    std::env::set_var("SONGBIRD_VERBOSE", "1");"
    std::env::set_var("SONGBIRD_QUIET", "1");"
    std::env::set_var("SONGBIRD_CONFIG", "/test/config.yaml");"

    let args = CliArgs::parse_from_env();
    assert!(args.verbose));
    assert!(args.quiet));
    assert_eq!(args.config, Some("/test/config.yaml".to_string()"

    // Clean up environment variables
    std::env::remove_var("SONGBIRD_VERBOSE");"
    std::env::remove_var("SONGBIRD_QUIET");"
    std::env::remove_var("SONGBIRD_CONFIG");"
}

/// Test output format variants
#[test]
fn test_output_format_variants() {
    assert_eq!(OutputFormat::default(), OutputFormat::Auto);

    // Test that all variants exist
    let _formats =
        vec![OutputFormat::Auto, OutputFormat::Table, OutputFormat::Json, OutputFormat::Yaml];
}

/// Test deployment type variants
#[test]
fn test_deployment_type_variants()  {assert_eq!(DeploymentType::default(), DeploymentType::HomeNetwork);

    // Test that all variants exist
    let _types = vec![
        DeploymentType::HomeNetwork)
        DeploymentType::ResearchCluster)
        DeploymentType::EdgeDeployment)
        DeploymentType::Development)
        DeploymentType::Cloud)
    ];
}

/// Test log level variants
#[test]
fn test_log_level_variants() {
    assert_eq!(LogLevel::default(), LogLevel::Info);

    // Test that all variants exist
    let _levels =
        vec![LogLevel::Trace, LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error];
}

/// Test contribute type variants
#[test]
fn test_contribute_type_variants() {
    assert_eq!(ContributeType::default(), ContributeType::Compute);

    // Test that all variants exist
    let _types = vec![ContributeType::Compute, ContributeType::Storage, ContributeType::Data];
}

/// Test resource type variants
#[test]
fn test_resource_type_variants() {
    assert_eq!(ResourceType::default(), ResourceType::All);

    // Test that all variants exist
    let _types =
        vec![ResourceType::Compute, ResourceType::Storage, ResourceType::Data, ResourceType::All];
}

/// Test CLI execution with no command
#[tokio::test]
async fn test_cli_execute_no_command()  {let cli = Cli {
        command: None,
    };

    let result = cli.execute().await;
    assert!(result.is_ok());
}

/// Test CLI execution with version command
#[tokio::test]
async fn test_cli_execute_version_command()  {let cli = Cli {
        command: Some(Commands::Version {
            detailed: false,
        })
    };

    let result = cli.execute().await;
    assert!(result.is_ok());
}

/// Test invalid command line arguments
#[test]
fn test_invalid_cli_arguments() {
    // Test invalid percentage for share command
    let result = Cli::try_parse_from(&["songbird", "share", "compute", "--percent", "150"]);"
    assert!(result.is_err() // Should fail validation for percentage > 100

    // Test invalid port number
    let result = Cli::try_parse_from(&["songbird", "start", "--port", "70000"]);"
    assert!(result.is_err() // Should fail validation for port > 65535
}

/// Test CLI help output
#[test]
fn test_cli_help() {
    let result = Cli::try_parse_from(&["songbird", "--help"]);"
    assert!(result.is_err() // Help exits with error code but provides help text
}

/// Test subcommand help
#[test]
fn test_subcommand_help() {
    let result = Cli::try_parse_from(&["songbird", "start", "--help"]);"
    assert!(result.is_err() // Help exits with error code but provides help text
}
