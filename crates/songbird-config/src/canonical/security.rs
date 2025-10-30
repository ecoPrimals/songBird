//! Canonical security types and levels

use serde::{Deserialize, Serialize};

/// Security level for services and endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// No security required
    None,
    /// Minimal security (basic validation)
    Minimal,
    /// Basic security (authentication)
    Basic,
    /// Low security level
    Low,
    /// Medium security level
    Medium,
    /// Standard security level
    Standard,
    /// Public access (default)
    #[default]
    Public,
    /// High security level
    High,
    /// Private access
    Private,
    /// Critical security
    Critical,
    /// Confidential data handling
    Confidential,
    /// Enhanced security
    Enhanced,
    /// Maximum security level
    Maximum,
    /// Classified information
    Classified,
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Minimal => write!(f, "minimal"),
            Self::Basic => write!(f, "basic"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::Standard => write!(f, "standard"),
            Self::Public => write!(f, "public"),
            Self::High => write!(f, "high"),
            Self::Private => write!(f, "private"),
            Self::Critical => write!(f, "critical"),
            Self::Confidential => write!(f, "confidential"),
            Self::Enhanced => write!(f, "enhanced"),
            Self::Maximum => write!(f, "maximum"),
            Self::Classified => write!(f, "classified"),
        }
    }
}

impl std::str::FromStr for SecurityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "minimal" => Ok(Self::Minimal),
            "basic" => Ok(Self::Basic),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "standard" => Ok(Self::Standard),
            "public" => Ok(Self::Public),
            "high" => Ok(Self::High),
            "private" => Ok(Self::Private),
            "critical" => Ok(Self::Critical),
            "confidential" => Ok(Self::Confidential),
            "enhanced" => Ok(Self::Enhanced),
            "maximum" => Ok(Self::Maximum),
            "classified" => Ok(Self::Classified),
            _ => Err(format!("Invalid security level: {s}")),
        }
    }
}

impl SecurityLevel {
    /// Get the numeric value of the security level (0-13)
    #[must_use]
    pub fn as_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Minimal => 1,
            Self::Basic => 2,
            Self::Low => 3,
            Self::Medium => 4,
            Self::Standard => 5,
            Self::Public => 6,
            Self::High => 7,
            Self::Private => 8,
            Self::Critical => 9,
            Self::Confidential => 10,
            Self::Enhanced => 11,
            Self::Maximum => 12,
            Self::Classified => 13,
        }
    }

    /// Create from numeric value (0-13)
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Minimal),
            2 => Some(Self::Basic),
            3 => Some(Self::Low),
            4 => Some(Self::Medium),
            5 => Some(Self::Standard),
            6 => Some(Self::Public),
            7 => Some(Self::High),
            8 => Some(Self::Private),
            9 => Some(Self::Critical),
            10 => Some(Self::Confidential),
            11 => Some(Self::Enhanced),
            12 => Some(Self::Maximum),
            13 => Some(Self::Classified),
            _ => None,
        }
    }

    /// Check if this level requires authentication
    #[must_use]
    pub fn requires_authentication(self) -> bool {
        !matches!(self, Self::None | Self::Public)
    }
}
