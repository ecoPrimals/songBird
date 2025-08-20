//! Security levels and authentication type definitions

use serde::{Deserialize, Serialize};

// Removed unused SongbirdResponse import
/// **CANONICAL**: Security level classification
///
/// Unified from multiple definitions across:
/// - `songbird-security/src/types.rs`
/// - `songbird-config/src/security.rs`
/// - `songbird-universal/src/security.rs`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// No security - for development/testing only
    None,
    /// Minimal security - basic validation
    Minimal,
    /// Basic security - standard practices (alias for Minimal)
    Basic,
    /// Low security - minimal encryption
    Low,
    /// Medium security - standard encryption
    Medium,
    /// Standard security - enhanced practices
    Standard,
    /// Public security - standard encryption
    Public,
    /// High security - strong encryption + authentication
    High,
    /// Private security - enhanced encryption + authentication
    Private,
    /// Critical security - maximum security measures
    Critical,
    /// Confidential security - maximum security measures
    Confidential,
    /// Enhanced security - advanced security measures
    Enhanced,
    /// Maximum security - highest available security
    Maximum,
    /// Classified security - government/military grade
    Classified,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Public
    }
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityLevel::None => write!(f, "none"),
            SecurityLevel::Minimal => write!(f, "minimal"),
            SecurityLevel::Basic => write!(f, "basic"),
            SecurityLevel::Low => write!(f, "low"),
            SecurityLevel::Medium => write!(f, "medium"),
            SecurityLevel::Standard => write!(f, "standard"),
            SecurityLevel::Public => write!(f, "public"),
            SecurityLevel::High => write!(f, "high"),
            SecurityLevel::Private => write!(f, "private"),
            SecurityLevel::Critical => write!(f, "critical"),
            SecurityLevel::Confidential => write!(f, "confidential"),
            SecurityLevel::Enhanced => write!(f, "enhanced"),
            SecurityLevel::Maximum => write!(f, "maximum"),
            SecurityLevel::Classified => write!(f, "classified"),
        }
    }
}

impl std::str::FromStr for SecurityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(songbird_errors::evolved_success(SecurityLevel::None)),
            "minimal" => Ok(songbird_errors::evolved_success(SecurityLevel::Minimal)),
            "basic" => Ok(songbird_errors::evolved_success(SecurityLevel::Basic)),
            "low" => Ok(songbird_errors::evolved_success(SecurityLevel::Low)),
            "medium" => Ok(songbird_errors::evolved_success(SecurityLevel::Medium)),
            "standard" => Ok(songbird_errors::evolved_success(SecurityLevel::Standard)),
            "public" => Ok(songbird_errors::evolved_success(SecurityLevel::Public)),
            "high" => Ok(songbird_errors::evolved_success(SecurityLevel::High)),
            "private" => Ok(songbird_errors::evolved_success(SecurityLevel::Private)),
            "critical" => Ok(songbird_errors::evolved_success(SecurityLevel::Critical)),
            "confidential" => Ok(songbird_errors::evolved_success(
                SecurityLevel::Confidential,
            )),
            "enhanced" => Ok(songbird_errors::evolved_success(SecurityLevel::Enhanced)),
            "maximum" => Ok(songbird_errors::evolved_success(SecurityLevel::Maximum)),
            "classified" => Ok(songbird_errors::evolved_success(SecurityLevel::Classified)),
            _ => Err(SongbirdError::internal_error(internal_error("Unknown security level: {s}")),
        }
    }
}

impl SecurityLevel {
    /// Get the numeric security level (higher = more secure)
    #[must_use]
    pub fn level(&self) -> u8 {
        match self {
            SecurityLevel::None => 0,
            SecurityLevel::Minimal | SecurityLevel::Basic => 1,
            SecurityLevel::Low => 2,
            SecurityLevel::Medium => 3,
            SecurityLevel::Standard | SecurityLevel::Public => 4,
            SecurityLevel::High => 5,
            SecurityLevel::Private => 6,
            SecurityLevel::Critical => 7,
            SecurityLevel::Confidential => 8,
            SecurityLevel::Enhanced => 9,
            SecurityLevel::Maximum => 10,
            SecurityLevel::Classified => 11,
        }
    }

    /// Check if this security level is at least as secure as another
    #[must_use]
    pub fn is_at_least(&self, other: &SecurityLevel) -> bool {
        self.level() >= other.level()
    }

    /// Check if this security level requires encryption
    #[must_use]
    pub fn requires_encryption(&self) -> bool {
        matches!(
            self,
            SecurityLevel::Public
                | SecurityLevel::Private
                | SecurityLevel::Confidential
                | SecurityLevel::Classified
        )
    }

    /// Check if this security level requires authentication
    #[must_use]
    pub fn requires_authentication(&self) -> bool {
        matches!(
            self,
            SecurityLevel::Private | SecurityLevel::Confidential | SecurityLevel::Classified
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_errors::SongbirdResult;

    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Classified.level() > SecurityLevel::Public.level());
        assert!(SecurityLevel::Public.level() > SecurityLevel::None.level());
    }

    #[test]
    fn test_security_level_parsing() -> SongbirdResult<()> {
        assert_eq!(
            "none".parse::<SecurityLevel>().map_err(|e| {
                songbird_errors::SongbirdError::operation_error(format!("Operation failed: {e}"))
            })?,
            SecurityLevel::None
        );
        assert_eq!(
            "minimal".parse::<SecurityLevel>().map_err(|e| {
                songbird_errors::SongbirdError::operation_error(format!("Operation failed: {e}"))
            })?,
            SecurityLevel::Minimal
        );
        assert_eq!(
            "public".parse::<SecurityLevel>().map_err(|e| {
                songbird_errors::SongbirdError::operation_error(format!("Operation failed: {e}"))
            })?,
            SecurityLevel::Public
        );
        Ok(())
    }

    #[test]
    fn test_security_requirements() {
        assert!(!SecurityLevel::None.requires_encryption());
        assert!(SecurityLevel::Public.requires_encryption());
        assert!(!SecurityLevel::Public.requires_authentication());
        assert!(SecurityLevel::Private.requires_authentication());
    }

    #[test]
    fn test_security_level_comparison() {
        assert!(SecurityLevel::Private.is_at_least(&SecurityLevel::Public));
        assert!(!SecurityLevel::Minimal.is_at_least(&SecurityLevel::Public));
    }
}
