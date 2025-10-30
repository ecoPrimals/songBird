//! Canonical environment types and configuration
//!
//! Unified environment definitions for deployment and configuration management

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Environment type for deployment configuration
///
/// Unified from multiple definitions across the ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment
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
            Self::Development => write!(f, "development"),
            Self::Staging => write!(f, "staging"),
            Self::Production => write!(f, "production"),
            Self::Testing => write!(f, "testing"),
            Self::Local => write!(f, "local"),
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" | "stage" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            "testing" | "test" => Ok(Self::Testing),
            "local" => Ok(Self::Local),
            _ => Err(format!("Unknown environment: {s}")),
        }
    }
}

impl Environment {
    /// Check if this is a production environment
    #[must_use]
    pub fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }

    /// Check if this is a development environment
    #[must_use]
    pub fn is_development(self) -> bool {
        matches!(self, Self::Development | Self::Local)
    }

    /// Check if this environment should enable debug features
    #[must_use]
    pub fn enable_debug(self) -> bool {
        matches!(self, Self::Development | Self::Testing | Self::Local)
    }

    /// Get the log level for this environment
    #[must_use]
    pub fn log_level(self) -> &'static str {
        match self {
            Self::Development | Self::Local => "debug",
            Self::Testing | Self::Staging => "info",
            Self::Production => "warn",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdResult;
    // Removed unused imports

    #[test]
    fn test_environment_variants() -> SongbirdResult<()> {
        let envs = vec![
            Environment::Development,
            Environment::Staging,
            Environment::Production,
            Environment::Testing,
            Environment::Local,
        ];

        for env in envs {
            let serialized = serde_json::to_string(&env)
                .expect("JSON serialization should succeed");
            let deserialized: Environment = serde_json::from_str(&serialized)
                .expect("JSON deserialization should succeed");
            assert_eq!(env, deserialized);
        }
        Ok(())
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Development.to_string(), "development");
        assert_eq!(Environment::Staging.to_string(), "staging");
        assert_eq!(Environment::Production.to_string(), "production");
    }

    #[test]
    fn test_environment_from_str() {
        assert_eq!("development".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("prod".parse::<Environment>().unwrap(), Environment::Production);
        assert_eq!("staging".parse::<Environment>().unwrap(), Environment::Staging);
    }
}
