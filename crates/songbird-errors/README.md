# Songbird Errors

[![Crates.io](https://img.shields.io/crates/v/songbird-errors.svg)](https://crates.io/crates/songbird-errors)
[![Documentation](https://docs.rs/songbird-errors/badge.svg)](https://docs.rs/songbird-errors)

Unified error handling system for the Songbird Universal Orchestrator ecosystem, providing AI-first error responses with rich context and automation hints.

## Features

- **AI-First Error Responses**: Rich error context with automation hints and suggested actions
- **Zero Production Crashes**: Eliminates panic-prone patterns with graceful error handling
- **Ecosystem Compliance**: Full compatibility with EcoPrimals AI-First Citizen API standard
- **Performance Optimized**: Zero-cost abstractions with compile-time error handling
- **Professional Patterns**: World-class deprecation and migration support

## Usage

```rust
use songbird_errors::{SongbirdError, SongbirdResult, success};

// Modern error handling with rich context
let result = operation.into_result().map_err(|e| {
    SongbirdError::Network {
        message: format!("Operation failed: {:?}", e),
        operation: Some("data_processing".to_string()),
        suggestion: Some("Check network connectivity".to_string()),
    }
})?;

// AI-First response creation
let response = success(data);
```

## Error Types

- `SongbirdError::Network` - Network and communication errors
- `SongbirdError::Config` - Configuration and validation errors  
- `SongbirdError::Service` - Service discovery and routing errors
- `SongbirdError::Security` - Authentication and authorization errors
- `SongbirdError::Federation` - Cluster and federation errors

## AI-First Features

- **Automation Hints**: Suggested automated recovery actions
- **Confidence Scores**: Error certainty for AI decision making
- **Rich Context**: Structured debugging information
- **Suggested Actions**: Next steps for both AI and human operators

## License

Licensed under the same terms as the Songbird Universal Orchestrator project. 