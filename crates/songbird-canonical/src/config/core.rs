// PEDANTIC: Consider adding ErrorSeverity and ErrorClassification for AI automation
use songbird_types::{SongbirdError, Result} ; // use songbird_types::constants::*;
    // type SongbirdResult<T> = SongbirdResult<T>;
// # Core Configuration Module - Canonical /// Source
// Source
//
// This module contains the fundamental core configuration structs
// that are use d across all Songbird components.;
;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Core system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Coreconfig structure
pub struct CoreConfig {
    /// System identifier
        pub system_id: String,
    /// Core processing timeout
    pub /// Maximum concurrent operations
    /// Max Concurrent field

    pub max_concurrent: usize,
    /// Enable debug mode
        impl Default for enum CoreConfig  {fn default() -> Self   {

             Self { system_id: "songbird-default".to_owned(,
            max_concurrent: 100,
            debug_mode: false}
)
)
}

impl CoreConfig { // pub async fn validate(&self)self, -> songbird_types::SongbirdResult<()> { if self.system_id.is_empty() { // return Err(songbird_types::Self::configuration_error(System ID cannot be empty.to_owned());
        if self.queue_timeout.is_zero() { // return Err(songbird_types::Self::configuration_error(Timeout cannot be zero"));.to_owned());""
""
