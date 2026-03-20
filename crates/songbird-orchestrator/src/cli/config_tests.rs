// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for CLI configuration

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::float_cmp)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::unreadable_literal)]

    use crate::cli::config::CliConfig;
    use songbird_types::{SongbirdError, SongbirdResult};

    #[test]
    fn test_cli_config_default() {
        let config = CliConfig::default();
        assert!(!config.is_verbose());
        assert!(config.is_colored_output_enabled());
        assert!(config.get_config_path().is_none());
    }

    #[test]
    fn test_cli_config_new() {
        let config = CliConfig::new();
        assert!(!config.is_verbose());
        assert!(config.is_colored_output_enabled());
    }

    #[test]
    fn test_cli_config_set_verbose() {
        let mut config = CliConfig::new();
        config.set_verbose(true);
        assert!(config.is_verbose());
    }

    #[test]
    fn test_cli_config_unset_verbose() {
        let mut config = CliConfig::new();
        config.set_verbose(true);
        config.set_verbose(false);
        assert!(!config.is_verbose());
    }

    #[test]
    fn test_cli_config_set_colored_output() -> SongbirdResult<()> {
        let mut config = CliConfig::new();
        config.set_colored_output(false);
        assert!(!config.is_colored_output_enabled());
        Ok(())
    }

    #[test]
    fn test_cli_config_colored_output_default_true() -> SongbirdResult<()> {
        let config = CliConfig::new();
        assert!(config.is_colored_output_enabled());
        Ok(())
    }

    #[test]
    fn test_cli_config_set_config_path() -> SongbirdResult<()> {
        let mut config = CliConfig::new();
        config.set_config_path("/etc/songbird/config.toml".to_string());
        assert!(config.get_config_path().is_some());
        assert_eq!(
            config
                .get_config_path()
                .ok_or_else(|| SongbirdError::configuration("Config path not set".to_string()))?,
            "/etc/songbird/config.toml"
        );
        Ok(())
    }

    #[test]
    fn test_cli_config_no_config_path() -> SongbirdResult<()> {
        let config = CliConfig::new();
        assert!(config.get_config_path().is_none());
        Ok(())
    }

    #[test]
    fn test_cli_config_clone() -> SongbirdResult<()> {
        let mut config1 = CliConfig::new();
        config1.set_verbose(true);
        config1.set_config_path("/path/to/config.toml".to_string());

        let config2 = config1.clone();
        assert_eq!(config1.is_verbose(), config2.is_verbose());
        assert_eq!(config1.get_config_path(), config2.get_config_path());
        Ok(())
    }

    #[test]
    fn test_cli_config_debug_output() -> SongbirdResult<()> {
        let config = CliConfig::new();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CliConfig"));
        Ok(())
    }

    #[test]
    fn test_cli_config_multiple_operations() -> SongbirdResult<()> {
        let mut config = CliConfig::new();
        config.set_verbose(true);
        config.set_colored_output(false);
        config.set_config_path("/custom/path.toml".to_string());

        assert!(config.is_verbose());
        assert!(!config.is_colored_output_enabled());
        assert_eq!(
            config
                .get_config_path()
                .ok_or_else(|| SongbirdError::configuration("Config path not set".to_string()))?,
            "/custom/path.toml"
        );
        Ok(())
    }

    #[test]
    fn test_cli_config_change_config_path() -> SongbirdResult<()> {
        let mut config = CliConfig::new();
        config.set_config_path("/first/path.toml".to_string());
        assert_eq!(
            config
                .get_config_path()
                .ok_or_else(|| SongbirdError::configuration("Config path not set".to_string()))?,
            "/first/path.toml"
        );

        config.set_config_path("/second/path.toml".to_string());
        assert_eq!(
            config
                .get_config_path()
                .ok_or_else(|| SongbirdError::configuration("Config path not set".to_string()))?,
            "/second/path.toml"
        );
        Ok(())
    }

    #[test]
    fn test_cli_config_verbose_toggle() {
        let mut config = CliConfig::new();
        assert!(!config.is_verbose());

        config.set_verbose(true);
        assert!(config.is_verbose());

        config.set_verbose(false);
        assert!(!config.is_verbose());
    }

    #[test]
    fn test_cli_config_colored_output_toggle() {
        let mut config = CliConfig::new();
        assert!(config.is_colored_output_enabled());

        config.set_colored_output(false);
        assert!(!config.is_colored_output_enabled());

        config.set_colored_output(true);
        assert!(config.is_colored_output_enabled());
    }
}
