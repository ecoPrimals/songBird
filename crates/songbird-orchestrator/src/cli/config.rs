// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CLI configuration management
//!
//! This module handles CLI-specific configuration options and management.

/// CLI configuration structure
#[derive(Clone, Debug)]
pub struct CliConfig {
    verbose: bool,
    colored_output: bool,
    config_path: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CliConfig {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            verbose: false,
            colored_output: true,
            config_path: None,
        }
    }

    pub const fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    #[must_use]
    pub const fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub const fn set_colored_output(&mut self, colored: bool) {
        self.colored_output = colored;
    }

    #[must_use]
    pub const fn is_colored_output_enabled(&self) -> bool {
        self.colored_output
    }

    pub fn set_config_path(&mut self, path: String) {
        self.config_path = Some(path);
    }

    #[must_use]
    pub fn get_config_path(&self) -> Option<String> {
        self.config_path.clone()
    }
}
