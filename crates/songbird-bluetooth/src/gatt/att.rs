// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! ATT opcodes and attribute UUID constants used by the GATT client.

/// ATT opcodes
/// Note: Constants awaiting hardware validation - will be used in Phase 3
#[allow(dead_code)]
pub mod att_opcode {
    pub const ERROR_RSP: u8 = 0x01;
    pub const READ_BY_GROUP_TYPE_REQ: u8 = 0x10;
    pub const READ_BY_GROUP_TYPE_RSP: u8 = 0x11;
    pub const READ_BY_TYPE_REQ: u8 = 0x08;
    pub const READ_BY_TYPE_RSP: u8 = 0x09;
    pub const READ_REQ: u8 = 0x0A;
    pub const READ_RSP: u8 = 0x0B;
    pub const WRITE_REQ: u8 = 0x12;
    pub const WRITE_RSP: u8 = 0x13;
    pub const WRITE_CMD: u8 = 0x52;
    pub const HANDLE_VALUE_NTF: u8 = 0x1B;
}

/// ATT UUIDs
/// Note: Constants awaiting hardware validation - will be used in Phase 3
#[allow(dead_code)]
pub mod att_uuid {
    /// Primary Service UUID (0x2800)
    pub const PRIMARY_SERVICE: u16 = 0x2800;

    /// Characteristic UUID (0x2803)
    pub const CHARACTERISTIC: u16 = 0x2803;

    /// Client Characteristic Configuration Descriptor (0x2902)
    pub const CLIENT_CHAR_CONFIG: u16 = 0x2902;
}
