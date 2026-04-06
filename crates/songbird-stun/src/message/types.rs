// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::{StunError, StunResult};

/// STUN message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Binding Request (0x0001)
    BindingRequest,

    /// Binding Success Response (0x0101)
    BindingResponse,

    /// Binding Error Response (0x0111)
    BindingError,
}

impl MessageType {
    /// Convert to wire format (u16)
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        match self {
            Self::BindingRequest => 0x0001,
            Self::BindingResponse => 0x0101,
            Self::BindingError => 0x0111,
        }
    }

    /// Parse from wire format
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a valid message type.
    pub fn from_u16(value: u16) -> StunResult<Self> {
        match value {
            0x0001 => Ok(Self::BindingRequest),
            0x0101 => Ok(Self::BindingResponse),
            0x0111 => Ok(Self::BindingError),
            _ => Err(StunError::InvalidResponse(format!("Unknown message type: 0x{value:04x}"))),
        }
    }
}

/// STUN attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    /// MAPPED-ADDRESS (0x0001)
    MappedAddress,

    /// XOR-MAPPED-ADDRESS (0x0020) - preferred
    XorMappedAddress,

    /// OTHER-ADDRESS (0x802C) - for NAT type detection
    OtherAddress,

    /// Unknown attribute
    Unknown(u16),
}

impl AttributeType {
    /// Convert to wire format (u16)
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        match self {
            Self::MappedAddress => 0x0001,
            Self::XorMappedAddress => 0x0020,
            Self::OtherAddress => 0x802C,
            Self::Unknown(value) => value,
        }
    }

    /// Parse from wire format
    #[must_use]
    pub const fn from_u16(value: u16) -> Self {
        match value {
            0x0001 => Self::MappedAddress,
            0x0020 => Self::XorMappedAddress,
            0x802C => Self::OtherAddress,
            _ => Self::Unknown(value),
        }
    }
}
