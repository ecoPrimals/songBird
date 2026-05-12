// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::{StunError, StunResult};

/// STUN/TURN message types (RFC 5389 + RFC 5766).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Binding Request (0x0001)
    BindingRequest,

    /// Binding Success Response (0x0101)
    BindingResponse,

    /// Binding Error Response (0x0111)
    BindingError,

    /// TURN Allocate (0x0003 — RFC 5766 §6)
    Allocate,

    /// TURN Allocate Success (0x0103)
    AllocateSuccess,

    /// TURN Allocate Error (0x0113)
    AllocateError,

    /// TURN Refresh (0x0004 — RFC 5766 §7)
    Refresh,

    /// TURN Refresh Success (0x0104)
    RefreshSuccess,

    /// TURN `CreatePermission` (0x0008 — RFC 5766 §9)
    CreatePermission,

    /// TURN `CreatePermission` Success (0x0108)
    CreatePermissionSuccess,

    /// TURN `ChannelBind` (0x0009 — RFC 5766 §11)
    ChannelBind,

    /// TURN `ChannelBind` Success (0x0109)
    ChannelBindSuccess,

    /// TURN Send Indication (0x0016 — RFC 5766 §10)
    SendIndication,

    /// TURN Data Indication (0x0017 — RFC 5766 §10)
    DataIndication,
}

impl MessageType {
    /// Convert to wire format (u16)
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        match self {
            Self::BindingRequest => 0x0001,
            Self::BindingResponse => 0x0101,
            Self::BindingError => 0x0111,
            Self::Allocate => 0x0003,
            Self::AllocateSuccess => 0x0103,
            Self::AllocateError => 0x0113,
            Self::Refresh => 0x0004,
            Self::RefreshSuccess => 0x0104,
            Self::CreatePermission => 0x0008,
            Self::CreatePermissionSuccess => 0x0108,
            Self::ChannelBind => 0x0009,
            Self::ChannelBindSuccess => 0x0109,
            Self::SendIndication => 0x0016,
            Self::DataIndication => 0x0017,
        }
    }

    /// Parse from wire format
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a recognized message type.
    pub fn from_u16(value: u16) -> StunResult<Self> {
        match value {
            0x0001 => Ok(Self::BindingRequest),
            0x0101 => Ok(Self::BindingResponse),
            0x0111 => Ok(Self::BindingError),
            0x0003 => Ok(Self::Allocate),
            0x0103 => Ok(Self::AllocateSuccess),
            0x0113 => Ok(Self::AllocateError),
            0x0004 => Ok(Self::Refresh),
            0x0104 => Ok(Self::RefreshSuccess),
            0x0008 => Ok(Self::CreatePermission),
            0x0108 => Ok(Self::CreatePermissionSuccess),
            0x0009 => Ok(Self::ChannelBind),
            0x0109 => Ok(Self::ChannelBindSuccess),
            0x0016 => Ok(Self::SendIndication),
            0x0017 => Ok(Self::DataIndication),
            _ => Err(StunError::InvalidResponse(format!("Unknown message type: 0x{value:04x}"))),
        }
    }
}

/// STUN attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    /// MAPPED-ADDRESS (0x0001)
    MappedAddress,

    /// USERNAME (0x0006) — RFC 5389 short-term credentials
    Username,

    /// MESSAGE-INTEGRITY (0x0008) — HMAC-SHA1 over the STUN message
    MessageIntegrity,

    /// REALM (0x0014) — RFC 5389 long-term credentials
    Realm,

    /// NONCE (0x0015) — RFC 5389 long-term credentials
    Nonce,

    /// XOR-MAPPED-ADDRESS (0x0020) - preferred
    XorMappedAddress,

    /// FINGERPRINT (0x8028) — CRC-32 XOR 0x5354554E
    Fingerprint,

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
            Self::Username => 0x0006,
            Self::MessageIntegrity => 0x0008,
            Self::Realm => 0x0014,
            Self::Nonce => 0x0015,
            Self::XorMappedAddress => 0x0020,
            Self::Fingerprint => 0x8028,
            Self::OtherAddress => 0x802C,
            Self::Unknown(value) => value,
        }
    }

    /// Parse from wire format
    #[must_use]
    pub const fn from_u16(value: u16) -> Self {
        match value {
            0x0001 => Self::MappedAddress,
            0x0006 => Self::Username,
            0x0008 => Self::MessageIntegrity,
            0x0014 => Self::Realm,
            0x0015 => Self::Nonce,
            0x0020 => Self::XorMappedAddress,
            0x8028 => Self::Fingerprint,
            0x802C => Self::OtherAddress,
            _ => Self::Unknown(value),
        }
    }
}
