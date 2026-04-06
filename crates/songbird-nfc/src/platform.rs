// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
impl NfcDevice {
    /// Construct with a custom backend for unit tests (same crate only).
    pub(crate) fn from_backend_for_test(backend: Box<dyn NfcBackend>, timeout: Duration) -> Self {
        Self {
            backend,
            timeout,
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::redundant_pub_crate,
    reason = "test_support is crate-private; pub(crate) is intentional"
)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
pub(crate) mod test_support {
    //! In-crate test doubles for [`NfcBackend`].

    use super::{NfcBackend, NfcDevice};
    use crate::error::{NfcError, Result};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// Records sends and serves scripted receive bytes in FIFO order.
    pub(crate) struct ScriptedBackend {
        recv_buf: Arc<Mutex<Vec<u8>>>,
        sent_frames: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl ScriptedBackend {
        pub(crate) fn new(recv_buf: Vec<u8>) -> Self {
            Self {
                recv_buf: Arc::new(Mutex::new(recv_buf)),
                sent_frames: Arc::new(Mutex::new(Vec::new())),
            }
        }

        /// Clone of the shared send log for assertions after [`Self::into_device`].
        pub(crate) fn sent_frames_handle(&self) -> Arc<Mutex<Vec<Vec<u8>>>> {
            Arc::clone(&self.sent_frames)
        }

        pub(crate) fn into_device(self, timeout: Duration) -> NfcDevice {
            NfcDevice::from_backend_for_test(Box::new(self), timeout)
        }
    }

    #[async_trait::async_trait]
    impl NfcBackend for ScriptedBackend {
        async fn connect(&mut self, _timeout: Duration) -> Result<()> {
            Ok(())
        }

        async fn disconnect(&mut self) -> Result<()> {
            Ok(())
        }

        async fn send(&mut self, data: &[u8]) -> Result<()> {
            self.sent_frames.lock().expect("scripted backend lock").push(data.to_vec());
            Ok(())
        }

        async fn receive(&mut self, expected_len: usize) -> Result<Vec<u8>> {
            let mut buf = self.recv_buf.lock().expect("scripted backend lock");
            if buf.len() < expected_len {
                return Err(NfcError::ConnectionLost);
            }
            let chunk: Vec<u8> = buf.drain(..expected_len).collect();
            Ok(chunk)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::NfcDevice;
    use super::test_support::ScriptedBackend;
    use crate::protocol::NfcMessage;
    use crate::{MSG_TYPE_GENESIS_REQUEST, PROTOCOL_VERSION, PUBLIC_KEY_SIZE, SIGNATURE_SIZE};
    use std::time::Duration;

    #[test]
    fn new_reports_platform_unsupported() {
        let res = NfcDevice::new(Duration::from_secs(1));
        let err = res.expect_err("native NFC backend is not wired in this crate yet");
        match err {
            crate::NfcError::PlatformUnsupported(msg) => {
                assert!(
                    !msg.is_empty(),
                    "unsupported message should explain why NFC is unavailable"
                );
            }
            other => panic!("expected PlatformUnsupported, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_and_receive_message_roundtrip_on_scripted_backend() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0x11u8; PUBLIC_KEY_SIZE],
            [0x22u8; crate::NONCE_SIZE],
            vec![0x33u8; 7],
            [0x44u8; SIGNATURE_SIZE],
        );
        let wire = msg.to_bytes().expect("valid test frame");

        let backend = ScriptedBackend::new(Vec::new());
        let sent = backend.sent_frames_handle();
        let mut device = backend.into_device(Duration::from_secs(2));

        device.send_message(&msg).await.expect("send_message should serialize and send");
        let frames = sent.lock().expect("sent_frames lock");
        assert_eq!(frames.len(), 1, "send_message should produce exactly one raw frame");
        assert_eq!(frames[0], wire, "raw bytes should match NfcMessage::to_bytes");

        drop(frames);
        let mut device2 = ScriptedBackend::new(wire.clone()).into_device(Duration::from_secs(2));
        let got =
            device2.receive_message().await.expect("receive_message should parse scripted wire");
        assert_eq!(got.version, PROTOCOL_VERSION);
        assert_eq!(got.msg_type, MSG_TYPE_GENESIS_REQUEST);
        assert_eq!(got.encrypted_payload.len(), 7);
    }

    #[tokio::test]
    async fn receive_raw_errors_when_buffer_underruns() {
        let mut device = ScriptedBackend::new(vec![0u8; PUBLIC_KEY_SIZE - 1])
            .into_device(Duration::from_secs(1));
        let err = device
            .receive_raw(PUBLIC_KEY_SIZE)
            .await
            .expect_err("short buffer should surface connection loss");
        assert!(
            matches!(err, crate::NfcError::ConnectionLost),
            "expected ConnectionLost on underrun, got {err:?}"
        );
    }
}
