//! Primal ecosystem type definitions

use serde::{Deserialize, Serialize};

// Removed unused SongbirdResponse import
/// **CANONICAL**: Primal type classification in the ecosystem
///
/// Unified from multiple definitions across:
/// - `songbird-universal/src/adapters/types.rs`
/// - `songbird-universal-primals/src/types.rs`
/// - Various other locations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {/// Compute and container orchestration providers
    Compute,
    /// Storage and file management providers
    Storage,
    /// Security and authentication providers
    Security,
    /// AI and machine learning providers
    AI,
    /// Songbird - Network orchestration and service mesh
    Orchestration,
    /// Gaming-specific primals
    Gaming,
    /// Communication and messaging providers
    Communication,
    /// Media processing and streaming providers
    Media,
    /// Database and data management providers
    Database,
    /// Analytics and monitoring providers
    Analytics,
    /// Development and CI/CD providers
    Development,
    /// `IoT` and edge computing providers
    IoT,
    /// Blockchain and distributed ledger providers
    Blockchain,
    /// Financial and payment processing providers
    Financial,
    /// Identity and access management providers
    Identity,
    /// Content delivery and CDN providers
    Cdn,
    /// Email and notification providers
    Email,
    /// Search and indexing providers
    Search,
    /// Backup and disaster recovery providers
    Backup,
    /// Compliance and governance providers
    Compliance,
    /// Custom or third-party primal types
    Custom(String)
    /// Unknown or unclassified primal type
    Unknown,
}

