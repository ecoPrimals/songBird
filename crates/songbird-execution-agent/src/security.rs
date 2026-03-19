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
                message: "No auth token configured".to_string(),
                operation: Some("authentication".to_string()),
                required_permission: None,
                context: None,
                remediation: Some("Configure auth token in agent config".to_string()),
            })
        })?;

        let provided = provided_token.ok_or_else(|| {
            SongbirdError::Security(SecurityError {
                message: "No auth token provided".to_string(),
                operation: Some("authentication".to_string()),
                required_permission: Some("valid_auth_token".to_string()),
                context: None,
                remediation: Some("Provide auth token in request".to_string()),
            })
        })?;

        if provided != expected {
            warn!("Invalid authentication token provided");
            return Err(SongbirdError::Security(SecurityError {
                message: "Invalid authentication token".to_string(),
                operation: Some("authentication".to_string()),
                required_permission: Some("valid_auth_token".to_string()),
                context: None,
                remediation: Some("Use correct auth token".to_string()),
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
                message: "Empty command".to_string(),
                field: Some("command".to_string()),
                suggestion: Some("Provide a valid command".to_string()),
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
                    operation: Some("command_validation".to_string()),
                    required_permission: None,
                    context: Some(format!("command: {command}")),
                    remediation: Some(
                        "Avoid dangerous commands that could harm the system".to_string(),
                    ),
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
        let validator = SecurityValidator::new(true, Some("secret123".to_string()));
        assert!(validator.validate_auth(Some("secret123")).is_ok());
    }

    #[test]
    fn test_auth_invalid_token() {
        let validator = SecurityValidator::new(true, Some("secret123".to_string()));
        assert!(validator.validate_auth(Some("wrong")).is_err());
    }

    #[test]
    fn test_auth_missing_token() {
        let validator = SecurityValidator::new(true, Some("secret123".to_string()));
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
