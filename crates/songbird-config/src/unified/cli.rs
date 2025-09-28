//! CLI configuration structures

use serde::{Deserialize, Serialize};
use std::env;

/// Unified CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCliConfig  {pub output_format: String,
    pub verbosity: String,
    pub color_output: bool,
    pub pager_enabled: bool,
    pub interactive_mode: bool,
    pub gaming: GamingCliConfig,
}

impl Default for UnifiedCliConfig  {fn default() -> Self  {Self {
            output_format: env::var("SONGBIRD_OUTPUT_FORMAT")"
                .unwrap_or_else(|_| "pretty".to_string(),"
            verbosity: env::var("SONGBIRD_VERBOSITY").unwrap_or_else(|_| "normal".to_string(),"
            color_output: env::var("NO_COLOR").is_err(),"
            pager_enabled: env::var("SONGBIRD_PAGER_ENABLED").is_ok(),"
            interactive_mode: true,
            gaming: GamingCliConfig::default(),
        }
    }
}

/// Gaming CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingCliConfig  {pub enable_gaming_commands: bool,
    pub default_game_session_timeout: u64,
    pub max_concurrent_sessions: usize,
}

impl Default for GamingCliConfig  {fn default() -> Self  {Self {
            enable_gaming_commands: env::var("SONGBIRD_ENABLE_GAMING_CLI").is_ok(),"
            default_game_session_timeout: 300,
            max_concurrent_sessions: 10,
        }
    }
}
