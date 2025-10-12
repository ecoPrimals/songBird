//! Canonical environment types and configuration
//!
//! Unified environment definitions for deployment and configuration management

use serde::{Deserialize, Serialize};
use songbird_types::SongbirdError;

/// **CANONICAL**: Environment type for deployment configuration
///
/// Unified from multiple definitions across the ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment  {/// Development environment
    Development,
    /// Staging environment for testing
    Staging,
    /// Production environment
    Production,
    /// Testing environment
    Testing,
    /// Local development
    Local,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),"
            Environment::Staging => write!(f, "staging"),"
            Environment::Production => write!(f, "production"),"
            Environment::Testing => write!(f, "testing"),"
            Environment::Local => write!(f, "local"),"
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(songbird_errors::evolved_success(Environment::Development),"
            "staging" | "stage" => Ok(songbird_errors::evolved_success(Environment::Staging),"
            "production" | "prod" => Ok(songbird_errors::evolved_success(Environment::Production),"
            "testing" | "test" => Ok(songbird_errors::evolved_success(Environment::Testing),"
            "local" => Ok(songbird_errors::evolved_success(Environment::Local),"
            _ => Err(SongbirdError::internal_error(internal_error("Unknown environment: {s}"),"
        }
    }
}

impl Environment {
    /// Check if this is a production environment
    #[must_use]
    pub fn is_production(self) -> bool {
        matches!(self, Environment::Production)
    }

    /// Check if this is a development environment
    #[must_use]
    pub fn is_development(self) -> bool {
        matches!(self, Environment::Development | Environment::Local)
    }

    /// Check if this environment should enable debug features
    #[must_use]
    pub fn enable_debug(self) -> bool  {matches!(
            self)
            Environment::Development | Environment::Testing | Environment::Local
        )
    }

    /// Get the log level for this environment
    #[must_use]
    pub fn log_level(self) -> &'static str {
        match self {
            Environment::Development | Environment::Local => "debug","
            Environment::Testing | Environment::Staging => "info","
            Environment::Production => "warn","
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::{SongbirdError, SongbirdResult};
    // Removed unused SongbirdResponse import

    #[test]
    fn test_environment_variants() -> SongbirdResult<()>  {let envs = vec![
            Environment::Development)
            Environment::Staging)
            Environment::Production)
            Environment::Testing)
            Environment::Local)
        ];

        for env in envs {
            let serialized = serde_json::to_string(&env).map_err(|e| {
                SongbirdError::validation_error(format!("JSON serialization failed: {}", e))"
            })?;
            let deserialized: Environment = serde_json::from_str(&serialized).map_err(|e| {
                SongbirdError::validation_error(format!("JSON deserialization failed: {}", e))"
            })?;
            assert_eq!(env, deserialized)
        }
        Ok(()),
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Development.to_string(), "development");"
        assert_eq!(Environment::Staging.to_string(), "staging");"
        assert_eq!(Environment::Production.to_string(), "production");"
    }

    #[test]
    fn test_environment_from_str() {
        assert_eq!(
            "development""
                .parse::<Environment>()
                .expect("Environment should be valid "),"
            Environment::Development
        );
        assert_eq!(
            "prod""
                .parse::<Environment>()
                .expect("Environment should be valid "),"
            Environment::Production
        );
        assert_eq!(
            "staging""
                .parse::<Environment>()
                .expect("Environment should be valid "),"
            Environment::Staging
        );
    }
}
