// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HCI Controller adapter for trouble-host
//!
//! Simplified controller implementation focusing on async operations.
//! This bridges our Transport trait to work with trouble-host's BLE stack.

use crate::{error::Result, transport::Transport};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, trace};

/// Controller adapter for trouble-host
///
/// Adapts our `Transport` trait to provide HCI command/event handling.
/// Simplified implementation focusing on core functionality.
pub struct ControllerAdapter<T: Transport> {
    transport: Arc<Mutex<T>>,
}

impl<T: Transport> ControllerAdapter<T> {
    /// Create new controller adapter
    pub fn new(transport: Arc<Mutex<T>>) -> Self {
        debug!("Creating HCI controller adapter");
        Self {
            transport,
        }
    }

    /// Send HCI command
    pub async fn send_command(&self, data: &[u8]) -> Result<()> {
        trace!("Sending HCI command: {} bytes", data.len());
        let mut transport = self.transport.lock().await;
        transport.send_command(data).await
    }

    /// Receive HCI event
    pub async fn receive_event(&self) -> Result<Vec<u8>> {
        let event = {
            let mut transport = self.transport.lock().await;
            transport.receive_event().await?
        };
        trace!("Received HCI event: {} bytes", event.len());
        Ok(event)
    }

    /// Check if controller is connected
    ///
    /// Note: Awaiting hardware validation - will be used in Phase 3 testing
    #[expect(
        dead_code,
        reason = "public HCI helper; only exercised from unit tests in this crate today"
    )]
    pub async fn is_connected(&self) -> bool {
        let transport = self.transport.lock().await;
        transport.is_connected()
    }
}

/// HCI packet types
#[expect(dead_code, reason = "reserved for Phase 3 HCI framing")]
mod hci_packet {
    pub const COMMAND: u8 = 0x01;
    pub const ACL_DATA: u8 = 0x02;
    pub const SCO_DATA: u8 = 0x03;
    pub const EVENT: u8 = 0x04;
}

/// HCI Command opcodes
#[expect(dead_code, reason = "reserved for Phase 3 HCI command path")]
mod hci_opcode {
    pub const RESET: u16 = 0x0C03;
    pub const LE_SET_SCAN_PARAMETERS: u16 = 0x200B;
    pub const LE_SET_SCAN_ENABLE: u16 = 0x200C;
    pub const LE_CREATE_CONNECTION: u16 = 0x200D;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    /// Mock transport for testing
    struct MockTransport {
        connected: bool,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                connected: true,
            }
        }
    }

    impl Transport for MockTransport {
        fn transport_type(&self) -> crate::transport::TransportType {
            crate::transport::TransportType::Usb
        }

        async fn send_command(&mut self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive_event(&mut self) -> Result<Vec<u8>> {
            Ok(vec![0x04, 0x0E, 0x04, 0x01, 0x03, 0x0C, 0x00])
        }

        async fn send_acl(&mut self, _data: &[u8]) -> Result<()> {
            Ok(())
        }

        async fn receive_acl(&mut self) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        async fn close(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_controller_adapter_creation() {
        let transport = MockTransport::new();
        let adapter = ControllerAdapter::new(Arc::new(Mutex::new(transport)));
        assert!(adapter.is_connected().await);
    }

    #[tokio::test]
    async fn test_send_command() {
        let transport = MockTransport::new();
        let adapter = ControllerAdapter::new(Arc::new(Mutex::new(transport)));

        let result = adapter.send_command(&[0x01, 0x03, 0x0C, 0x00]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_receive_event() {
        let transport = MockTransport::new();
        let adapter = ControllerAdapter::new(Arc::new(Mutex::new(transport)));

        let result = adapter.receive_event().await;
        assert!(result.is_ok());
        let event = match result {
            Ok(e) => e,
            Err(e) => panic!("receive_event: {e:?}"),
        };
        assert!(!event.is_empty());
    }
}