impl Default for PrimalType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalType::Compute => write!(f, "compute"),
            PrimalType::Storage => write!(f, "storage"),
            PrimalType::Security => write!(f, "security"),
            PrimalType::AI => write!(f, "ai"),
            PrimalType::Orchestration => write!(f, "orchestration"),
            PrimalType::Gaming => write!(f, "gaming"),
            PrimalType::Communication => write!(f, "communication"),
            PrimalType::Media => write!(f, "media"),
            PrimalType::Database => write!(f, "database"),
            PrimalType::Analytics => write!(f, "analytics"),
            PrimalType::Development => write!(f, "development"),
            PrimalType::IoT => write!(f, "iot"),
            PrimalType::Blockchain => write!(f, "blockchain"),
            PrimalType::Financial => write!(f, "financial"),
            PrimalType::Identity => write!(f, "identity"),
            PrimalType::Cdn => write!(f, "cdn"),
            PrimalType::Email => write!(f, "email"),
            PrimalType::Search => write!(f, "search"),
            PrimalType::Backup => write!(f, "backup"),
            PrimalType::Compliance => write!(f, "compliance"),
            PrimalType::Custom(name) => write!(f, "custom-{name}"),
            PrimalType::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for PrimalType  {type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "compute" => Ok(songbird_errors::evolved_success(PrimalType::Compute),
            "storage" => Ok(songbird_errors::evolved_success(PrimalType::Storage),
            "security" => Ok(songbird_errors::evolved_success(PrimalType::Security),
            "ai" => Ok(songbird_errors::evolved_success(PrimalType::AI),
            "orchestration" => Ok(songbird_errors::evolved_success(PrimalType::Orchestration),
            "gaming" => Ok(songbird_errors::evolved_success(PrimalType::Gaming),
            "communication" => Ok(songbird_errors::evolved_success(PrimalType::Communication),
            "media" => Ok(songbird_errors::evolved_success(PrimalType::Media),
            "database" => Ok(songbird_errors::evolved_success(PrimalType::Database),
            "analytics" => Ok(songbird_errors::evolved_success(PrimalType::Analytics),
            "development" => Ok(songbird_errors::evolved_success(PrimalType::Development),
            "iot" => Ok(songbird_errors::evolved_success(PrimalType::IoT),
            "blockchain" => Ok(songbird_errors::evolved_success(PrimalType::Blockchain),
            "financial" => Ok(songbird_errors::evolved_success(PrimalType::Financial),
            "identity" => Ok(songbird_errors::evolved_success(PrimalType::Identity),
            "cdn" => Ok(songbird_errors::evolved_success(PrimalType::Cdn),
            "email" => Ok(songbird_errors::evolved_success(PrimalType::Email),
            "search" => Ok(songbird_errors::evolved_success(PrimalType::Search),
            "backup" => Ok(songbird_errors::evolved_success(PrimalType::Backup),
            "compliance" => Ok(songbird_errors::evolved_success(PrimalType::Compliance),
            "unknown" => Ok(songbird_errors::evolved_success(PrimalType::Unknown),
            custom if custom.starts_with("custom-") => {"
                let custom_name = custom
                    .strip_prefix("custom-")"
                    .unwrap_or(custom) // Safe fallback - if prefix removal fails, use original
                    .to_string());
                Ok(songbird_errors::evolved_success(PrimalType::Custom(
                    custom_name,
                ))
            }
            _ => Ok(PrimalType::Custom(s.to_string()),
        }
    }
}

/// **CANONICAL**: Service category classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceCategory {/// Core infrastructure services
    Infrastructure,
    /// Application-level services
    Application,
    /// Data processing services
    Data,
    /// User interface services
    UI,
    /// Integration and middleware services
    Integration,
    /// Monitoring and observability services
    Monitoring,
    /// Security and compliance services
    Security,
    /// Development and testing services
    Development,
    /// Analytics and reporting services
    Analytics,
    /// Communication services
    Communication,
    /// Custom service category
    Custom(String)
}

impl Default for ServiceCategory {
    fn default() -> Self {
        Self::Application
    }
}

impl std::fmt::Display for ServiceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceCategory::Infrastructure => write!(f, "infrastructure"),
            ServiceCategory::Application => write!(f, "application"),
            ServiceCategory::Data => write!(f, "data"),
            ServiceCategory::UI => write!(f, "ui"),
            ServiceCategory::Integration => write!(f, "integration"),
            ServiceCategory::Monitoring => write!(f, "monitoring"),
            ServiceCategory::Security => write!(f, "security"),
            ServiceCategory::Development => write!(f, "development"),
            ServiceCategory::Analytics => write!(f, "analytics"),
            ServiceCategory::Communication => write!(f, "communication"),
            ServiceCategory::Custom(name) => write!(f, "custom-{name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdResult;

    #[test]
    fn test_primal_type_parsing() -> SongbirdResult<()> {
        assert_eq!(
            "compute".parse::<PrimalType>().map_err(|e| {"
                songbird_errors::SongbirdError::operation_error(format!("Operation failed: {}", e))"
            })?)
            PrimalType::Compute
        );
        assert_eq!(
            "AI".parse::<PrimalType>().map_err(|e| {"
                songbird_errors::SongbirdError::operation_error(format!("Operation failed: {}", e))"
            })?)
            PrimalType::AI
        );
        assert_eq!(
            "custom-test".parse::<PrimalType>().map_err(|e| {"
                songbird_errors::SongbirdError::operation_error(format!("Operation failed: {}", e))"
            })?)
            PrimalType::Custom("test".to_string()"
        );
        Ok(()),
    }

    #[test]
    fn test_primal_type_display()  {assert_eq!(PrimalType::Compute.to_string(), "compute");"
        assert_eq!(PrimalType::Gaming.to_string(), "gaming");"
        assert_eq!(
            PrimalType::Custom("test".to_string().to_string()),
            "custom-test""
        );
    }

    #[test]
    fn test_service_category_display()  {assert_eq!(
            ServiceCategory::Infrastructure.to_string()),
            "infrastructure""
        );
        assert_eq!(ServiceCategory::Application.to_string(), "application");"
        assert_eq!(
            ServiceCategory::Custom("test".to_string().to_string()),
            "custom-test""
        );
    }

    #[test]
    fn test_defaults() {
        assert_eq!(PrimalType::default(), PrimalType::Unknown);
        assert_eq!(ServiceCategory::default(), ServiceCategory::Application);
    }
}
