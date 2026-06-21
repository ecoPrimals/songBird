// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security and validation for remote execution

use songbird_types::{SecurityError, SongbirdError, SongbirdResult};
use tracing::warn;

/// Security validator for execution requests
pub struct SecurityValidator {
    enable_auth: bool,
    auth_token: Option<String>,
}

impl SecurityValidator {
    /// Create a new security validator
    #[must_use]
    pub const fn new(enable_auth: bool, auth_token: Option<String>) -> Self {
        Self {
            enable_auth,
            auth_token,
        }
    }

    /// Validate authentication token
    ///
    /// # Errors
    ///
    /// Returns an error if auth is enabled and token is missing or invalid
    pub fn validate_auth(&self, provided_token: Option<&str>) -> SongbirdResult<()> {
        if !self.enable_auth {
            return Ok(());
        }

        let expected = self.auth_token.as_deref().ok_or_else(|| {
            SongbirdError::Security(SecurityError {
                message: String::from("No auth token configured"),
                operation: Some(String::from("authentication")),
                required_permission: None,
                context: None,
                remediation: Some(String::from("Configure auth token in agent config")),
            })
        })?;

        let provided = provided_token.ok_or_else(|| {
            SongbirdError::Security(SecurityError {
                message: String::from("No auth token provided"),
                operation: Some(String::from("authentication")),
                required_permission: Some(String::from("valid_auth_token")),
                context: None,
                remediation: Some(String::from("Provide auth token in request")),
            })
        })?;

        if provided != expected {
            warn!("Invalid authentication token provided");
            return Err(SongbirdError::Security(SecurityError {
                message: String::from("Invalid authentication token"),
                operation: Some(String::from("authentication")),
                required_permission: Some(String::from("valid_auth_token")),
                context: None,
                remediation: Some(String::from("Use correct auth token")),
            }));
        }

        Ok(())
    }

    /// Validate command for security risks
    ///
    /// # Errors
    ///
    /// Returns an error if command is empty or contains dangerous patterns
    pub fn validate_command(&self, command: &str) -> SongbirdResult<()> {
        // Check for empty command
        if command.trim().is_empty() {
            return Err(SongbirdError::Validation {
                message: String::from("Empty command"),
                field: Some(String::from("command")),
                suggestion: Some(String::from("Provide a valid command")),
            });
        }

        // Check for dangerous patterns (basic check - can be enhanced)
        let dangerous_patterns = [
            "rm -rf /",
            ":(){ :|:& };:", // Fork bomb
            "mkfs",
            "dd if=/dev/zero",
        ];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                warn!("Potentially dangerous command detected: {}", command);
                return Err(SongbirdError::Security(SecurityError {
                    message: format!("Command contains dangerous pattern: {pattern}"),
                    operation: Some(String::from("command_validation")),
                    required_permission: None,
                    context: Some(format!("command: {command}")),
                    remediation: Some(String::from(
                        "Avoid dangerous commands that could harm the system",
                    )),
                }));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_disabled() {
        let validator = SecurityValidator::new(false, None);
        assert!(validator.validate_auth(None).is_ok());
    }

    #[test]
    fn test_auth_valid_token() {
        let validator = SecurityValidator::new(true, Some(String::from("secret123")));
        assert!(validator.validate_auth(Some("secret123")).is_ok());
    }

    #[test]
    fn test_auth_invalid_token() {
        let validator = SecurityValidator::new(true, Some(String::from("secret123")));
        assert!(validator.validate_auth(Some("wrong")).is_err());
    }

    #[test]
    fn test_auth_missing_token() {
        let validator = SecurityValidator::new(true, Some(String::from("secret123")));
        assert!(validator.validate_auth(None).is_err());
    }

    #[test]
    fn test_validate_safe_command() {
        let validator = SecurityValidator::new(false, None);
        assert!(validator.validate_command("echo hello").is_ok());
    }

    #[test]
    fn test_validate_dangerous_command() {
        let validator = SecurityValidator::new(false, None);
        assert!(validator.validate_command("rm -rf /").is_err());
    }

    #[test]
    fn test_validate_empty_command() {
        let validator = SecurityValidator::new(false, None);
        assert!(validator.validate_command("").is_err());
    }
}
