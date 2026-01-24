//! Domain types for connection management

use serde::{Deserialize, Serialize};
use songbird_types::TrustLevel;
use std::time::SystemTime;

/// Metadata about a peer connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerMetadata {
    pub peer_id: String,
    pub endpoint: String,
    pub trust_level: TrustLevel,
    pub discovery_method: String,
    pub capabilities: Vec<String>,
    #[serde(with = "systemtime_as_secs")]
    pub established_at: SystemTime,
}

/// SystemTime serialization helper
///
/// Serializes SystemTime as seconds since UNIX_EPOCH for JSON compatibility
pub mod systemtime_as_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}
