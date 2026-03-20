// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-specific NFC device abstraction
//!
//! Provides a unified interface across targets. **Capability discovery** is explicit:
//! [`NfcDevice::new`] returns [`NfcError::PlatformUnsupported`]
//! until a native backend is wired for that OS.
//!
//! ## Discovery / integration paths (by target)
//!
//! - **Android** — JNI bridge to `android.nfc.NfcAdapter`, `NfcManager`, and tag technologies
//!   (`IsoDep`, `NfcA`, etc.). Discover adapters from Java/Kotlin (`NfcAdapter.getDefault()`),
//!   then surface session lifecycle + I/O into Rust via `jni-rs` (or equivalent).
//! - **iOS** — [CoreNFC](https://developer.apple.com/documentation/corenfc): `NFCNDEFReaderSession` /
//!   `NFCTagReaderSession` in Swift/Objective-C, bridged to Rust; capability is
//!   `NFCReaderUsageDescription` + device hardware, queried at runtime from the app bundle.
//! - **Linux** — [libnfc](https://github.com/nfc-tools/libnfc) or kernel NFC sockets (`AF_NFC`),
//!   typically after listing readers via `nfc-list` / udev; PC/SC may apply for some hardware.

use crate::error::{NfcError, Result};
use crate::protocol::NfcMessage;
use std::time::Duration;
use tracing::{debug, info};

/// NFC device abstraction
///
/// Platform-agnostic interface for NFC operations.
pub struct NfcDevice {
    /// Platform-specific backend
    backend: Box<dyn NfcBackend>,

    /// Connection timeout
    timeout: Duration,
}

impl std::fmt::Debug for NfcDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NfcDevice").field("timeout", &self.timeout).finish_non_exhaustive()
    }
}

impl NfcDevice {
    /// Create new NFC device
    ///
    /// # Errors
    ///
    /// Returns [`NfcError::PlatformUnsupported`] when
    /// no backend is linked for this target yet, or the host stack cannot be opened.
    pub fn new(timeout: Duration) -> Result<Self> {
        let backend = Self::create_platform_backend()?;

        Ok(Self {
            backend,
            timeout,
        })
    }

    /// Connect to peer device
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails or times out.
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to NFC peer");
        self.backend.connect(self.timeout).await
    }

    /// Disconnect from peer
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails.
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from NFC peer");
        self.backend.disconnect().await
    }

    /// Send raw bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the send operation fails.
    pub async fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        debug!("Sending {} bytes", data.len());
        self.backend.send(data).await
    }

    /// Receive raw bytes
    ///
    /// # Errors
    ///
    /// Returns an error if receive fails or times out.
    pub async fn receive_raw(&mut self, expected_len: usize) -> Result<Vec<u8>> {
        debug!("Receiving {} bytes", expected_len);
        self.backend.receive(expected_len).await
    }

    /// Send NFC message
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or send fails.
    pub async fn send_message(&mut self, message: &NfcMessage) -> Result<()> {
        let bytes = message.to_bytes()?;
        self.send_raw(&bytes).await
    }

    /// Receive NFC message
    ///
    /// # Errors
    ///
    /// Returns an error if receive or deserialization fails.
    pub async fn receive_message(&mut self) -> Result<NfcMessage> {
        // First receive header to get payload length
        let header = self.receive_raw(crate::HEADER_SIZE).await?;

        let payload_len = u16::from_be_bytes([header[2], header[3]]) as usize;
        let total_len = crate::FRAME_OVERHEAD + payload_len;

        // Receive full frame
        let mut full_frame = header;
        let remaining = self.receive_raw(total_len - crate::HEADER_SIZE).await?;
        full_frame.extend_from_slice(&remaining);

        NfcMessage::from_bytes(&full_frame)
    }

    /// Create platform-specific backend
    fn create_platform_backend() -> Result<Box<dyn NfcBackend>> {
        #[cfg(target_os = "android")]
        {
            // Capability path: JNI → `NfcAdapter` / tag tech; see module rustdoc.
            Err(NfcError::PlatformUnsupported(
                "Android NFC requires a JNI bridge to android.nfc (NfcAdapter, Tag, IsoDep, etc.)"
                    .to_string(),
            ))
        }

        #[cfg(target_os = "ios")]
        {
            // Capability path: CoreNFC in Swift + FFI; see module rustdoc.
            Err(NfcError::PlatformUnsupported(
                "iOS NFC requires CoreNFC (NFCNDEFReaderSession / tag sessions) via a Swift bridge"
                    .to_string(),
            ))
        }

        #[cfg(target_os = "linux")]
        {
            // Capability path: libnfc or AF_NFC; see module rustdoc.
            Err(NfcError::PlatformUnsupported(
                "Linux NFC requires libnfc or kernel NFC (AF_NFC) with a discovered USB/PCMCIA reader"
                    .to_string(),
            ))
        }

        #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "linux")))]
        {
            Err(NfcError::PlatformUnsupported(
                "NFC backend not available for this target OS".to_string(),
            ))
        }
    }
}

/// Platform-specific NFC backend trait
#[async_trait::async_trait]
pub trait NfcBackend: Send + Sync {
    /// Connect to peer
    async fn connect(&mut self, timeout: Duration) -> Result<()>;

    /// Disconnect from peer
    async fn disconnect(&mut self) -> Result<()>;

    /// Send data
    async fn send(&mut self, data: &[u8]) -> Result<()>;

    /// Receive data
    async fn receive(&mut self, expected_len: usize) -> Result<Vec<u8>>;
}
