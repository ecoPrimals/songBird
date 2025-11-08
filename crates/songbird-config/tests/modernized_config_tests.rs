#![allow(deprecated)]
//! Modernized configuration tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! Tests for the modernized configuration system

#[cfg(test)]
mod config_tests {
    use songbird_config::canonical::constants::DEFAULT_CONFIG_PATH;
    use songbird_config::EnvironmentConfig;

    #[test]
    fn test_default_constants() {
        // Test that default constants are properly defined
        assert!(!DEFAULT_CONFIG_PATH.is_empty());
        assert!(DEFAULT_CONFIG_PATH.contains("songbird"));

        let path = std::path::Path::new(DEFAULT_CONFIG_PATH);
        assert!(path.extension().is_some());
    }

    #[test]
    fn test_environment_config() {
        // Test environment config creation with default values
        let env_config = EnvironmentConfig::default();

        // Test that config has reasonable defaults
        assert!(!env_config.environment.is_empty());
        assert!(!env_config.bind_address.is_empty());
        assert!(env_config.connection_timeout_secs > 0);
        assert!(env_config.max_connections > 0);

        // Test connection timeout conversion
        let timeout = env_config.connection_timeout();
        assert!(timeout.as_secs() > 0);
    }
}
