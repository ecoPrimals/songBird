// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tor protocol primitives (cells, constants)

mod cells;
mod constants;

pub use cells::{CELL_LEN, Cell, CellCommand, RelayCell, RelayCommand};
pub use constants::*;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn tor_protocol_constants_are_consistent() {
        assert_eq!(TOR_PROTOCOL_VERSION, 5);
        assert_eq!(CELL_LEN, 4 + 1 + MAX_CELL_PAYLOAD);
        assert!(MAX_RELAY_PAYLOAD < MAX_CELL_PAYLOAD);
        assert!(STREAM_WINDOW < CIRCUIT_WINDOW);
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
    }

    #[test]
    fn cell_wire_encoding_is_deterministic_for_same_logical_cell() {
        let a = Cell {
            circ_id: 0x1234_5678,
            command: CellCommand::RelayEarly,
            payload: vec![0x01, 0x02],
        };
        let b = Cell {
            circ_id: 0x1234_5678,
            command: CellCommand::RelayEarly,
            payload: vec![0x01, 0x02],
        };
        assert_eq!(a.encode(), b.encode());
    }
}
