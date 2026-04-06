// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! ATT opcodes and attribute UUID constants used by the GATT client.

/// ATT opcodes
/// Note: Constants awaiting hardware validation - will be used in Phase 3
pub mod att_opcode {
    pub const ERROR_RSP: u8 = 0x01;
    pub const READ_BY_GROUP_TYPE_REQ: u8 = 0x10;
    pub const READ_BY_GROUP_TYPE_RSP: u8 = 0x11;
    pub const READ_BY_TYPE_REQ: u8 = 0x08;
    #[allow(dead_code)]
    pub const READ_BY_TYPE_RSP: u8 = 0x09;
    pub const READ_REQ: u8 = 0x0A;
    #[allow(dead_code)]
    pub const READ_RSP: u8 = 0x0B;
    pub const WRITE_REQ: u8 = 0x12;
    #[allow(dead_code)]
    pub const WRITE_RSP: u8 = 0x13;
    pub const WRITE_CMD: u8 = 0x52;
    #[allow(dead_code)]
    pub const HANDLE_VALUE_NTF: u8 = 0x1B;
}

/// ATT UUIDs
/// Note: Constants awaiting hardware validation - will be used in Phase 3
pub mod att_uuid {
    /// Primary Service UUID (0x2800)
    pub const PRIMARY_SERVICE: u16 = 0x2800;

    /// Characteristic UUID (0x2803)
    pub const CHARACTERISTIC: u16 = 0x2803;

    /// Client Characteristic Configuration Descriptor (0x2902)
    #[allow(dead_code)]
    pub const CLIENT_CHAR_CONFIG: u16 = 0x2902;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{att_opcode, att_uuid};

    #[test]
    fn att_request_response_opcode_pairs_are_distinct() {
        assert_ne!(att_opcode::READ_REQ, att_opcode::READ_RSP);
        assert_ne!(att_opcode::WRITE_REQ, att_opcode::WRITE_RSP);
        assert_ne!(att_opcode::READ_BY_GROUP_TYPE_REQ, att_opcode::READ_BY_GROUP_TYPE_RSP);
    }

    #[test]
    fn att_uuid_constants_match_assigned_numbers() {
        assert_eq!(att_uuid::PRIMARY_SERVICE, 0x2800);
        assert_eq!(att_uuid::CHARACTERISTIC, 0x2803);
        assert_eq!(att_uuid::CLIENT_CHAR_CONFIG, 0x2902);
    }

    #[test]
    fn notification_and_write_cmd_opcodes_are_unique() {
        assert_eq!(att_opcode::HANDLE_VALUE_NTF, 0x1B);
        assert_eq!(att_opcode::WRITE_CMD, 0x52);
        assert_ne!(att_opcode::HANDLE_VALUE_NTF, att_opcode::WRITE_CMD);
    }
}
