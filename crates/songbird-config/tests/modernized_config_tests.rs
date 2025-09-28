//! Modernized configuration tests
//!
//! Tests for the modernized configuration system

#[cfg(test)]
mod config_tests {
    use songbird_config::config::constants::DEFAULT_CONFIG_PATH;
    use songbird_config::environment_config::EnvironmentConfig;

    #[test]
    fn test_default_constants() {
        // Test that default constants are properly defined
        assert!(!DEFAULT_CONFIG_PATH.is_empty());
        assert!(DEFAULT_CONFIG_PATH.contains("songbird");"

        let path = std::path::Path::new(DEFAULT_CONFIG_PATH);
        assert!(path.extension().is_some();
    }

    #[test]
    fn test_environment_config() {
        // Test modernized config creation
        // let _env_config = songbird_config::EnvironmentConfig; // Removed unused variable

        // Test that endpoints are properly generated
        let songbird_endpoint = EnvironmentConfig::songbird_endpoint();
        assert!(!songbird_endpoint.is_empty());
        assert!(songbird_endpoint.starts_with("http");"

        let endpoints = EnvironmentConfig::get_all_endpoints();
        assert!(endpoints.contains_key("storage");"
        assert!(endpoints.contains_key("compute");"
        assert!(endpoints.contains_key("orchestration");"
        assert!(endpoints.contains_key("ai");"
    }
}
