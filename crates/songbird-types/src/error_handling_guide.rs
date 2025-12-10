//! Error Handling Evolution Guidelines
//!
//! Patterns for converting unwrap/expect to proper error handling

use thiserror::Error;

/// Common error handling patterns for Songbird
///
/// # Evolution Strategy
///
/// 1. **Test Code**: Use `expect()` with descriptive messages
/// 2. **Examples**: Use `expect()` with context
/// 3. **Production**: Use `?` operator with proper error types
/// 4. **Libraries**: Return `Result` types

/// Example: Converting unwrap to proper error handling
pub mod examples {
    use super::*;

    // ❌ BEFORE: Panic-prone
    pub fn bad_example() {
        let config = std::env::var("CONFIG_PATH").unwrap();
        let data = std::fs::read_to_string(config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&data).unwrap();
    }

    // ✅ AFTER: Proper error handling
    pub fn good_example() -> Result<serde_json::Value, ConfigError> {
        let config = std::env::var("CONFIG_PATH")
            .map_err(|_| ConfigError::MissingEnvironmentVariable("CONFIG_PATH"))?;
        
        let data = std::fs::read_to_string(&config)
            .map_err(|e| ConfigError::FileRead { path: config.clone(), source: e })?;
        
        let parsed: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| ConfigError::JsonParse { source: e })?;
        
        Ok(parsed)
    }

    // ✅ TEST CODE: Descriptive expect
    #[cfg(test)]
    mod tests {
        #[test]
        fn test_example() {
            let value = serde_json::json!({"key": "value"});
            let serialized = serde_json::to_string(&value)
                .expect("Serialization of simple JSON should never fail");
            assert!(!serialized.is_empty());
        }
    }
}

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Missing required environment variable
    #[error("Missing required environment variable: {0}")]
    MissingEnvironmentVariable(&'static str),
    
    /// Failed to read configuration file
    #[error("Failed to read config file {path}: {source}")]
    FileRead {
        /// Path to the configuration file
        path: String,
        /// Underlying IO error
        #[source]
        source: std::io::Error,
    },
    
    /// Failed to parse JSON
    #[error("Failed to parse JSON: {source}")]
    JsonParse {
        /// Underlying JSON error
        #[source]
        source: serde_json::Error,
    },
}

/// Pattern: Option handling
pub mod option_patterns {
    use super::*;

    // ❌ BEFORE
    pub fn bad_get(map: &std::collections::HashMap<String, String>, key: &str) -> String {
        map.get(key).unwrap().clone()
    }

    // ✅ AFTER: Return Option
    pub fn good_get(map: &std::collections::HashMap<String, String>, key: &str) -> Option<String> {
        map.get(key).cloned()
    }

    // ✅ AFTER: Return Result with context
    pub fn better_get(map: &std::collections::HashMap<String, String>, key: &str) -> Result<String, KeyNotFound> {
        map.get(key)
            .cloned()
            .ok_or_else(|| KeyNotFound { key: key.to_string() })
    }

    #[derive(Debug, Error)]
    #[error("Key not found: {key}")]
    pub struct KeyNotFound {
        key: String,
    }
}

/// Pattern: Default values
pub mod default_patterns {
    // ❌ BEFORE: Panic on missing value
    pub fn bad_with_default(opt: Option<u32>) -> u32 {
        opt.unwrap_or_else(|| panic!("Missing value"))
    }

    // ✅ AFTER: Sensible default
    pub fn good_with_default(opt: Option<u32>) -> u32 {
        opt.unwrap_or(100) // Sensible default
    }

    // ✅ AFTER: Explicit about fallback
    pub fn better_with_default(opt: Option<u32>) -> u32 {
        opt.unwrap_or_else(|| {
            tracing::warn!("Using default value of 100");
            100
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_example() {
        // Set up test environment
        std::env::set_var("CONFIG_PATH", "/tmp/test_config.json");
        
        // This would fail in real usage, but demonstrates the pattern
        let result = examples::good_example();
        assert!(result.is_err()); // File doesn't exist
        
        std::env::remove_var("CONFIG_PATH");
    }
}

