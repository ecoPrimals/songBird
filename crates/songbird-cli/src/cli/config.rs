// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Module imports
// CLI Configuration
//
// Configuration specific to the CLI interface

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
/// CLI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default configuration directory
    pub config_dir: PathBuf,
    /// Default data directory
    pub data_dir: PathBuf,
    /// Default log directory
    pub log_dir: PathBuf,
    /// Preferred editor
    pub editor: Option<String>,
    /// Color output preference
    pub color: bool,
    /// Default deployment type
    pub default_deployment_type: String,
}
impl Default for CliConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home_dir.join(".songbird");

        Self {
            config_dir: config_dir.clone(),
            data_dir: config_dir.join("data"),
            log_dir: config_dir.join("logs"),
            editor: songbird_process_env::var("EDITOR").ok(),
            color: true,
            default_deployment_type: "home-network".to_string(),
        }
    }
}
