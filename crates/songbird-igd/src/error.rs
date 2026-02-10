//! Error types for IGD operations

use std::io;
use thiserror::Error;

/// Result type alias for IGD operations
pub type Result<T> = std::result::Result<T, IgdError>;

/// Errors that can occur during IGD operations
#[derive(Debug, Error)]
pub enum IgdError {
    /// No gateway found (neither UPnP nor NAT-PMP)
    #[error("No IGD-capable gateway found on network")]
    NoGatewayFound,

    /// SSDP discovery failed
    #[error("SSDP discovery failed: {0}")]
    SsdpError(String),

    /// SOAP action failed
    #[error("SOAP action failed: {0}")]
    SoapError(String),

    /// NAT-PMP operation failed
    #[error("NAT-PMP error: {0}")]
    NatPmpError(String),

    /// Port mapping conflict
    #[error("Port mapping conflict: port {0} already mapped to {1}")]
    MappingConflict(u16, String),

    /// Port mapping denied by router
    #[error("Port mapping denied by router: {0}")]
    MappingDenied(String),

    /// Invalid gateway response
    #[error("Invalid gateway response: {0}")]
    InvalidResponse(String),

    /// Network I/O error
    #[error("Network I/O error: {0}")]
    Io(#[from] io::Error),

    /// Timeout waiting for gateway response
    #[error("Gateway response timeout")]
    Timeout,

    /// Gateway not reachable
    #[error("Gateway not reachable at {0}")]
    GatewayUnreachable(String),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    XmlParse(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Protocol not supported by gateway
    #[error("Protocol {0} not supported by this gateway")]
    ProtocolNotSupported(String),
}

/// SOAP-specific error codes from UPnP specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoapErrorCode {
    /// Action failed (general error)
    ActionFailed = 501,
    /// Argument value invalid
    ArgumentValueInvalid = 402,
    /// Argument value out of range
    ArgumentValueOutOfRange = 603,
    /// Conflict in mapping entry
    ConflictInMappingEntry = 718,
    /// Only permanent leases supported
    OnlyPermanentLeasesSupported = 725,
    /// Remote host only supports wildcard
    RemoteHostOnlySupportsWildcard = 726,
    /// External port only supports wildcard
    ExternalPortOnlySupportsWildcard = 727,
    /// No such entry in array
    NoSuchEntryInArray = 714,
    /// Invalid array index
    InvalidArrayIndex = 713,
}

impl SoapErrorCode {
    /// Parse SOAP error code from response
    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            501 => Some(Self::ActionFailed),
            402 => Some(Self::ArgumentValueInvalid),
            603 => Some(Self::ArgumentValueOutOfRange),
            718 => Some(Self::ConflictInMappingEntry),
            725 => Some(Self::OnlyPermanentLeasesSupported),
            726 => Some(Self::RemoteHostOnlySupportsWildcard),
            727 => Some(Self::ExternalPortOnlySupportsWildcard),
            714 => Some(Self::NoSuchEntryInArray),
            713 => Some(Self::InvalidArrayIndex),
            _ => None,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::ActionFailed => "Action failed",
            Self::ArgumentValueInvalid => "Argument value invalid",
            Self::ArgumentValueOutOfRange => "Argument value out of range",
            Self::ConflictInMappingEntry => "Port already mapped to another device",
            Self::OnlyPermanentLeasesSupported => "Router only supports permanent leases",
            Self::RemoteHostOnlySupportsWildcard => "Remote host must be wildcard",
            Self::ExternalPortOnlySupportsWildcard => "External port must be wildcard",
            Self::NoSuchEntryInArray => "No such mapping exists",
            Self::InvalidArrayIndex => "Invalid mapping index",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soap_error_codes() {
        assert_eq!(SoapErrorCode::from_code(718), Some(SoapErrorCode::ConflictInMappingEntry));
        assert_eq!(SoapErrorCode::from_code(999), None);
    }

    #[test]
    fn test_error_descriptions() {
        assert_eq!(
            SoapErrorCode::ConflictInMappingEntry.description(),
            "Port already mapped to another device"
        );
    }
}
