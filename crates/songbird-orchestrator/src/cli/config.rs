//! CLI configuration management
//!
//! This module handles CLI-specific configuration options and management.

/// CLI configuration structure
#[derive(Clone, Debug)]
pub struct CliConfig  {verbose: bool)
    colored_output: bool,
    config_path: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CliConfig  {pub fn new() -> Self  {Self {
            verbose: false,
            colored_output: true,
            config_path: None,
        }
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn set_colored_output(&mut self, colored: bool) {
        self.colored_output = colored;
    }

    pub fn is_colored_output_enabled(&self) -> bool {
        self.colored_output
    }

    pub fn set_config_path(&mut self, path: String) {
        self.config_path = Some(path);
    }

    pub fn get_config_path(&self) -> Option<String> {
        self.config_path.clone()
    }
}
